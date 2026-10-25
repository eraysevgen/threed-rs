use thiserror::Error;

/// Error types for spatial search and KD-Tree operations.
#[derive(Error, Debug, PartialEq)]
pub enum SearchError {
    #[error("point cloud cannot be empty when constructing a spatial index")]
    EmptyPointCloud,

    #[error("failed to build spatial index: {0}")]
    TreeBuildError(String),

    #[error("invalid search parameter: {0}")]
    InvalidParameter(String),

    #[error("number of requested nearest neighbors (k) must be greater than zero")]
    ZeroNeighborsRequested,

    #[error("search radius must be strictly positive, got {0}")]
    InvalidRadius(f64),
}

/// Result alias for spatial search operations.
pub type Result<T> = std::result::Result<T, SearchError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SearchError::InvalidRadius(-1.0);
        assert_eq!(
            err.to_string(),
            "search radius must be strictly positive, got -1"
        );
    }
}
