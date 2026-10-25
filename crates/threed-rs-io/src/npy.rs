use ndarray::Array2;
use ndarray_npy::{read_npy, write_npy};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::{IoError, Result};

/// Writes a 2D feature matrix to a NumPy (.npy) file.
///
/// Each item in `features` is a row corresponding to a point's feature vector.
/// All rows must have identical column lengths.
pub fn write_features_npy<P: AsRef<Path>>(path: P, features: &[Vec<f64>]) -> Result<()> {
    if features.is_empty() {
        let empty_arr = Array2::<f64>::zeros((0, 0));
        write_npy(path, &empty_arr).map_err(|e| IoError::NpyError(e.to_string()))?;
        return Ok(());
    }

    let rows = features.len();
    let cols = features[0].len();

    let mut flat_vec = Vec::with_capacity(rows * cols);
    for (i, row) in features.iter().enumerate() {
        if row.len() != cols {
            return Err(IoError::ShapeMismatch {
                rows: i,
                cols: row.len(),
            });
        }
        flat_vec.extend_from_slice(row);
    }

    let array = Array2::from_shape_vec((rows, cols), flat_vec)
        .map_err(|e| IoError::NpyError(e.to_string()))?;

    write_npy(path, &array).map_err(|e| IoError::NpyError(e.to_string()))?;

    Ok(())
}

/// Reads a 2D feature matrix from a NumPy (.npy) file.
pub fn read_features_npy<P: AsRef<Path>>(path: P) -> Result<Array2<f64>> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(IoError::FileNotFound(path_ref.display().to_string()));
    }

    read_npy(path_ref).map_err(|e| IoError::NpyError(e.to_string()))
}

/// Writes metadata YAML content to the specified path.
pub fn write_metadata_yaml<P: AsRef<Path>>(path: P, yaml_content: &str) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(yaml_content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npy_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_features.npy");

        let features = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        write_features_npy(&file_path, &features).expect("Failed to write NPY");
        let arr = read_features_npy(&file_path).expect("Failed to read NPY");

        assert_eq!(arr.shape(), &[3, 3]);
        assert_eq!(arr[[0, 0]], 1.0);
        assert_eq!(arr[[1, 1]], 5.0);
        assert_eq!(arr[[2, 2]], 9.0);
    }

    #[test]
    fn test_npy_mismatched_rows() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_mismatched.npy");

        let features = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0], // uneven length
        ];

        let result = write_features_npy(&file_path, &features);
        assert!(result.is_err());
    }
}
