/// Legacy config module that wraps the unified_config module
///
/// This module is provided for backward compatibility with code that still uses the old config module.
/// New code should use the unified_config module directly.
use crate::unified_config::AppConfig;
use anyhow::Result;
use std::path::PathBuf;

/// Re-export constants from unified_config
pub use crate::unified_config::GITHUB_TOKEN_KEY;

/// Config struct that wraps AppConfig
#[derive(Debug, Clone)]
pub struct Config {
    /// The wrapped AppConfig
    pub app_config: AppConfig,
    /// Cache for options
    pub cache: im::HashMap<String, String>,
}

impl Config {
    /// Create a new Config with default values
    pub fn default() -> Self {
        Self {
            app_config: AppConfig::default(),
            cache: im::HashMap::new(),
        }
    }

    /// Set an option in the config
    pub fn with_option(mut self, key: &str, value: &str) -> Self {
        self.cache.insert(key.to_string(), value.to_string());
        self
    }

    /// Get the data path as a string
    pub fn data_path_str(&self) -> Result<String> {
        Ok(self.app_config.data_path.to_string_lossy().to_string())
    }

    /// Get the database path as a string
    pub fn db_path_str(&self) -> Result<String> {
        Ok(self.app_config.db_path.to_string_lossy().to_string())
    }

    /// Get the database path
    pub fn db_path(&self) -> PathBuf {
        self.app_config.db_path.clone()
    }
}
