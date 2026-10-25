use serde::{Deserialize, Serialize};

/// Number of baseline geometric features derived from PCA / eigenvalues.
pub const NUM_GEOMETRIC_FEATURES: usize = 9;

/// Number of height features.
pub const NUM_HEIGHT_FEATURES: usize = 5;

/// Number of density features.
pub const NUM_DENSITY_FEATURES: usize = 1;

/// Number of spatial coordinates (X, Y, Z).
pub const NUM_XYZ_FEATURES: usize = 3;

/// Names of the 9 standard geometric features in order of extraction.
pub const GEOMETRIC_FEATURE_NAMES: [&str; NUM_GEOMETRIC_FEATURES] = [
    "sum_eigenvalues",
    "linearity",
    "planarity",
    "sphericity",
    "omnivariance",
    "anisotropy",
    "eigenentropy",
    "surface_variation",
    "verticality",
];

/// Names of the 5 height features in order of extraction.
pub const HEIGHT_FEATURE_NAMES: [&str; NUM_HEIGHT_FEATURES] = [
    "height_range",
    "height_above_min",
    "height_below_max",
    "height_mean",
    "height_variance",
];

/// Name of the density feature.
pub const DENSITY_FEATURE_NAMES: [&str; NUM_DENSITY_FEATURES] = ["density"];

/// Names of the XYZ coordinates.
pub const XYZ_FEATURE_NAMES: [&str; NUM_XYZ_FEATURES] = ["x", "y", "z"];

/// Configuration determining which feature groups to calculate and append.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Whether to compute and append 5 height features.
    pub add_height: bool,
    /// Whether to append 1 density feature (e.g. number of neighbors or distance).
    pub add_density: bool,
    /// Whether to append raw (X, Y, Z) coordinates at the end of each row.
    pub add_xyz: bool,
}

impl FeatureConfig {
    /// Computes total number of output feature columns according to the configuration.
    #[inline]
    pub fn total_features(&self) -> usize {
        let mut count = NUM_GEOMETRIC_FEATURES;
        if self.add_height {
            count += NUM_HEIGHT_FEATURES;
        }
        if self.add_density {
            count += NUM_DENSITY_FEATURES;
        }
        if self.add_xyz {
            count += NUM_XYZ_FEATURES;
        }
        count
    }

    /// Returns the complete list of column names for this configuration.
    pub fn column_names(&self) -> Vec<&'static str> {
        let mut names = Vec::with_capacity(self.total_features());
        names.extend_from_slice(&GEOMETRIC_FEATURE_NAMES);

        if self.add_height {
            names.extend_from_slice(&HEIGHT_FEATURE_NAMES);
        }
        if self.add_density {
            names.extend_from_slice(&DENSITY_FEATURE_NAMES);
        }
        if self.add_xyz {
            names.extend_from_slice(&XYZ_FEATURE_NAMES);
        }
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_config_counts() {
        let default_cfg = FeatureConfig::default();
        assert_eq!(default_cfg.total_features(), 9);
        assert_eq!(default_cfg.column_names().len(), 9);

        let full_cfg = FeatureConfig {
            add_height: true,
            add_density: true,
            add_xyz: true,
        };
        assert_eq!(full_cfg.total_features(), 18);
        let names = full_cfg.column_names();
        assert_eq!(names.len(), 18);
        assert_eq!(names[0], "sum_eigenvalues");
        assert_eq!(names[8], "verticality");
        assert_eq!(names[9], "height_range");
        assert_eq!(names[14], "density");
        assert_eq!(names[15], "x");
        assert_eq!(names[17], "z");
    }
}
