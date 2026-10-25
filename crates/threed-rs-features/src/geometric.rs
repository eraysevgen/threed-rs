use crate::pca::PcaResult;

/// 9 standard geometric features computed from eigenvalue and eigenvector decomposition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometricFeatures {
    /// Sum of eigenvalues: λ1 + λ2 + λ3
    pub sum_eigenvalues: f64,
    /// Linearity: (λ1 - λ2) / λ1
    pub linearity: f64,
    /// Planarity: (λ2 - λ3) / λ1
    pub planarity: f64,
    /// Sphericity: λ3 / λ1
    pub sphericity: f64,
    /// Omnivariance: (λ1 * λ2 * λ3)^(1/3)
    pub omnivariance: f64,
    /// Anisotropy: (λ1 - λ3) / λ1
    pub anisotropy: f64,
    /// Eigenentropy: -(λ1 * ln(λ1)) + (λ2 * ln(λ2)) + (λ3 * ln(λ3))
    pub eigenentropy: f64,
    /// Surface variation / Curvature: λ3 / (λ1 + λ2 + λ3)
    pub surface_variation: f64,
    /// Verticality: 1.0 - |z_axis · ev3|
    pub verticality: f64,
}

impl GeometricFeatures {
    /// Computes the 9 geometric features from a PCA decomposition result.
    ///
    /// Preserves exact numerical formulation from the original C++ reference implementation.
    pub fn from_pca(pca: &PcaResult) -> Self {
        let e1 = pca.e1;
        let e2 = pca.e2;
        let e3 = pca.e3;

        let sum_eigenvalues = e1 + e2 + e3;

        let (linearity, planarity, sphericity, anisotropy) = if e1 > 1e-15 {
            ((e1 - e2) / e1, (e2 - e3) / e1, e3 / e1, (e1 - e3) / e1)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        let omnivariance = (e1 * e2 * e3).cbrt();

        let safe_term = |val: f64| {
            if val > 1e-15 {
                val * val.ln()
            } else {
                0.0
            }
        };

        // Original threed C++ formula: -(e1*ln(e1)) + (e2*ln(e2)) + (e3*ln(e3))
        let eigenentropy = -safe_term(e1) + safe_term(e2) + safe_term(e3);

        let surface_variation = if sum_eigenvalues > 1e-15 {
            e3 / sum_eigenvalues
        } else {
            0.0
        };

        // Verticality: 1.0 - |ev3.z|
        let verticality = 1.0 - pca.ev3[2].abs();

        Self {
            sum_eigenvalues,
            linearity,
            planarity,
            sphericity,
            omnivariance,
            anisotropy,
            eigenentropy,
            surface_variation,
            verticality,
        }
    }

    /// Creates a NaN-filled feature struct for degenerate neighborhoods (< 3 points).
    pub fn nan() -> Self {
        Self {
            sum_eigenvalues: f64::NAN,
            linearity: f64::NAN,
            planarity: f64::NAN,
            sphericity: f64::NAN,
            omnivariance: f64::NAN,
            anisotropy: f64::NAN,
            eigenentropy: f64::NAN,
            surface_variation: f64::NAN,
            verticality: f64::NAN,
        }
    }

    /// Converts into a standard fixed-size array `[f64; 9]`.
    pub fn to_array(&self) -> [f64; 9] {
        [
            self.sum_eigenvalues,
            self.linearity,
            self.planarity,
            self.sphericity,
            self.omnivariance,
            self.anisotropy,
            self.eigenentropy,
            self.surface_variation,
            self.verticality,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_geometric_features() {
        let pca = PcaResult {
            e1: 3.0,
            e2: 2.0,
            e3: 1.0,
            ev1: Vector3::new(1.0, 0.0, 0.0),
            ev2: Vector3::new(0.0, 1.0, 0.0),
            ev3: Vector3::new(0.0, 0.0, 1.0),
        };

        let gf = GeometricFeatures::from_pca(&pca);
        assert!((gf.sum_eigenvalues - 6.0).abs() < 1e-9);
        assert!((gf.linearity - (1.0 / 3.0)).abs() < 1e-9);
        assert!((gf.planarity - (1.0 / 3.0)).abs() < 1e-9);
        assert!((gf.sphericity - (1.0 / 3.0)).abs() < 1e-9);
        assert!((gf.surface_variation - (1.0 / 6.0)).abs() < 1e-9);
        assert!((gf.verticality - 0.0).abs() < 1e-9); // Normal is vertical Z, verticality = 1 - 1 = 0
    }
}
