//! Point cloud file I/O operations (LAS, LAZ, PLY, NPY).

pub mod error;
pub mod las;
pub mod npy;
pub mod ply;

pub use error::{IoError, Result};
pub use las::{read_las, write_las};
pub use npy::{read_features_npy, write_features_npy, write_metadata_yaml};
pub use ply::{read_ply, write_ply};

use std::path::Path;
use threed_rs_core::PointCloud;

/// Reads a point cloud from the specified path, dispatching by file extension.
///
/// Supported formats: `.las`, `.laz`, `.ply`.
pub fn read_point_cloud<P: AsRef<Path>>(path: P) -> Result<PointCloud> {
    let p = path.as_ref();
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "las" | "laz" => read_las(p),
        "ply" => read_ply(p),
        _ => Err(IoError::UnsupportedFormat(p.display().to_string())),
    }
}

/// Writes a point cloud to the specified path, dispatching by file extension.
///
/// Supported formats: `.las`, `.laz`, `.ply` (defaults to binary PLY).
pub fn write_point_cloud<P: AsRef<Path>>(path: P, cloud: &PointCloud) -> Result<()> {
    let p = path.as_ref();
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "las" | "laz" => write_las(p, cloud),
        "ply" => write_ply(p, cloud, true),
        _ => Err(IoError::UnsupportedFormat(p.display().to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use threed_rs_core::Point3D;

    #[test]
    fn test_read_point_cloud_unsupported() {
        let err = read_point_cloud("sample.xyz").unwrap_err();
        match err {
            IoError::UnsupportedFormat(s) => assert!(s.contains("sample.xyz")),
            _ => panic!("Expected UnsupportedFormat error"),
        }
    }

    #[test]
    fn test_auto_dispatch_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let ply_path = temp_dir.join("test_auto_dispatch.ply");

        let mut cloud = PointCloud::new();
        cloud.push(Point3D::new(1.0, 2.0, 3.0));

        write_point_cloud(&ply_path, &cloud).expect("Failed to write point cloud");
        let read = read_point_cloud(&ply_path).expect("Failed to read point cloud");
        assert_eq!(read.len(), 1);
        assert!((read[0].x - 1.0).abs() < 1e-5);
    }
}
