use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use threed_rs_core::FeatureConfig;
use threed_rs_search::NeighborhoodMethod;

/// Complete application configuration compatible with C++ config.yaml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub files: FilesConfig,
    pub neighborhood: NeighborhoodConfig,
    #[serde(default)]
    pub features: FeaturesSettings,
    #[serde(default)]
    pub params: ParamsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    pub host_file: PathBuf,
    #[serde(default)]
    pub query_file: Option<PathBuf>,
    pub out_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodConfig {
    /// Search method name: "knn" or "radius"
    pub name: String,
    /// Engine: "cpu" (Rust kiddo)
    #[serde(default = "default_engine")]
    pub engine: String,
    /// Search radius for radius search
    #[serde(default = "default_radius")]
    pub radius: f64,
    /// Maximum number of neighbors for radius search
    #[serde(default = "default_max_k")]
    pub max_k: usize,
    /// Number of neighbors for KNN search
    #[serde(default = "default_k")]
    pub k: usize,
    /// Batch size for processing (kept for compatibility)
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_engine() -> String {
    "cpu".to_string()
}
fn default_radius() -> f64 {
    0.25
}
fn default_max_k() -> usize {
    512
}
fn default_k() -> usize {
    8
}
fn default_batch_size() -> usize {
    65536
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesSettings {
    #[serde(default)]
    pub add_height: bool,
    #[serde(default = "default_true")]
    pub add_density: bool,
    #[serde(default)]
    pub add_xyz: bool,
}

fn default_true() -> bool {
    true
}

impl Default for FeaturesSettings {
    fn default() -> Self {
        Self {
            add_height: false,
            add_density: true,
            add_xyz: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParamsConfig {
    /// Number of parallel threads (0 = auto / all logical cores)
    #[serde(default)]
    pub num_threads: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            files: FilesConfig {
                host_file: PathBuf::from("host.las"),
                query_file: None,
                out_file: PathBuf::from("output.npy"),
            },
            neighborhood: NeighborhoodConfig {
                name: "radius".to_string(),
                engine: "cpu".to_string(),
                radius: 0.25,
                max_k: 512,
                k: 8,
                batch_size: 65536,
            },
            features: FeaturesSettings::default(),
            params: ParamsConfig::default(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from a YAML file.
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        let config: AppConfig = serde_yaml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse YAML config from {}",
                path.as_ref().display()
            )
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Serializes configuration to a YAML string.
    pub fn to_yaml_string(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize config to YAML")
    }

    /// Validates configuration parameters.
    pub fn validate(&self) -> Result<()> {
        let name = self.neighborhood.name.to_lowercase();
        if name != "knn" && name != "radius" {
            bail!(
                "Neighborhood search method must be 'knn' or 'radius', got '{}'",
                self.neighborhood.name
            );
        }
        if name == "radius" && self.neighborhood.radius <= 0.0 {
            bail!(
                "Search radius must be positive, got {}",
                self.neighborhood.radius
            );
        }
        if name == "knn" && self.neighborhood.k == 0 {
            bail!("K in KNN search must be at least 1, got 0");
        }
        Ok(())
    }

    /// Converts neighborhood settings to `NeighborhoodMethod`.
    pub fn to_neighborhood_method(&self) -> Result<NeighborhoodMethod> {
        match self.neighborhood.name.to_lowercase().as_str() {
            "knn" => Ok(NeighborhoodMethod::Knn {
                k: self.neighborhood.k,
            }),
            "radius" => {
                let max_k = if self.neighborhood.max_k > 0 {
                    Some(self.neighborhood.max_k)
                } else {
                    None
                };
                Ok(NeighborhoodMethod::Radius {
                    radius: self.neighborhood.radius,
                    max_k,
                })
            }
            other => bail!("Unknown neighborhood method '{}'", other),
        }
    }

    /// Converts feature settings to `FeatureConfig`.
    pub fn to_feature_config(&self) -> FeatureConfig {
        FeatureConfig {
            add_height: self.features.add_height,
            add_density: self.features.add_density,
            add_xyz: self.features.add_xyz,
        }
    }

    /// Returns query file path if set, otherwise host file path.
    pub fn resolved_query_file(&self) -> &Path {
        self.files
            .query_file
            .as_deref()
            .unwrap_or(&self.files.host_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let cfg = AppConfig::default();
        assert!(cfg.validate().is_ok());
        let method = cfg.to_neighborhood_method().unwrap();
        match method {
            NeighborhoodMethod::Radius { radius, max_k } => {
                assert_eq!(radius, 0.25);
                assert_eq!(max_k, Some(512));
            }
            _ => panic!("Expected Radius method"),
        }
    }

    #[test]
    fn test_yaml_roundtrip() {
        let cfg = AppConfig::default();
        let yaml = cfg.to_yaml_string().unwrap();
        let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.neighborhood.name, "radius");
        assert!(parsed.features.add_density);
    }
}
