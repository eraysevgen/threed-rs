use thiserror::Error;

/// Error types for core point cloud structures and operations.
#[derive(Error, Debug, PartialEq)]
pub enum CoreError {
    #[error("point cloud is empty")]
    EmptyPointCloud,

    #[error("index out of bounds: requested {index}, but size is {len}")]
    IndexOutOfBounds { index: usize, len: usize },

    #[error("invalid dimensions: expected {expected}, found {found}")]
    InvalidDimensions { expected: usize, found: usize },

    #[error("insufficient points: needed at least {required}, but got {found}")]
    InsufficientPoints { required: usize, found: usize },
}

/// Standard Result type alias for threed-rs-core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_display() {
        let err = CoreError::IndexOutOfBounds { index: 10, len: 5 };
        assert_eq!(
            err.to_string(),
            "index out of bounds: requested 10, but size is 5"
        );

        let err2 = CoreError::EmptyPointCloud;
        assert_eq!(err2.to_string(), "point cloud is empty");
    }
}
