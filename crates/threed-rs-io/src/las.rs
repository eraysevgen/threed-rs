use las::{Builder, Point, Reader, Transform, Vector, Version, Writer};
use std::path::Path;
use threed_rs_core::{Point3D, PointCloud};

use crate::error::{IoError, Result};

/// Reads a LAS or LAZ point cloud file into a `PointCloud`.
pub fn read_las<P: AsRef<Path>>(path: P) -> Result<PointCloud> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(IoError::FileNotFound(path_ref.display().to_string()));
    }

    let mut reader = Reader::from_path(path_ref)?;
    let header = reader.header();
    let num_points = header.number_of_points() as usize;

    let offsets = [
        header.transforms().x.offset,
        header.transforms().y.offset,
        header.transforms().z.offset,
    ];

    let mut cloud = PointCloud::with_capacity(num_points);
    cloud.offsets = offsets;

    let point_data = reader.read_all()?;
    for pt in point_data.points() {
        let p = pt?;
        cloud.push(Point3D::new(p.x, p.y, p.z));
    }

    Ok(cloud)
}

/// Writes a `PointCloud` to a LAS or LAZ file.
pub fn write_las<P: AsRef<Path>>(path: P, cloud: &PointCloud) -> Result<()> {
    if cloud.is_empty() {
        return Err(IoError::EmptyPointCloud);
    }

    let mut builder = Builder::default();
    builder.version = Version::new(1, 4);
    builder.transforms = Vector {
        x: Transform {
            scale: 0.001,
            offset: cloud.offsets[0],
        },
        y: Transform {
            scale: 0.001,
            offset: cloud.offsets[1],
        },
        z: Transform {
            scale: 0.001,
            offset: cloud.offsets[2],
        },
    };

    let header = builder.into_header()?;
    let mut writer = Writer::from_path(path, header)?;

    for p in cloud.iter() {
        let las_point = Point {
            x: p.x,
            y: p.y,
            z: p.z,
            ..Default::default()
        };
        writer.write_point(las_point)?;
    }

    writer.close()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_las_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_roundtrip.las");

        let mut cloud = PointCloud::new();
        cloud.push(Point3D::new(10.5, 20.25, 30.125));
        cloud.push(Point3D::new(-5.0, 15.0, 25.0));

        write_las(&file_path, &cloud).expect("Failed to write LAS");
        let read_cloud = read_las(&file_path).expect("Failed to read LAS");

        assert_eq!(read_cloud.len(), 2);
        assert!((read_cloud[0].x - 10.5).abs() < 1e-3);
        assert!((read_cloud[0].y - 20.25).abs() < 1e-3);
        assert!((read_cloud[1].z - 25.0).abs() < 1e-3);
    }
}
