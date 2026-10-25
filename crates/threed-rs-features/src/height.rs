use threed_rs_core::Point3D;

/// 5 height / elevation features computed from local neighborhood Z coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeightFeatures {
    /// Height range: z_max - z_min
    pub height_range: f64,
    /// Height above min: z_query - z_min
    pub height_above_min: f64,
    /// Height below max: z_max - z_query
    pub height_below_max: f64,
    /// Average height: mean(z)
    pub mean: f64,
    /// Sample variance: s^2 = 1 / (N - 1) * sum((z_i - mean)^2)
    pub variance: f64,
}

impl HeightFeatures {
    /// Computes the 5 height features from query point height and neighbor points.
    pub fn compute(query_z: f64, neighbors: &[Point3D]) -> Self {
        let n = neighbors.len();
        if n == 0 {
            return Self::nan();
        }

        let mut min_z = neighbors[0].z;
        let mut max_z = neighbors[0].z;
        let mut sum_z = 0.0;

        for p in neighbors {
            if p.z < min_z {
                min_z = p.z;
            }
            if p.z > max_z {
                max_z = p.z;
            }
            sum_z += p.z;
        }

        let mean = sum_z / (n as f64);

        let variance = if n > 1 {
            let mut sum_sq_diff = 0.0;
            for p in neighbors {
                let diff = p.z - mean;
                sum_sq_diff += diff * diff;
            }
            sum_sq_diff / ((n - 1) as f64)
        } else {
            0.0
        };

        Self {
            height_range: max_z - min_z,
            height_above_min: query_z - min_z,
            height_below_max: max_z - query_z,
            mean,
            variance,
        }
    }

    /// Creates a NaN-filled feature struct for empty neighborhoods.
    pub fn nan() -> Self {
        Self {
            height_range: f64::NAN,
            height_above_min: f64::NAN,
            height_below_max: f64::NAN,
            mean: f64::NAN,
            variance: f64::NAN,
        }
    }

    /// Converts into a standard fixed-size array `[f64; 5]`.
    pub fn to_array(&self) -> [f64; 5] {
        [
            self.height_range,
            self.height_above_min,
            self.height_below_max,
            self.mean,
            self.variance,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_height_features() {
        let query_z = 5.0;
        let neighbors = vec![
            Point3D::new(0.0, 0.0, 2.0),
            Point3D::new(0.0, 0.0, 5.0),
            Point3D::new(0.0, 0.0, 8.0),
        ];

        let hf = HeightFeatures::compute(query_z, &neighbors);
        assert_eq!(hf.height_range, 6.0);
        assert_eq!(hf.height_above_min, 3.0);
        assert_eq!(hf.height_below_max, 3.0);
        assert_eq!(hf.mean, 5.0);
        assert_eq!(hf.variance, 9.0);
    }
}
