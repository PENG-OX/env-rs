//! Java JDK version management
//!
//! Handles installing and switching Java/JDK versions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Java version information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JavaVersion {
    pub version: String,
    pub vendor: String,
    pub install_path: PathBuf,
}

/// Java manager
pub struct JavaManager {
    install_dir: PathBuf,
}

impl JavaManager {
    pub fn new(install_dir: Option<PathBuf>) -> Self {
        let install_dir = install_dir.unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
                .join("env-switcher\\java")
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
        let java_exe = self.version_path(version).join("bin\\java.exe");
        java_exe.exists()
    }

    /// List installed versions
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let mut versions = Vec::new();

        if self.install_dir.exists() {
            for entry in std::fs::read_dir(&self.install_dir)
                .with_context(|| format!("Failed to read install dir: {:?}", self.install_dir))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if path.join("bin\\java.exe").exists() {
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

    /// Register an existing JDK installation
    pub fn register_existing_installation(&mut self, version: &str, path: &Path) -> Result<()> {
        if !path.join("bin\\java.exe").exists() {
            anyhow::bail!("Invalid JDK path: java.exe not found");
        }

        let target_path = self.version_path(version);

        // Create symlink or copy
        if target_path.exists() {
            std::fs::remove_dir_all(&target_path)?;
        }

        // For Windows, we'll copy the directory
        self.copy_dir(path, &target_path)?;

        Ok(())
    }

    /// Copy a directory recursively
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                self.copy_dir(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Get the current active Java version
    pub fn get_active_version() -> Result<Option<String>> {
        let output = Command::new("java")
            .arg("-version")
            .output()
            .context("Failed to execute java -version")?;

        if output.status.success() {
            // Parse version from stderr (java -version outputs to stderr)
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version = Self::parse_java_version(&stderr);
            Ok(Some(version))
        } else {
            Ok(None)
        }
    }

    /// Parse Java version string
    fn parse_java_version(output: &str) -> String {
        // Example: openjdk version "17.0.1" 2021-10-19
        for line in output.lines() {
            if line.contains("version") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return line[start + 1..start + 1 + end].to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    /// Verify a Java installation
    pub fn verify_installation(&self, version: &str) -> Result<bool> {
        let java_exe = self.version_path(version).join("bin\\java.exe");

        if !java_exe.exists() {
            return Ok(false);
        }

        // Test execution
        let output = Command::new(&java_exe)
            .arg("-version")
            .output()
            .context("Failed to execute installed java");

        match output {
            Ok(out) => {
                if !out.status.success() {
                    return Ok(false);
                }

                let stderr = String::from_utf8_lossy(&out.stderr);
                let version_output = Self::parse_java_version(&stderr);

                Ok(version_output.starts_with(version))
            }
            Err(_) => Ok(false),
        }
    }

    /// Uninstall a Java version
    pub fn uninstall_version(&self, version: &str) -> Result<()> {
        let version_path = self.version_path(version);
        if !version_path.exists() {
            anyhow::bail!("Version {} is not installed", version);
        }

        std::fs::remove_dir_all(&version_path)
            .with_context(|| format!("Failed to remove version: {:?}", version_path))?;

        Ok(())
    }

    /// Get available JDK downloads (Adoptium/Temurin)
    pub async fn fetch_available_versions(&self) -> Result<Vec<String>> {
        // Using Adoptium API
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.adoptium.net/v3/info/available_releases")
            .send()
            .await
            .with_context(|| "Failed to fetch Java releases")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch Java versions: {}", response.status());
        }

        // Parse response (simplified - actual API returns more complex structure)
        let versions: Vec<String> = response
            .json()
            .await
            .with_context(|| "Failed to parse Java release index")?;

        Ok(versions)
    }

    /// Download and install a Java version (Adoptium Temurin)
    pub async fn install_version(&self, version: &str) -> Result<PathBuf> {
        if self.is_installed(version) {
            return Ok(self.version_path(version));
        }

        // Build download URL for Temurin
        // Example: https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.1%2B12/OpenJDK17U-jdk_x64_windows_hotspot_17.0.1_12.zip
        let _major_version = version.split('.').next().unwrap_or(version);
        let url = format!(
            "https://api.adoptium.net/v3/binary/version/jdk-{}/windows/x64/jdk/hotspot/normal/eclipse",
            version
        );

        // Create install directory
        std::fs::create_dir_all(&self.install_dir)
            .with_context(|| format!("Failed to create install dir: {:?}", self.install_dir))?;

        let version_path = self.version_path(version);
        let temp_archive = self.install_dir.join(format!("jdk-{}.zip", version));

        // Download
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to download JDK {}", version))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download JDK {}: {}",
                version,
                response.status()
            );
        }

        // Download in chunks for large files
        let bytes = response
            .bytes()
            .await
            .with_context(|| "Failed to read response body")?;

        std::fs::write(&temp_archive, &bytes)
            .with_context(|| format!("Failed to save downloaded file: {:?}", temp_archive))?;

        // Extract
        self.extract_zip(&temp_archive, &self.install_dir)?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_archive);

        // Find and rename extracted folder
        for entry in std::fs::read_dir(&self.install_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.contains("jdk") && name.contains(version) {
                    std::fs::rename(&path, &version_path)?;
                    break;
                }
            }
        }

        Ok(version_path)
    }

    /// Extract a zip file
    fn extract_zip(&self, zip_path: &Path, dest_dir: &Path) -> Result<()> {
        let file = std::fs::File::open(zip_path)
            .with_context(|| format!("Failed to open zip file: {:?}", zip_path))?;

        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read zip archive: {:?}", zip_path))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out_path = dest_dir.join(file.mangled_name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&out_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}
