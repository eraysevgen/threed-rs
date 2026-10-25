use thiserror::Error;

/// Error types for feature extraction algorithms.
#[derive(Error, Debug, PartialEq)]
pub enum FeatureError {
    #[error("insufficient neighbors: minimum {required} required, got {found}")]
    InsufficientNeighbors { required: usize, found: usize },

    #[error("eigenvalue decomposition failed: {0}")]
    EigenDecompositionFailed(String),

    #[error("mismatched neighborhood indices and query points: query points {queries}, indices {indices}")]
    MismatchedLengths { queries: usize, indices: usize },
}

/// Result type alias for feature extraction.
pub type Result<T> = std::result::Result<T, FeatureError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_error_display() {
        let err = FeatureError::InsufficientNeighbors {
            required: 3,
            found: 2,
        };
        assert_eq!(
            err.to_string(),
            "insufficient neighbors: minimum 3 required, got 2"
        );
    }
}
