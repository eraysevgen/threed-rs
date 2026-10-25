//! Feature extraction algorithms (covariance, PCA, eigenvalues, geometric, height, density).

pub mod error;
pub mod geometric;
pub mod height;
pub mod pca;
pub mod pipeline;

// Re-exports
pub use error::{FeatureError, Result};
pub use geometric::GeometricFeatures;
pub use height::HeightFeatures;
pub use pca::{compute_covariance_matrix, PcaResult};
pub use pipeline::{compute_features, FeaturePipeline};
