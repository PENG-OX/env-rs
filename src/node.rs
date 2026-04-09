//! Node.js version management
//!
//! Handles downloading, installing, and switching Node.js versions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Node.js version information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NodeVersion {
    pub version: String,
    pub npm_version: String,
    pub download_url: String,
}

/// Node.js release index (from nodejs.org)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReleaseIndex {
    pub version: String,
    pub files: Vec<String>,
    #[serde(default)]
    pub npm: Option<String>,
}

/// Node.js manager
pub struct NodeManager {
    install_dir: PathBuf,
}

impl NodeManager {
    pub fn new(install_dir: Option<PathBuf>) -> Self {
        let install_dir = install_dir.unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
                .join("env-switcher\\node")
        });
        Self { install_dir }
    }

    /// Get the installation directory
    pub fn install_dir(&self) -> &Path {
        &self.install_dir
    }

    /// Get the path for a specific version
    pub fn version_path(&self, version: &str) -> PathBuf {
        self.install_dir.join(version)
    }

    /// Check if a version is installed
    pub fn is_installed(&self, version: &str) -> bool {
        let node_exe = self.version_path(version).join("node.exe");
        node_exe.exists()
    }

    /// List installed versions
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let mut versions = Vec::new();

        if self.install_dir.exists() {
            for entry in fs::read_dir(&self.install_dir)
                .with_context(|| format!("Failed to read install dir: {:?}", self.install_dir))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if path.join("node.exe").exists() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            versions.push(name.to_string());
                        }
                    }
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Fetch available versions from nodejs.org
    pub async fn fetch_available_versions(&self) -> Result<Vec<NodeReleaseIndex>> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://nodejs.org/dist/index.json")
            .send()
            .await
            .with_context(|| "Failed to fetch Node.js release index")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to fetch Node.js versions: {}",
                response.status()
            );
        }

        let versions: Vec<NodeReleaseIndex> = response
            .json()
            .await
            .with_context(|| "Failed to parse Node.js release index")?;

        Ok(versions)
    }

    /// Install a Node.js version
    pub async fn install_version(&self, version: &str) -> Result<PathBuf> {
        if self.is_installed(version) {
            return Ok(self.version_path(version));
        }

        // Determine download URL
        let url = format!(
            "https://nodejs.org/dist/{}/node-{}-win-x64.zip",
            version, version
        );

        // Create install directory
        fs::create_dir_all(&self.install_dir)
            .with_context(|| format!("Failed to create install dir: {:?}", self.install_dir))?;

        let version_path = self.version_path(version);
        let temp_zip = self.install_dir.join(format!("node-{}.zip", version));

        // Download
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to download Node.js {}", version))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download Node.js {}: {}",
                version,
                response.status()
            );
        }

        // Save to temp file
        let bytes = response
            .bytes()
            .await
            .with_context(|| "Failed to read response body")?;

        fs::write(&temp_zip, &bytes)
            .with_context(|| format!("Failed to save downloaded file: {:?}", temp_zip))?;

        // Extract
        self.extract_zip(&temp_zip, &self.install_dir)?;

        // Clean up temp file
        let _ = fs::remove_file(&temp_zip);

        // Rename extracted folder (node-vX.Y.Z-win-x64 -> X.Y.Z)
        let extracted_name = format!("node-{}-win-x64", version);
        let extracted_path = self.install_dir.join(&extracted_name);
        if extracted_path.exists() {
            fs::rename(&extracted_path, &version_path)
                .with_context(|| format!("Failed to rename extracted folder: {:?}", extracted_path))?;
        }

        Ok(version_path)
    }

    /// Uninstall a Node.js version
    pub fn uninstall_version(&self, version: &str) -> Result<()> {
        let version_path = self.version_path(version);
        if !version_path.exists() {
            anyhow::bail!("Version {} is not installed", version);
        }

        fs::remove_dir_all(&version_path)
            .with_context(|| format!("Failed to remove version: {:?}", version_path))?;

        Ok(())
    }

    /// Get the current active version
    pub fn get_active_version() -> Result<Option<String>> {
        let output = Command::new("node")
            .arg("--version")
            .output()
            .context("Failed to execute node --version")?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_start_matches('v')
                .to_string();
            Ok(Some(version))
        } else {
            Ok(None)
        }
    }

    /// Extract a zip file
    fn extract_zip(&self, zip_path: &Path, dest_dir: &Path) -> Result<()> {
        let file = fs::File::open(zip_path)
            .with_context(|| format!("Failed to open zip file: {:?}", zip_path))?;

        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read zip archive: {:?}", zip_path))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out_path = dest_dir.join(file.mangled_name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&out_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    /// Verify a Node.js installation
    pub fn verify_installation(&self, version: &str) -> Result<bool> {
        let node_exe = self.version_path(version).join("node.exe");
        let _npm_cmd = self.version_path(version).join("npm.cmd");

        if !node_exe.exists() {
            return Ok(false);
        }

        // Test execution
        let output = Command::new(&node_exe)
            .arg("--version")
            .output()
            .context("Failed to execute installed node")?;

        if !output.status.success() {
            return Ok(false);
        }

        let version_output = String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_start_matches('v')
            .to_string();

        Ok(version_output.starts_with(version))
    }
}

// Use std::fs since we're not using tokio in this module
use std::fs;
