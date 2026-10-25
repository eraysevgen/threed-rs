use rayon::prelude::*;
use threed_rs_core::{FeatureConfig, PointCloud};

use crate::error::{FeatureError, Result};
use crate::geometric::GeometricFeatures;
use crate::height::HeightFeatures;
use crate::pca::PcaResult;

/// Parallel pipeline for extracting local geometric, height, density, and coordinate features.
pub struct FeaturePipeline<'a> {
    host_cloud: &'a PointCloud,
    query_cloud: &'a PointCloud,
    config: FeatureConfig,
}

impl<'a> FeaturePipeline<'a> {
    /// Creates a new `FeaturePipeline`.
    pub fn new(
        host_cloud: &'a PointCloud,
        query_cloud: &'a PointCloud,
        config: FeatureConfig,
    ) -> Self {
        Self {
            host_cloud,
            query_cloud,
            config,
        }
    }

    /// Computes features for a single query point given its host neighbor indices.
    pub fn extract_single(&self, query_idx: usize, neighbor_indices: &[usize], out: &mut Vec<f64>) {
        let query_point = self.query_cloud[query_idx];
        let num_neighbors = neighbor_indices.len();

        // 1. Degenerate neighborhood check (< 3 points)
        if num_neighbors < 3 {
            out.extend_from_slice(&GeometricFeatures::nan().to_array());

            if self.config.add_height {
                out.extend_from_slice(&HeightFeatures::nan().to_array());
            }

            if self.config.add_density {
                out.push(num_neighbors as f64);
            }

            if self.config.add_xyz {
                out.push(query_point.x);
                out.push(query_point.y);
                out.push(query_point.z);
            }
            return;
        }

        // 2. Collect neighbor points
        let mut neighbors = Vec::with_capacity(num_neighbors);
        for &idx in neighbor_indices {
            if let Some(pt) = self.host_cloud.get(idx) {
                neighbors.push(*pt);
            }
        }

        // 3. Compute PCA and geometric features
        let geo_features = match PcaResult::from_points(&neighbors) {
            Ok(pca) => GeometricFeatures::from_pca(&pca),
            Err(_) => GeometricFeatures::nan(),
        };
        out.extend_from_slice(&geo_features.to_array());

        // 4. Optionally compute 5 height features
        if self.config.add_height {
            let height_features = HeightFeatures::compute(query_point.z, &neighbors);
            out.extend_from_slice(&height_features.to_array());
        }

        // 5. Optionally compute density feature
        if self.config.add_density {
            out.push(num_neighbors as f64);
        }

        // 6. Optionally append raw (X, Y, Z) coordinates
        if self.config.add_xyz {
            out.push(query_point.x);
            out.push(query_point.y);
            out.push(query_point.z);
        }
    }

    /// Computes features in parallel using Rayon across all query points.
    ///
    /// Accepts a slice of neighbor index lists (one for each query point).
    /// Returns a 2D matrix (`Vec<Vec<f64>>`) where each row corresponds to a query point.
    pub fn compute_parallel(&self, neighborhood_indices: &[Vec<usize>]) -> Result<Vec<Vec<f64>>> {
        if self.query_cloud.len() != neighborhood_indices.len() {
            return Err(FeatureError::MismatchedLengths {
                queries: self.query_cloud.len(),
                indices: neighborhood_indices.len(),
            });
        }

        let num_features = self.config.total_features();

        let results: Vec<Vec<f64>> = (0..self.query_cloud.len())
            .into_par_iter()
            .map(|idx| {
                let mut row = Vec::with_capacity(num_features);
                self.extract_single(idx, &neighborhood_indices[idx], &mut row);
                row
            })
            .collect();

        Ok(results)
    }
}

/// Convenience entry point matching original C++ `threed::compute_features`.
pub fn compute_features(
    host_cloud: &PointCloud,
    query_cloud: &PointCloud,
    neighborhood_indices: &[Vec<usize>],
    config: FeatureConfig,
) -> Result<Vec<Vec<f64>>> {
    let pipeline = FeaturePipeline::new(host_cloud, query_cloud, config);
    pipeline.compute_parallel(neighborhood_indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use threed_rs_core::Point3D;

    #[test]
    fn test_feature_pipeline_parallel() {
        let host = PointCloud::from_points(vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
        ]);

        let query = PointCloud::from_points(vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 10.0), // isolated point
        ]);

        let indices = vec![
            vec![0, 1, 2, 3], // 4 neighbors
            vec![0],          // 1 neighbor (< 3 -> NaN)
        ];

        let config = FeatureConfig {
            add_height: true,
            add_density: true,
            add_xyz: true,
        };

        let features = compute_features(&host, &query, &indices, config).unwrap();
        assert_eq!(features.len(), 2);
        assert_eq!(features[0].len(), 18);
        assert_eq!(features[1].len(), 18);

        // First row should have real values
        assert!(!features[0][0].is_nan());
        assert_eq!(features[0][14], 4.0); // density = 4
        assert_eq!(features[0][15], 0.0); // x
        assert_eq!(features[0][16], 0.0); // y
        assert_eq!(features[0][17], 0.0); // z

        // Second row (<3 points) should have NaN geometric and height features
        assert!(features[1][0].is_nan());
        assert!(features[1][9].is_nan());
        assert_eq!(features[1][14], 1.0); // density = 1
        assert_eq!(features[1][15], 10.0); // x
        assert_eq!(features[1][16], 10.0); // y
        assert_eq!(features[1][17], 10.0); // z
    }
}
