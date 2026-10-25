use thiserror::Error;

/// Error types for point cloud file I/O operations.
#[derive(Error, Debug)]
pub enum IoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("LAS error: {0}")]
    Las(#[from] las::Error),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("unsupported file format or extension: {0}")]
    UnsupportedFormat(String),

    #[error("PLY parsing/formatting error: {0}")]
    PlyError(String),

    #[error("NumPy (.npy) export error: {0}")]
    NpyError(String),

    #[error("empty point cloud cannot be written")]
    EmptyPointCloud,

    #[error("matrix shape mismatch: expected {rows} rows with {cols} cols")]
    ShapeMismatch { rows: usize, cols: usize },
}

/// Result alias for point cloud file I/O operations.
pub type Result<T> = std::result::Result<T, IoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let err = IoError::UnsupportedFormat("xyz".into());
        assert_eq!(err.to_string(), "unsupported file format or extension: xyz");
    }
}
