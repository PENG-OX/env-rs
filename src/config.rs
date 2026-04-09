//! Configuration management for Env Switcher

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Node.js version installations: version -> installation path
    #[serde(default)]
    pub node_versions: HashMap<String, VersionConfig>,

    /// Java version installations: version -> installation path
    #[serde(default)]
    pub java_versions: HashMap<String, VersionConfig>,

    /// Path to environment mappings
    #[serde(default)]
    pub path_mappings: Vec<PathMapping>,

    /// Configuration file path (not serialized)
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

/// Version configuration for Node.js or Java
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    /// Installation path
    pub path: String,
    /// Version string (e.g., "18.18.0", "11", "17")
    pub version: String,
    /// Optional: download URL for auto-install
    #[serde(default)]
    pub download_url: Option<String>,
}

/// Path to environment mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathMapping {
    /// Project path (absolute)
    pub path: String,
    /// Node.js version to use (optional)
    #[serde(default)]
    pub node_version: Option<String>,
    /// Java version to use (optional)
    #[serde(default)]
    pub java_version: Option<String>,
}

impl Config {
    /// Get the default config directory path
    pub fn default_config_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
            .join("env-switcher")
    }

    /// Get the default config file path
    pub fn default_config_path() -> PathBuf {
        Self::default_config_dir().join("config.json")
    }

    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path();
        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            // Create default config if not exists
            let config = Config::default();
            config.save_to_path(path)?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let mut config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        config.config_path = Some(path.to_path_buf());
        Ok(config)
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        let config_path = self
            .config_path
            .clone()
            .unwrap_or_else(Self::default_config_path);
        self.save_to_path(&config_path)
    }

    /// Save configuration to a specific path
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    /// Add a path mapping
    pub fn add_path_mapping(&mut self, path: String, node_version: Option<String>, java_version: Option<String>) {
        // Remove existing mapping if exists
        self.path_mappings.retain(|m| m.path != path);

        self.path_mappings.push(PathMapping {
            path,
            node_version,
            java_version,
        });
    }

    /// Remove a path mapping
    pub fn remove_path_mapping(&mut self, path: &str) {
        self.path_mappings.retain(|m| m.path != path);
    }

    /// Add a Node.js version
    pub fn add_node_version(&mut self, version: String, path: String) {
        self.node_versions.insert(version.clone(), VersionConfig {
            path,
            version,
            download_url: None,
        });
    }

    /// Add a Java version
    pub fn add_java_version(&mut self, version: String, path: String) {
        self.java_versions.insert(version.clone(), VersionConfig {
            path,
            version,
            download_url: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.add_node_version("18.18.0".to_string(), "C:\\node\\18.18.0".to_string());
        config.add_java_version("11".to_string(), "C:\\jdk\\11".to_string());
        config.add_path_mapping(
            "C:\\projects\\web-app".to_string(),
            Some("18.18.0".to_string()),
            Some("11".to_string()),
        );

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.node_versions.len(), deserialized.node_versions.len());
        assert_eq!(config.java_versions.len(), deserialized.java_versions.len());
        assert_eq!(config.path_mappings.len(), deserialized.path_mappings.len());
    }

    #[test]
    fn test_config_persistence() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let mut config = Config::default();
        config.add_node_version("20.9.0".to_string(), "C:\\node\\20.9.0".to_string());
        config.save_to_path(&config_path).unwrap();

        assert!(config_path.exists());

        let loaded = Config::load_from_path(&config_path).unwrap();
        assert!(loaded.node_versions.contains_key("20.9.0"));
    }
}
