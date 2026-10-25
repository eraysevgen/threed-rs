//! Core data structures, point cloud representation, and feature metadata for threed-rs.

pub mod bbox;
pub mod cloud;
pub mod error;
pub mod features;
pub mod point;

// Re-exports for convenience
pub use bbox::BoundingBox;
pub use cloud::PointCloud;
pub use error::{CoreError, Result};
pub use features::{
    FeatureConfig, DENSITY_FEATURE_NAMES, GEOMETRIC_FEATURE_NAMES, HEIGHT_FEATURE_NAMES,
    NUM_DENSITY_FEATURES, NUM_GEOMETRIC_FEATURES, NUM_HEIGHT_FEATURES, NUM_XYZ_FEATURES,
    XYZ_FEATURE_NAMES,
};
pub use point::Point3D;
