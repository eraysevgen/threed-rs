use crate::point::Point3D;
use serde::{Deserialize, Serialize};

/// An axis-aligned 3D bounding box (AABB).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Point3D,
    pub max: Point3D,
}

impl BoundingBox {
    /// Creates a new bounding box with explicitly given min and max corners.
    #[inline]
    pub fn new(min: Point3D, max: Point3D) -> Self {
        Self { min, max }
    }

    /// Computes the tight bounding box encompassing a slice of points.
    /// Returns `None` if the slice is empty.
    pub fn from_points(points: &[Point3D]) -> Option<Self> {
        let first = points.first()?;
        let mut min = *first;
        let mut max = *first;

        for p in &points[1..] {
            if p.x < min.x {
                min.x = p.x;
            }
            if p.y < min.y {
                min.y = p.y;
            }
            if p.z < min.z {
                min.z = p.z;
            }

            if p.x > max.x {
                max.x = p.x;
            }
            if p.y > max.y {
                max.y = p.y;
            }
            if p.z > max.z {
                max.z = p.z;
            }
        }

        Some(Self { min, max })
    }

    /// Computes the extent (dimensions / size) along each axis: (max - min).
    #[inline]
    pub fn extent(&self) -> Point3D {
        Point3D::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    /// Computes the geometric center of the bounding box.
    #[inline]
    pub fn center(&self) -> Point3D {
        Point3D::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    /// Returns `true` if the point is within or on the boundaries of the box.
    #[inline]
    pub fn contains(&self, p: &Point3D) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    /// Expands the bounding box to enclose the given point.
    #[inline]
    pub fn expand(&mut self, p: &Point3D) {
        if p.x < self.min.x {
            self.min.x = p.x;
        }
        if p.y < self.min.y {
            self.min.y = p.y;
        }
        if p.z < self.min.z {
            self.min.z = p.z;
        }

        if p.x > self.max.x {
            self.max.x = p.x;
        }
        if p.y > self.max.y {
            self.max.y = p.y;
        }
        if p.z > self.max.z {
            self.max.z = p.z;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_computation() {
        let pts = vec![
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(-2.0, 5.0, 1.0),
            Point3D::new(4.0, -1.0, 6.0),
        ];

        let bbox = BoundingBox::from_points(&pts).expect("Should compute bbox");
        assert_eq!(bbox.min, Point3D::new(-2.0, -1.0, 1.0));
        assert_eq!(bbox.max, Point3D::new(4.0, 5.0, 6.0));
        assert_eq!(bbox.extent(), Point3D::new(6.0, 6.0, 5.0));
        assert_eq!(bbox.center(), Point3D::new(1.0, 2.0, 3.5));
        assert!(bbox.contains(&Point3D::new(0.0, 0.0, 2.0)));
        assert!(!bbox.contains(&Point3D::new(10.0, 0.0, 2.0)));
    }

    #[test]
    fn test_empty_points_bbox() {
        assert_eq!(BoundingBox::from_points(&[]), None);
    }
}

