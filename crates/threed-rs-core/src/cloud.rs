use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

use crate::bbox::BoundingBox;
use crate::point::Point3D;

/// Container representing a 3D point cloud with optional coordinate offsets.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PointCloud {
    pub points: Vec<Point3D>,
    pub offsets: [f64; 3],
}

impl PointCloud {
    /// Creates an empty point cloud.
    #[inline]
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            offsets: [0.0, 0.0, 0.0],
        }
    }

    /// Creates an empty point cloud with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            points: Vec::with_capacity(capacity),
            offsets: [0.0, 0.0, 0.0],
        }
    }

    /// Creates a point cloud from an existing vector of `Point3D`.
    #[inline]
    pub fn from_points(points: Vec<Point3D>) -> Self {
        Self {
            points,
            offsets: [0.0, 0.0, 0.0],
        }
    }

    /// Creates a point cloud from an existing slice of raw coordinate arrays `[f64; 3]`.
    pub fn from_slices(slices: &[[f64; 3]]) -> Self {
        let points = slices.iter().map(|&arr| Point3D::from(arr)).collect();
        Self {
            points,
            offsets: [0.0, 0.0, 0.0],
        }
    }

    /// Number of points in the point cloud.
    #[inline]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Checks if the point cloud contains no points.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Appends a point to the point cloud.
    #[inline]
    pub fn push(&mut self, point: Point3D) {
        self.points.push(point);
    }

    /// Clears the points and resets offsets.
    #[inline]
    pub fn clear(&mut self) {
        self.points.clear();
        self.offsets = [0.0, 0.0, 0.0];
    }

    /// Shrinks the capacity of the points vector to fit its current length.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.points.shrink_to_fit();
    }

    /// Returns the bounding box of the points, or `None` if empty.
    #[inline]
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        BoundingBox::from_points(&self.points)
    }

    /// Converts points into an owned vector of fixed-size coordinate arrays `[f64; 3]`.
    /// Useful for spatial search indices (such as Kiddo).
    pub fn to_arrays(&self) -> Vec<[f64; 3]> {
        self.points.iter().map(|p| p.to_array()).collect()
    }

    /// Returns a slice view of the underlying points.
    #[inline]
    pub fn as_slice(&self) -> &[Point3D] {
        &self.points
    }

    /// Returns a mutable slice view of the underlying points.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Point3D] {
        &mut self.points
    }

    /// Returns an iterator over the points.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Point3D> {
        self.points.iter()
    }

    /// Returns a mutable iterator over the points.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Point3D> {
        self.points.iter_mut()
    }

    /// Safely gets a reference to a point by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Point3D> {
        self.points.get(index)
    }
}

impl Index<usize> for PointCloud {
    type Output = Point3D;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

impl IndexMut<usize> for PointCloud {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.points[index]
    }
}

impl IntoIterator for PointCloud {
    type Item = Point3D;
    type IntoIter = std::vec::IntoIter<Point3D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}

impl<'a> IntoIterator for &'a PointCloud {
    type Item = &'a Point3D;
    type IntoIter = std::slice::Iter<'a, Point3D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_cloud_operations() {
        let mut cloud = PointCloud::with_capacity(10);
        assert!(cloud.is_empty());
        assert_eq!(cloud.len(), 0);

        cloud.push(Point3D::new(1.0, 2.0, 3.0));
        cloud.push(Point3D::new(4.0, 5.0, 6.0));

        assert_eq!(cloud.len(), 2);
        assert_eq!(cloud[0], Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(cloud[1], Point3D::new(4.0, 5.0, 6.0));

        let arrays = cloud.to_arrays();
        assert_eq!(arrays, vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);

        let bbox = cloud.bounding_box().expect("Should have bbox");
        assert_eq!(bbox.min, Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(bbox.max, Point3D::new(4.0, 5.0, 6.0));

        cloud.clear();
        assert!(cloud.is_empty());
    }
}
