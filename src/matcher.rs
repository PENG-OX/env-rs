//! Path matching logic for environment detection
//!
//! Supports exact match and prefix inheritance (subdirectories inherit parent config)

use crate::config::{Config, PathMapping};
use std::path::Path;

/// Path matcher for finding environment configurations
pub struct PathMatcher {
    config: Config,
}

/// Result of path matching
#[derive(Debug, Clone, Default)]
pub struct MatchResult {
    /// Matched path (the configured path that matched)
    pub matched_path: Option<String>,
    /// Node.js version to use
    pub node_version: Option<String>,
    /// Node.js installation path
    pub node_path: Option<String>,
    /// Java version to use
    pub java_version: Option<String>,
    /// Java installation path (JAVA_HOME)
    pub java_home: Option<String>,
    /// Whether this is an inherited match (subdirectory)
    pub is_inherited: bool,
}

impl PathMatcher {
    /// Create a new path matcher with the given config
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Reload config from file
    pub fn reload_config(&mut self) -> anyhow::Result<()> {
        self.config = Config::load()?;
        Ok(())
    }

    /// Find matching environment for a given path
    ///
    /// Matching rules:
    /// 1. Exact match takes priority
    /// 2. Longest prefix match for subdirectory inheritance
    /// 3. No match returns empty result (use system environment)
    pub fn find_match(&self, target_path: &Path) -> MatchResult {
        let target_str = target_path.to_string_lossy();
        let target_normalized = Self::normalize_path(&target_str);

        let mut best_match: Option<&PathMapping> = None;
        let mut best_prefix_len = 0;
        let mut is_exact_match = false;

        for mapping in &self.config.path_mappings {
            let mapping_normalized = Self::normalize_path(&mapping.path);

            // Check for exact match
            if mapping_normalized == target_normalized {
                best_match = Some(mapping);
                is_exact_match = true;
                break;
            }

            // Check for prefix match (subdirectory inheritance)
            if target_normalized.starts_with(&mapping_normalized) {
                // Ensure it's a proper directory boundary
                // e.g., "C:\\projects\\app" should match "C:\\projects\\app\\src"
                // but "C:\\projects\\app2" should not match "C:\\projects\\app"
                let remaining = &target_normalized[mapping_normalized.len()..];
                if remaining.is_empty() || remaining.starts_with('\\') || remaining.starts_with('/') {
                    if mapping_normalized.len() > best_prefix_len {
                        best_prefix_len = mapping_normalized.len();
                        best_match = Some(mapping);
                    }
                }
            }
        }

        match best_match {
            Some(mapping) => self.build_result(mapping, !is_exact_match),
            None => MatchResult::default(),
        }
    }

    /// Build match result from a path mapping
    fn build_result(&self, mapping: &PathMapping, is_inherited: bool) -> MatchResult {
        let mut result = MatchResult {
            matched_path: Some(mapping.path.clone()),
            is_inherited,
            ..Default::default()
        };

        // Resolve Node.js version
        if let Some(ref version) = mapping.node_version {
            result.node_version = Some(version.clone());
            if let Some(config) = self.config.node_versions.get(version) {
                result.node_path = Some(config.path.clone());
            }
        }

        // Resolve Java version
        if let Some(ref version) = mapping.java_version {
            result.java_version = Some(version.clone());
            if let Some(config) = self.config.java_versions.get(version) {
                result.java_home = Some(config.path.clone());
            }
        }

        result
    }

    /// Normalize path separators
    fn normalize_path(path: &str) -> String {
        path.replace('/', "\\").to_lowercase()
    }

    /// Check if a path is within a configured directory (for status display)
    pub fn is_managed_path(&self, path: &Path) -> bool {
        !self.find_match(path).matched_path.is_none()
    }

    /// Get all configured paths
    pub fn get_configured_paths(&self) -> Vec<&PathMapping> {
        self.config.path_mappings.iter().collect()
    }
}

/// Global state for the path matcher (for use in PowerShell hook)
#[allow(dead_code)]
pub struct GlobalMatcher {
    matcher: PathMatcher,
    last_match: Option<(String, MatchResult)>,
}

#[allow(dead_code)]
impl GlobalMatcher {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            matcher: PathMatcher::new(config),
            last_match: None,
        })
    }

    pub fn find_or_cached(&mut self, path: &Path) -> &MatchResult {
        let path_str = path.to_string_lossy().to_string();

        // Return cached result if path unchanged
        if let Some((cached_path, _)) = &self.last_match {
            if cached_path == &path_str {
                return &self.last_match.as_ref().unwrap().1;
            }
        }

        let result = self.matcher.find_match(path);
        self.last_match = Some((path_str, result));

        &self.last_match.as_ref().unwrap().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.add_node_version("18.0.0".to_string(), "C:\\node\\18".to_string());
        config.add_node_version("20.0.0".to_string(), "C:\\node\\20".to_string());
        config.add_java_version("11".to_string(), "C:\\jdk\\11".to_string());
        config.add_java_version("17".to_string(), "C:\\jdk\\17".to_string());

        // Exact path
        config.add_path_mapping(
            "C:\\projects\\web-app".to_string(),
            Some("18.0.0".to_string()),
            Some("11".to_string()),
        );

        // Another path
        config.add_path_mapping(
            "C:\\projects\\backend".to_string(),
            Some("20.0.0".to_string()),
            Some("17".to_string()),
        );

        config
    }

    #[test]
    fn test_exact_match() {
        let config = create_test_config();
        let matcher = PathMatcher::new(config);

        let result = matcher.find_match(Path::new("C:\\projects\\web-app"));

        assert_eq!(result.matched_path, Some("C:\\projects\\web-app".to_string()));
        assert_eq!(result.node_version, Some("18.0.0".to_string()));
        assert_eq!(result.node_path, Some("C:\\node\\18".to_string()));
        assert_eq!(result.java_version, Some("11".to_string()));
        assert_eq!(result.java_home, Some("C:\\jdk\\11".to_string()));
        assert!(!result.is_inherited);
    }

    #[test]
    fn test_subdirectory_inheritance() {
        let config = create_test_config();
        let matcher = PathMatcher::new(config);

        // Subdirectory should inherit parent config
        let result = matcher.find_match(Path::new("C:\\projects\\web-app\\src\\components"));

        assert_eq!(result.matched_path, Some("C:\\projects\\web-app".to_string()));
        assert_eq!(result.node_version, Some("18.0.0".to_string()));
        assert!(result.is_inherited);
    }

    #[test]
    fn test_no_match() {
        let config = create_test_config();
        let matcher = PathMatcher::new(config);

        let result = matcher.find_match(Path::new("C:\\other\\project"));

        assert_eq!(result.matched_path, None);
        assert_eq!(result.node_version, None);
        assert_eq!(result.java_version, None);
    }

    #[test]
    fn test_similar_path_not_matched() {
        let config = create_test_config();
        let matcher = PathMatcher::new(config);

        // web-app2 should NOT match web-app config
        let result = matcher.find_match(Path::new("C:\\projects\\web-app2"));

        assert_eq!(result.matched_path, None);
    }

    #[test]
    fn test_longest_prefix_wins() {
        let mut config = Config::default();
        config.add_node_version("18.0.0".to_string(), "C:\\node\\18".to_string());

        // Parent directory
        config.add_path_mapping(
            "C:\\projects".to_string(),
            Some("18.0.0".to_string()),
            None,
        );

        // Child directory - should take priority for subdirectories
        config.add_path_mapping(
            "C:\\projects\\web-app".to_string(),
            Some("18.0.0".to_string()),
            None,
        );

        let matcher = PathMatcher::new(config);
        let result = matcher.find_match(Path::new("C:\\projects\\web-app\\src"));

        assert_eq!(result.matched_path, Some("C:\\projects\\web-app".to_string()));
    }
}
