use nalgebra::{Matrix3, SymmetricEigen, Vector3};
use threed_rs_core::Point3D;

use crate::error::{FeatureError, Result};

/// Computes the 3x3 sample covariance matrix of a neighborhood of 3D points.
///
/// Formula:
///   `C = 1 / (N - 1) * sum((p_i - mean) * (p_i - mean)^T)`
///
/// Returns a zero matrix if `points.len() < 2`.
pub fn compute_covariance_matrix(points: &[Point3D]) -> Matrix3<f64> {
    let n = points.len();
    if n < 2 {
        return Matrix3::zeros();
    }

    // 1. Calculate centroid / mean
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_z = 0.0;

    for p in points {
        sum_x += p.x;
        sum_y += p.y;
        sum_z += p.z;
    }

    let inv_n = 1.0 / (n as f64);
    let mean_x = sum_x * inv_n;
    let mean_y = sum_y * inv_n;
    let mean_z = sum_z * inv_n;

    // 2. Accumulate upper triangular covariance terms
    let mut c00 = 0.0;
    let mut c01 = 0.0;
    let mut c02 = 0.0;
    let mut c11 = 0.0;
    let mut c12 = 0.0;
    let mut c22 = 0.0;

    for p in points {
        let dx = p.x - mean_x;
        let dy = p.y - mean_y;
        let dz = p.z - mean_z;

        c00 += dx * dx;
        c01 += dx * dy;
        c02 += dx * dz;
        c11 += dy * dy;
        c12 += dy * dz;
        c22 += dz * dz;
    }

    let inv_n_minus_1 = 1.0 / ((n - 1) as f64);

    Matrix3::new(
        c00 * inv_n_minus_1,
        c01 * inv_n_minus_1,
        c02 * inv_n_minus_1,
        c01 * inv_n_minus_1,
        c11 * inv_n_minus_1,
        c12 * inv_n_minus_1,
        c02 * inv_n_minus_1,
        c12 * inv_n_minus_1,
        c22 * inv_n_minus_1,
    )
}

/// Principal Component Analysis (PCA) / Eigen decomposition representation.
///
/// Holds eigenvalues sorted in descending order: `e1 >= e2 >= e3 >= 0`,
/// along with their corresponding unit eigenvectors `v1, v2, v3`.
#[derive(Debug, Clone, PartialEq)]
pub struct PcaResult {
    /// Largest eigenvalue (λ1)
    pub e1: f64,
    /// Medium eigenvalue (λ2)
    pub e2: f64,
    /// Smallest eigenvalue (λ3)
    pub e3: f64,
    /// Eigenvector corresponding to λ1 (principal direction)
    pub ev1: Vector3<f64>,
    /// Eigenvector corresponding to λ2
    pub ev2: Vector3<f64>,
    /// Eigenvector corresponding to λ3 (surface normal)
    pub ev3: Vector3<f64>,
}

impl PcaResult {
    /// Computes PCA decomposition of a 3x3 symmetric covariance matrix.
    ///
    /// Eigenvalues and eigenvectors are sorted such that `e1 >= e2 >= e3`.
    pub fn from_covariance(cov: &Matrix3<f64>) -> Result<Self> {
        let eigen = SymmetricEigen::new(*cov);

        // nalgebra returns eigenvalues unsorted or ascending.
        // Pair each eigenvalue with its eigenvector column:
        let mut pairs = [
            (
                eigen.eigenvalues[0],
                eigen.eigenvectors.column(0).into_owned(),
            ),
            (
                eigen.eigenvalues[1],
                eigen.eigenvectors.column(1).into_owned(),
            ),
            (
                eigen.eigenvalues[2],
                eigen.eigenvectors.column(2).into_owned(),
            ),
        ];

        // Sort descending by eigenvalue
        pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Numerical clamp against tiny negative precision noise
        let e1 = pairs[0].0.max(0.0);
        let e2 = pairs[1].0.max(0.0);
        let e3 = pairs[2].0.max(0.0);

        if e1.is_nan() || e2.is_nan() || e3.is_nan() {
            return Err(FeatureError::EigenDecompositionFailed(
                "NaN detected in eigenvalues".into(),
            ));
        }

        Ok(Self {
            e1,
            e2,
            e3,
            ev1: pairs[0].1,
            ev2: pairs[1].1,
            ev3: pairs[2].1,
        })
    }

    /// Computes PCA directly from a neighborhood slice of points.
    pub fn from_points(points: &[Point3D]) -> Result<Self> {
        if points.len() < 3 {
            return Err(FeatureError::InsufficientNeighbors {
                required: 3,
                found: points.len(),
            });
        }

        let cov = compute_covariance_matrix(points);
        Self::from_covariance(&cov)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covariance_and_pca_flat_surface() {
        // Planar points lying entirely on the XY plane (z = 0)
        let pts = vec![
            Point3D::new(-1.0, -1.0, 0.0),
            Point3D::new(1.0, -1.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(-1.0, 1.0, 0.0),
        ];

        let cov = compute_covariance_matrix(&pts);
        assert!((cov[(2, 2)]).abs() < 1e-12);

        let pca = PcaResult::from_covariance(&cov).unwrap();
        assert!(pca.e1 >= pca.e2);
        assert!(pca.e2 >= pca.e3);
        assert!(pca.e3.abs() < 1e-10); // Normal eigenvalue is 0 for flat 2D

        // The normal vector (ev3) should align with the Z axis (0, 0, 1 or 0, 0, -1)
        assert!((pca.ev3[2].abs() - 1.0).abs() < 1e-6);
    }
}
