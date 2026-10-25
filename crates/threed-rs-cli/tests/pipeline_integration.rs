use threed_rs_core::{FeatureConfig, Point3D, PointCloud};
use threed_rs_features::compute_features;
use threed_rs_io::{read_features_npy, read_point_cloud, write_features_npy, write_point_cloud};
use threed_rs_search::{KdTreeIndex, NeighborhoodMethod};

#[test]
fn test_end_to_end_pipeline() {
    let temp_dir = std::env::temp_dir();
    let test_ply_path = temp_dir.join("pipeline_test_cloud.ply");
    let test_npy_path = temp_dir.join("pipeline_test_features.npy");

    // 1. Create a synthetic plane of points (Z = 0) with 25 points
    let mut cloud = PointCloud::new();
    for x in 0..5 {
        for y in 0..5 {
            cloud.push(Point3D::new(x as f64, y as f64, 0.0));
        }
    }
    assert_eq!(cloud.len(), 25);

    // 2. Write to PLY and read back via IO module
    write_point_cloud(&test_ply_path, &cloud).expect("Failed to write PLY");
    let loaded_cloud = read_point_cloud(&test_ply_path).expect("Failed to read PLY");
    assert_eq!(loaded_cloud.len(), 25);

    // 3. Build spatial index
    let index = KdTreeIndex::build_from_cloud(&loaded_cloud).expect("Failed to build KD-Tree");

    // 4. Neighborhood search (Radius = 1.5, max_k = 10)
    let method = NeighborhoodMethod::Radius {
        radius: 1.5,
        max_k: Some(10),
    };
    let neighbor_indices = index
        .batch_search_indices(&loaded_cloud, &method)
        .expect("Failed batch search");
    assert_eq!(neighbor_indices.len(), 25);

    // 5. Configure full feature extraction (all 18 features)
    let config = FeatureConfig {
        add_height: true,
        add_density: true,
        add_xyz: true,
    };
    assert_eq!(config.total_features(), 18);

    // 6. Compute features in parallel
    let features = compute_features(&loaded_cloud, &loaded_cloud, &neighbor_indices, config)
        .expect("Failed feature computation");
    assert_eq!(features.len(), 25);
    assert_eq!(features[0].len(), 18);

    // 7. Export to NumPy .npy and read back
    write_features_npy(&test_npy_path, &features).expect("Failed to export NPY");
    let npy_matrix = read_features_npy(&test_npy_path).expect("Failed to read NPY");

    assert_eq!(npy_matrix.shape(), &[25, 18]);

    // Check planarity of the center point (point index 12: x=2, y=2, z=0)
    // Feature order: [0: sum_eigenvalues, 1: linearity, 2: planarity, ...]
    // On a symmetric flat 2D plane: e1 ≈ e2, e3 ≈ 0 -> Linearity ≈ 0, Planarity ≈ 1.0
    let center_linearity = npy_matrix[[12, 1]];
    let center_planarity = npy_matrix[[12, 2]];
    assert!(
        center_linearity < 0.1,
        "Center point on symmetric grid should have low linearity, got {}",
        center_linearity
    );
    assert!(
        center_planarity > 0.9,
        "Center point on flat plane should have high planarity (~1.0), got {}",
        center_planarity
    );

    // Check density feature (index 14 = 9 geometric + 5 height = density is at index 14)
    let center_density = npy_matrix[[12, 14]];
    assert!(
        center_density >= 5.0,
        "Center point should have at least 5 neighbors within r=1.5, got {}",
        center_density
    );

    // Clean up temporary files
    let _ = std::fs::remove_file(test_ply_path);
    let _ = std::fs::remove_file(test_npy_path);
}

