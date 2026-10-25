use ply_rs::parser::Parser;
use ply_rs::ply::{
    Addable, ElementDef, Encoding, Ply, Property, PropertyAccess, PropertyDef, PropertyType,
    ScalarType, Version,
};
use ply_rs::writer::Writer;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use threed_rs_core::{Point3D, PointCloud};

use crate::error::{IoError, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlyPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PropertyAccess for PlyPoint {
    fn new() -> Self {
        PlyPoint {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_str(), property) {
            ("x", Property::Double(v)) => self.x = v,
            ("x", Property::Float(v)) => self.x = v as f64,
            ("y", Property::Double(v)) => self.y = v,
            ("y", Property::Float(v)) => self.y = v as f64,
            ("z", Property::Double(v)) => self.z = v,
            ("z", Property::Float(v)) => self.z = v as f64,
            _ => {}
        }
    }

    fn get_float(&self, key: &String) -> Option<f32> {
        match key.as_str() {
            "x" => Some(self.x as f32),
            "y" => Some(self.y as f32),
            "z" => Some(self.z as f32),
            _ => None,
        }
    }

    fn get_double(&self, key: &String) -> Option<f64> {
        match key.as_str() {
            "x" => Some(self.x),
            "y" => Some(self.y),
            "z" => Some(self.z),
            _ => None,
        }
    }
}

/// Reads a PLY file (ASCII or Binary) into a `PointCloud`.
pub fn read_ply<P: AsRef<Path>>(path: P) -> Result<PointCloud> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(IoError::FileNotFound(path_ref.display().to_string()));
    }

    let file = File::open(path_ref)?;
    let mut reader = BufReader::new(file);

    let parser = Parser::<PlyPoint>::new();
    let ply = parser
        .read_ply(&mut reader)
        .map_err(|e| IoError::PlyError(e.to_string()))?;

    let vertex_element = ply
        .payload
        .get("vertex")
        .ok_or_else(|| IoError::PlyError("Missing 'vertex' element in PLY file".to_string()))?;

    let mut cloud = PointCloud::with_capacity(vertex_element.len());
    for p in vertex_element {
        cloud.push(Point3D::new(p.x, p.y, p.z));
    }

    Ok(cloud)
}

/// Writes a `PointCloud` to an ASCII or Binary PLY file.
pub fn write_ply<P: AsRef<Path>>(path: P, cloud: &PointCloud, binary: bool) -> Result<()> {
    if cloud.is_empty() {
        return Err(IoError::EmptyPointCloud);
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let mut ply = Ply::<PlyPoint>::new();
    ply.header.version = Version { major: 1, minor: 0 };
    ply.header.encoding = if binary {
        Encoding::BinaryLittleEndian
    } else {
        Encoding::Ascii
    };

    let mut vertex_elem = ElementDef::new("vertex".to_string());
    vertex_elem.count = cloud.len();
    vertex_elem.properties.add(PropertyDef::new(
        "x".to_string(),
        PropertyType::Scalar(ScalarType::Double),
    ));
    vertex_elem.properties.add(PropertyDef::new(
        "y".to_string(),
        PropertyType::Scalar(ScalarType::Double),
    ));
    vertex_elem.properties.add(PropertyDef::new(
        "z".to_string(),
        PropertyType::Scalar(ScalarType::Double),
    ));

    ply.header.elements.add(vertex_elem);

    let points: Vec<PlyPoint> = cloud
        .iter()
        .map(|p| PlyPoint {
            x: p.x,
            y: p.y,
            z: p.z,
        })
        .collect();

    ply.payload.insert("vertex".to_string(), points);

    let ply_writer = Writer::new();
    ply_writer
        .write_ply(&mut writer, &mut ply)
        .map_err(|e| IoError::PlyError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ply_ascii_and_binary_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let ascii_path = temp_dir.join("test_roundtrip_ascii.ply");
        let bin_path = temp_dir.join("test_roundtrip_bin.ply");

        let mut cloud = PointCloud::new();
        cloud.push(Point3D::new(1.0, 2.0, 3.0));
        cloud.push(Point3D::new(4.5, 5.5, 6.5));

        // ASCII roundtrip
        write_ply(&ascii_path, &cloud, false).expect("Failed to write ASCII PLY");
        let read_ascii = read_ply(&ascii_path).expect("Failed to read ASCII PLY");
        assert_eq!(read_ascii.len(), 2);
        assert!((read_ascii[0].x - 1.0).abs() < 1e-5);
        assert!((read_ascii[1].z - 6.5).abs() < 1e-5);

        // Binary roundtrip
        write_ply(&bin_path, &cloud, true).expect("Failed to write binary PLY");
        let read_bin = read_ply(&bin_path).expect("Failed to read binary PLY");
        assert_eq!(read_bin.len(), 2);
        assert!((read_bin[0].x - 1.0).abs() < 1e-5);
        assert!((read_bin[1].z - 6.5).abs() < 1e-5);
    }
}
