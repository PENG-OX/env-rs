//! PowerShell hook injection for automatic environment switching
//!
//! This module handles injecting and managing the PowerShell hook script
//! that auto-detects path changes and switches environment variables.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

use crate::config::Config;

/// PowerShell hook injector
pub struct HookInjector {
    /// Path to the hook script (stored separately from profile)
    hook_script_path: PathBuf,
}

/// Result of hook installation
#[derive(Debug)]
pub struct InstallResult {
    /// Whether the hook was newly installed
    pub newly_installed: bool,
    /// The profile path that was modified
    pub profile_path: PathBuf,
}

impl HookInjector {
    pub fn new() -> Self {
        let hook_script_path = Config::default_config_dir().join("hook.ps1");
        Self { hook_script_path }
    }

    /// Get the PowerShell profile path for current host
    pub fn get_profile_path() -> PathBuf {
        // PowerShell 7+ profile path
        let ps7_profile = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Public\\Documents"))
            .join("PowerShell\\7\\Microsoft.PowerShell_profile.ps1");

        // Check if PS7 profile exists, otherwise fall back to Windows PowerShell
        if ps7_profile.exists() {
            ps7_profile
        } else {
            // Windows PowerShell profile
            dirs::document_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\Users\\Public\\Documents"))
                .join("WindowsPowerShell\\Microsoft.PowerShell_profile.ps1")
        }
    }

    /// Generate the hook script content
    pub fn generate_hook_script(&self, config_path: &Path) -> String {
        format!(
            r#"# Env Switcher Hook - Auto-generated
# This script is automatically loaded by your PowerShell profile
# and handles environment variable switching based on current directory.

$EnvSwitcherConfigPath = "{config_path}"
$EnvSwitcherDataDir = "{data_dir}"

# Environment state tracking
$script:CurrentEnvPath = $null
$script:CurrentNodeHome = $null
$script:CurrentJavaHome = $null

# Cache file for match results (optional optimization)
$EnvSwitcherCacheFile = Join-Path $EnvSwitcherDataDir "cache.json"

function Find-PathMapping {{
    param(
        [string]$targetPath,
        [string]$configPath
    )

    if (-not (Test-Path $configPath)) {{
        return $null
    }}

    try {{
        $config = Get-Content $configPath -Raw | ConvertFrom-Json
        $targetNormalized = $targetPath.ToLower().Replace('/', '\')

        $bestMatch = $null
        $bestPrefixLen = 0

        foreach ($mapping in $config.path_mappings) {{
            $mappingNormalized = $mapping.path.ToLower().Replace('/', '\')

            # Exact match
            if ($mappingNormalized -eq $targetNormalized) {{
                $bestMatch = $mapping
                break
            }}

            # Prefix match (subdirectory inheritance)
            if ($targetNormalized.StartsWith($mappingNormalized + '\')) {{
                if ($mappingNormalized.Length -gt $bestPrefixLen) {{
                    $bestPrefixLen = $mappingNormalized.Length
                    $bestMatch = $mapping
                }}
            }}
        }}

        return $bestMatch
    }}
    catch {{
        Write-Warning "EnvSwitcher: Error reading config: $_"
        return $null
    }}
}}

function Switch-Environment {{
    param([string]$targetPath)

    # Skip if path unchanged
    if ($script:CurrentEnvPath -eq $targetPath) {{
        return
    }}

    # Read config
    $config = Get-Content $EnvSwitcherConfigPath -Raw | ConvertFrom-Json
    $mapping = Find-PathMapping -targetPath $targetPath -configPath $EnvSwitcherConfigPath

    if ($mapping) {{
        # Restore original PATH (save before first modification)
        if ($null -eq $script:OriginalPath) {{
            $script:OriginalPath = $env:PATH
        }}

        # Reset PATH to original
        $env:PATH = $script:OriginalPath

        # Apply Node.js environment
        if ($mapping.node_version) {{
            $nodeConfig = $config.node_versions.($mapping.node_version)
            if ($nodeConfig) {{
                $nodePath = $nodeConfig.path
                $env:PATH = "$nodePath;$env:PATH"
                $env:NODE_HOME = $nodePath
                $script:CurrentNodeHome = $nodePath

                # Set npm prefix if needed
                $npmPrefix = Join-Path $nodePath "npm"
                if (Test-Path $npmPrefix) {{
                    $env:npm_config_prefix = $npmPrefix
                }}

                Write-Host "Node: $($mapping.node_version)" -ForegroundColor Green
            }}
        }}
        else {{
            $env:NODE_HOME = $null
            $script:CurrentNodeHome = $null
        }}

        # Apply Java environment
        if ($mapping.java_version) {{
            $javaConfig = $config.java_versions.($mapping.java_version)
            if ($javaConfig) {{
                $javaHome = $javaConfig.path
                $env:JAVA_HOME = $javaHome
                $env:PATH = "$javaHome\bin;$env:PATH"
                $script:CurrentJavaHome = $javaHome

                Write-Host "Java: $($mapping.java_version)" -ForegroundColor Green
            }}
        }}
        else {{
            $env:JAVA_HOME = $null
            $script:CurrentJavaHome = $null
        }}

        $script:CurrentEnvPath = $targetPath
    }}
    else {{
        # No mapping found - restore original environment
        if ($null -ne $script:OriginalPath) {{
            $env:PATH = $script:OriginalPath
        }}
        $env:NODE_HOME = $null
        $env:JAVA_HOME = $null
        $script:CurrentEnvPath = $targetPath

        Write-Host "Using system environment" -ForegroundColor Gray
    }}
}}

# Hook into Set-Location (cd command)
# We wrap it rather than replace to preserve existing functionality
$wrappedCmd = Get-Command Set-Location -CommandType Cmdlet

function global:Set-Location {{
    param(
        [string]$Path,
        [switch]$PassThru,
        [string]$StackName
    )

    # Execute the actual cd command
    if ($Path) {{
        & $wrappedCmd -Path $Path -PassThru:$PassThru -StackName $StackName
    }}
    else {{
        & $wrappedCmd -PassThru:$PassThru -StackName $StackName
    }}

    # Switch environment based on new location
    if ($PWD) {{
        Switch-Environment -targetPath $PWD.Path
    }}
}}

# Also hook Push-Location and Pop-Location for pushd/popd
$wrappedPush = Get-Command Push-Location -CommandType Cmdlet
function global:Push-Location {{
    param(
        [string]$Path,
        [switch]$PassThru,
        [string]$StackName
    )

    if ($Path) {{
        & $wrappedPush -Path $Path -PassThru:$PassThru -StackName $StackName
    }}
    else {{
        & $wrappedPush -PassThru:$PassThru -StackName $StackName
    }}

    if ($PWD) {{
        Switch-Environment -targetPath $PWD.Path
    }}
}}

$wrappedPop = Get-Command Pop-Location -CommandType Cmdlet
function global:Pop-Location {{
    param(
        [string]$StackName
    )

    & $wrappedPop -StackName $StackName

    if ($PWD) {{
        Switch-Environment -targetPath $PWD.Path
    }}
}}

# Initial environment check on load
Write-Host "Env Switcher loaded. Current: " -ForegroundColor Gray -NoNewline
if ($PWD) {{
    Switch-Environment -targetPath $PWD.Path
}}

# Export helper function for debugging
function global:Get-EnvSwitcherStatus {{
    [PSCustomObject]@{{
        CurrentPath = $script:CurrentEnvPath
        NodeHome = $script:CurrentNodeHome
        JavaHome = $script:CurrentJavaHome
        ConfigPath = $EnvSwitcherConfigPath
    }}
}}
"#,
            config_path = config_path.to_string_lossy().replace('\\', "\\\\"),
            data_dir = Config::default_config_dir().to_string_lossy().replace('\\', "\\\\")
        )
    }

    /// Install the hook into PowerShell profile
    pub fn install(&self) -> Result<InstallResult> {
        let profile_path = Self::get_profile_path();
        let config_path = Config::default_config_path();

        // Ensure config directory exists
        if let Some(parent) = self.hook_script_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create hook directory: {:?}", parent))?;
        }

        // Write hook script
        let hook_content = self.generate_hook_script(&config_path);
        fs::write(&self.hook_script_path, &hook_content)
            .with_context(|| format!("Failed to write hook script: {:?}", self.hook_script_path))?;

        // Add sourcing line to profile if not already present
        let source_line = format!(". '{}'", self.hook_script_path.to_string_lossy());

        let profile_content = if profile_path.exists() {
            fs::read_to_string(&profile_path)
                .with_context(|| format!("Failed to read profile: {:?}", profile_path))?
        } else {
            String::new()
        };

        let new_profile_content = if profile_content.contains(&source_line) {
            // Hook already installed
            InstallResult {
                newly_installed: false,
                profile_path: profile_path.clone(),
            }
        } else {
            // Append hook sourcing
            let new_content = format!(
                "{}\n\n{}",
                profile_content,
                source_line
            );

            if let Some(parent) = profile_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create profile directory: {:?}", parent))?;
            }

            fs::write(&profile_path, &new_content)
                .with_context(|| format!("Failed to write profile: {:?}", profile_path))?;

            InstallResult {
                newly_installed: true,
                profile_path: profile_path.clone(),
            }
        };

        Ok(new_profile_content)
    }

    /// Remove the hook from PowerShell profile
    pub fn uninstall(&self) -> Result<()> {
        let profile_path = Self::get_profile_path();
        let source_line = format!(". '{}'", self.hook_script_path.to_string_lossy());

        if profile_path.exists() {
            let content = fs::read_to_string(&profile_path)
                .with_context(|| format!("Failed to read profile: {:?}", profile_path))?;

            // Remove the source line and any empty lines around it
            let new_content: Vec<&str> = content
                .lines()
                .filter(|line| {
                    !line.trim().starts_with(&source_line) &&
                    !line.trim().starts_with(". $PSScriptRoot") ||
                    line.trim().is_empty()
                })
                .collect();

            fs::write(&profile_path, new_content.join("\n"))
                .with_context(|| format!("Failed to write profile: {:?}", profile_path))?;
        }

        // Remove hook script file
        if self.hook_script_path.exists() {
            fs::remove_file(&self.hook_script_path)
                .with_context(|| format!("Failed to remove hook script: {:?}", self.hook_script_path))?;
        }

        Ok(())
    }

    /// Check if hook is installed
    pub fn is_installed(&self) -> bool {
        let profile_path = Self::get_profile_path();
        let source_line = format!(". '{}'", self.hook_script_path.to_string_lossy());

        if !profile_path.exists() {
            return false;
        }

        if let Ok(content) = fs::read_to_string(&profile_path) {
            return content.contains(&source_line);
        }

        false
    }
}
