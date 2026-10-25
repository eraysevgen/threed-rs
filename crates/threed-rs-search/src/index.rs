use kiddo::{ImmutableKdTree, SquaredEuclidean};
use rayon::prelude::*;
use std::num::NonZero;
use threed_rs_core::{Point3D, PointCloud};

use crate::error::{Result, SearchError};
use crate::query::{NeighborItem, NeighborhoodMethod};

/// High-performance spatial search index based on Kiddo's SIMD-accelerated ImmutableKdTree.
pub struct KdTreeIndex {
    tree: ImmutableKdTree<f64, 3>,
    size: usize,
}

impl KdTreeIndex {
    /// Builds a spatial index from a slice of `[f64; 3]` arrays using parallel construction.
    pub fn build_from_arrays(arrays: &[[f64; 3]]) -> Result<Self> {
        if arrays.is_empty() {
            return Err(SearchError::EmptyPointCloud);
        }

        let tree = ImmutableKdTree::<f64, 3>::new_from_slice_parallel(arrays)
            .map_err(|e| SearchError::TreeBuildError(format!("{e:?}")))?;

        Ok(Self {
            size: tree.size(),
            tree,
        })
    }

    /// Builds a spatial index from a slice of `Point3D` using parallel construction.
    pub fn build_from_points(points: &[Point3D]) -> Result<Self> {
        if points.is_empty() {
            return Err(SearchError::EmptyPointCloud);
        }

        let arrays: Vec<[f64; 3]> = points.iter().map(|p| p.to_array()).collect();
        Self::build_from_arrays(&arrays)
    }

    /// Builds a spatial index from a `PointCloud` using parallel construction.
    pub fn build_from_cloud(cloud: &PointCloud) -> Result<Self> {
        Self::build_from_points(cloud.as_slice())
    }

    /// Returns the number of points in the spatial index.
    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Checks if the index is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Searches for the `k` nearest neighbors of a query point.
    ///
    /// Results are returned in ascending order of distance.
    pub fn search_knn(&self, query: &[f64; 3], k: usize) -> Result<Vec<NeighborItem>> {
        if k == 0 {
            return Err(SearchError::ZeroNeighborsRequested);
        }

        let k_clamped = k.min(self.size);
        let non_zero_k = NonZero::new(k_clamped).unwrap();

        let results = self
            .tree
            .query(query)
            .nearest_n::<SquaredEuclidean<f64>>(non_zero_k)
            .execute();

        Ok(results
            .into_iter()
            .map(|item| NeighborItem::new(item.item as usize, item.distance))
            .collect())
    }

    /// Searches for all neighbors within a given `radius`.
    ///
    /// If `max_k` is specified, caps the maximum number of neighbors returned.
    pub fn search_radius(
        &self,
        query: &[f64; 3],
        radius: f64,
        max_k: Option<usize>,
    ) -> Result<Vec<NeighborItem>> {
        if radius <= 0.0 || radius.is_nan() {
            return Err(SearchError::InvalidRadius(radius));
        }

        let radius_sq = radius * radius;

        let results = match max_k {
            Some(limit) => {
                if limit == 0 {
                    return Err(SearchError::InvalidParameter(
                        "max_k in radius search must be greater than zero".into(),
                    ));
                }
                let clamped_limit = limit.min(self.size);
                let non_zero_limit = NonZero::new(clamped_limit).unwrap();

                self.tree
                    .query(query)
                    .nearest_n::<SquaredEuclidean<f64>>(non_zero_limit)
                    .within(radius_sq)
                    .execute()
            }
            None => self
                .tree
                .query(query)
                .within::<SquaredEuclidean<f64>>(radius_sq)
                .execute(),
        };

        Ok(results
            .into_iter()
            .map(|item| NeighborItem::new(item.item as usize, item.distance))
            .collect())
    }

    /// Searches for neighbors of a query point according to the given `NeighborhoodMethod`.
    pub fn search(
        &self,
        query: &[f64; 3],
        method: &NeighborhoodMethod,
    ) -> Result<Vec<NeighborItem>> {
        match *method {
            NeighborhoodMethod::Knn { k } => self.search_knn(query, k),
            NeighborhoodMethod::Radius { radius, max_k } => {
                self.search_radius(query, radius, max_k)
            }
        }
    }

    /// Searches for neighbor indices only (dropping distances for maximum memory efficiency).
    pub fn search_indices(
        &self,
        query: &[f64; 3],
        method: &NeighborhoodMethod,
    ) -> Result<Vec<usize>> {
        let neighbors = self.search(query, method)?;
        Ok(neighbors.into_iter().map(|n| n.index).collect())
    }

    /// Executes parallel batch search over all points in `query_cloud`,
    /// returning a `Vec` of neighbor indices for each query point.
    pub fn batch_search_indices(
        &self,
        query_cloud: &PointCloud,
        method: &NeighborhoodMethod,
    ) -> Result<Vec<Vec<usize>>> {
        method.validate()?;

        if query_cloud.is_empty() {
            return Ok(Vec::new());
        }

        let results: Vec<Vec<usize>> = query_cloud
            .as_slice()
            .par_iter()
            .map(|p| {
                let coords = p.to_array();
                self.search_indices(&coords, method).unwrap_or_default()
            })
            .collect();

        Ok(results)
    }

    /// Executes parallel batch search over all points in `query_cloud`,
    /// returning full neighbor items (indices + distances) for each query point.
    pub fn batch_search_neighbors(
        &self,
        query_cloud: &PointCloud,
        method: &NeighborhoodMethod,
    ) -> Result<Vec<Vec<NeighborItem>>> {
        method.validate()?;

        if query_cloud.is_empty() {
            return Ok(Vec::new());
        }

        let results: Vec<Vec<NeighborItem>> = query_cloud
            .as_slice()
            .par_iter()
            .map(|p| {
                let coords = p.to_array();
                self.search(&coords, method).unwrap_or_default()
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<Point3D> {
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
            Point3D::new(5.0, 5.0, 5.0),
        ]
    }

    #[test]
    fn test_empty_build() {
        assert_eq!(
            KdTreeIndex::build_from_points(&[]).err(),
            Some(SearchError::EmptyPointCloud)
        );
    }

    #[test]
    fn test_knn_search() {
        let pts = sample_points();
        let index = KdTreeIndex::build_from_points(&pts).unwrap();
        assert_eq!(index.len(), 5);

        let query = [0.1, 0.0, 0.0];
        let neighbors = index.search_knn(&query, 3).unwrap();
        assert_eq!(neighbors.len(), 3);
        assert_eq!(neighbors[0].index, 0);
        assert_eq!(neighbors[1].index, 1);
        assert!(neighbors[0].distance_squared < neighbors[1].distance_squared);
    }

    #[test]
    fn test_radius_search_with_and_without_max_k() {
        let pts = sample_points();
        let index = KdTreeIndex::build_from_points(&pts).unwrap();

        // Query near origin with radius 1.5 (should capture (0,0,0), (1,0,0), (0,1,0), (0,0,1))
        let query = [0.0, 0.0, 0.0];
        let all_in_radius = index.search_radius(&query, 1.5, None).unwrap();
        assert_eq!(all_in_radius.len(), 4);

        // With max_k = 2
        let capped = index.search_radius(&query, 1.5, Some(2)).unwrap();
        assert_eq!(capped.len(), 2);
    }

    #[test]
    fn test_batch_search_indices() {
        let host_pts = sample_points();
        let index = KdTreeIndex::build_from_points(&host_pts).unwrap();

        let query_cloud = PointCloud::from_points(vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(5.1, 5.0, 5.0),
        ]);

        let method = NeighborhoodMethod::knn(2);
        let batch_results = index.batch_search_indices(&query_cloud, &method).unwrap();

        assert_eq!(batch_results.len(), 2);
        assert_eq!(batch_results[0].len(), 2);
        assert_eq!(batch_results[0][0], 0);

        assert_eq!(batch_results[1].len(), 2);
        assert_eq!(batch_results[1][0], 4);
    }
}
