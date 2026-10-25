use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

/// A 3-dimensional Cartesian point representation with double precision (f64).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    /// Creates a new `Point3D` with the given x, y, z coordinates.
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Converts the point into a fixed-size `[f64; 3]` array.
    #[inline]
    pub const fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    /// Computes the squared Euclidean distance to another point.
    #[inline]
    pub fn distance_squared(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// Computes the Euclidean distance to another point.
    #[inline]
    pub fn distance(&self, other: &Self) -> f64 {
        self.distance_squared(other).sqrt()
    }
}

impl From<[f64; 3]> for Point3D {
    #[inline]
    fn from(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl From<&[f64; 3]> for Point3D {
    #[inline]
    fn from(arr: &[f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl From<Point3D> for [f64; 3] {
    #[inline]
    fn from(p: Point3D) -> Self {
        [p.x, p.y, p.z]
    }
}

impl From<(f64, f64, f64)> for Point3D {
    #[inline]
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        Self::new(x, y, z)
    }
}

impl From<Point3D> for (f64, f64, f64) {
    #[inline]
    fn from(p: Point3D) -> Self {
        (p.x, p.y, p.z)
    }
}

impl From<nalgebra::Point3<f64>> for Point3D {
    #[inline]
    fn from(p: nalgebra::Point3<f64>) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

impl From<Point3D> for nalgebra::Point3<f64> {
    #[inline]
    fn from(p: Point3D) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

impl From<nalgebra::Vector3<f64>> for Point3D {
    #[inline]
    fn from(v: nalgebra::Vector3<f64>) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<Point3D> for nalgebra::Vector3<f64> {
    #[inline]
    fn from(p: Point3D) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

impl Index<usize> for Point3D {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Point3D index out of bounds: {index}"),
        }
    }
}

impl IndexMut<usize> for Point3D {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Point3D index out of bounds: {index}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation_and_indexing() {
        let mut p = Point3D::new(1.0, 2.5, -3.2);
        assert_eq!(p[0], 1.0);
        assert_eq!(p[1], 2.5);
        assert_eq!(p[2], -3.2);

        p[1] = 5.0;
        assert_eq!(p.y, 5.0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        assert_eq!(p1.distance_squared(&p2), 25.0);
        assert_eq!(p1.distance(&p2), 5.0);
    }

    #[test]
    fn test_conversions() {
        let arr = [1.0, 2.0, 3.0];
        let p: Point3D = arr.into();
        assert_eq!(p.to_array(), arr);

        let na_p: nalgebra::Point3<f64> = p.into();
        assert_eq!(na_p.x, 1.0);
        assert_eq!(na_p.y, 2.0);
        assert_eq!(na_p.z, 3.0);
    }
}
