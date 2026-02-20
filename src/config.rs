use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thresholds {
    pub warning: f32,
    pub critical: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            warning: 70.0,
            critical: 90.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub cpu_thresholds: Thresholds,
    pub ram_thresholds: Thresholds,
    pub storage_thresholds: Thresholds,
}

impl Config {
    pub fn load_from_file(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: PathBuf) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}
