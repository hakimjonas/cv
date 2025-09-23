/// Lean, principled configuration system
///
/// This module provides a simplified, type-safe configuration system that:
/// - Uses typed paths with camino::Utf8PathBuf
/// - Removes deprecated/unused fields
/// - Has clear hierarchy and precedence
/// - Follows builder pattern for flexibility
/// - Eliminates string-based path conversions
use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

/// GitHub cache refresh strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheStrategy {
    /// Load from cache, refresh in background
    Lazy,
    /// Always refresh before use
    Eager,
    /// Use cache without refresh
    Offline,
}

impl Default for CacheStrategy {
    fn default() -> Self {
        Self::Lazy
    }
}

/// GitHub API rate limiting strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitStrategy {
    /// Exponential backoff and retry
    Backoff,
    /// Wait for rate limit reset
    Wait,
    /// Fail immediately on rate limit
    Error,
}

impl Default for RateLimitStrategy {
    fn default() -> Self {
        Self::Backoff
    }
}

/// Path configuration with typed paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    /// CV data JSON file
    pub data_file: Utf8PathBuf,
    /// Static assets directory
    pub static_dir: Utf8PathBuf,
    /// Output directory for generated files
    pub output_dir: Utf8PathBuf,
    /// SQLite database file
    pub database: Utf8PathBuf,
    /// GitHub cache file
    pub github_cache: Utf8PathBuf,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            data_file: "data/cv_data.json".into(),
            static_dir: "static".into(),
            output_dir: "dist".into(),
            database: "data/cv.db".into(),
            github_cache: "data/github_cache.json".into(),
        }
    }
}

impl PathConfig {
    /// Get HTML output path
    pub fn html_output(&self) -> Utf8PathBuf {
        self.output_dir.join("cv.html")
    }

    /// Get PDF output path
    pub fn pdf_output(&self) -> Utf8PathBuf {
        self.output_dir.join("cv.pdf")
    }

    /// Get temporary Typst file path
    pub fn typst_temp(&self) -> Utf8PathBuf {
        "temp_cv.typ".into()
    }
}

/// GitHub integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// API token for authenticated requests
    pub token: Option<String>,
    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
    /// Cache refresh strategy
    #[serde(default)]
    pub cache_strategy: CacheStrategy,
    /// Rate limiting strategy
    #[serde(default)]
    pub rate_limit_strategy: RateLimitStrategy,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            cache_ttl: default_cache_ttl(),
            cache_strategy: CacheStrategy::default(),
            rate_limit_strategy: RateLimitStrategy::default(),
        }
    }
}

impl GitHubConfig {
    /// Get cache TTL as Duration
    pub fn cache_duration(&self) -> Duration {
        Duration::from_secs(self.cache_ttl)
    }

    /// Check if authenticated requests are possible
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Primary port for the API server
    #[serde(default = "default_port")]
    pub port: u16,
    /// Maximum port to try if primary is unavailable
    #[serde(default = "default_max_port")]
    pub max_port: u16,
    /// Development mode flag
    #[serde(default)]
    pub dev_mode: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            max_port: default_max_port(),
            dev_mode: false,
        }
    }
}

/// Data filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    /// Comma-separated list of fields to include in public output
    #[serde(default = "default_public_fields")]
    pub public_fields: String,
    /// Comma-separated list of fields to store in database
    #[serde(default = "default_db_fields")]
    pub database_fields: String,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            public_fields: default_public_fields(),
            database_fields: default_db_fields(),
        }
    }
}

impl DataConfig {
    /// Get public fields as a vector
    pub fn public_fields_list(&self) -> Vec<String> {
        self.public_fields
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Get database fields as a vector
    pub fn database_fields_list(&self) -> Vec<String> {
        self.database_fields
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Check if a field should be publicly visible
    pub fn is_public_field(&self, field: &str) -> bool {
        self.public_fields_list().contains(&field.to_string())
    }

    /// Check if a field should be stored in database
    pub fn is_database_field(&self, field: &str) -> bool {
        self.database_fields_list().contains(&field.to_string())
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfiguration {
    /// Path-related configuration
    #[serde(default)]
    pub paths: PathConfig,
    /// GitHub integration settings
    #[serde(default)]
    pub github: GitHubConfig,
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    /// Data filtering configuration
    #[serde(default)]
    pub data: DataConfig,
}

impl AppConfiguration {
    /// Load configuration from all available sources
    ///
    /// Precedence (highest to lowest):
    /// 1. Environment variables with CV_ prefix
    /// 2. config.toml file
    /// 3. Default values
    pub fn load() -> Result<Self> {
        info!("Loading application configuration");

        let mut builder = Config::builder();

        // Start with defaults
        debug!("Loading default configuration values");

        // Add configuration file if it exists
        let config_file = Utf8Path::new("config.toml");
        if config_file.exists() {
            info!("Loading configuration from {}", config_file);
            builder = builder.add_source(File::from(config_file.as_std_path()));
        } else {
            info!("No config.toml found, using defaults and environment variables");
        }

        // Add environment variables with CV_ prefix
        builder = builder.add_source(
            Environment::with_prefix("CV")
                .separator("__")
                .try_parsing(true),
        );

        // Build and deserialize
        let config = builder.build().context("Failed to build configuration")?;

        let mut app_config: AppConfiguration = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        // Try to get GitHub token from environment if not set
        if app_config.github.token.is_none()
            && let Ok(token) = std::env::var("GITHUB_TOKEN")
            && !token.is_empty()
        {
            info!("Using GitHub token from GITHUB_TOKEN environment variable");
            app_config.github.token = Some(token);
        }

        info!("Configuration loaded successfully");
        debug!("Data file: {}", app_config.paths.data_file);
        debug!("Output directory: {}", app_config.paths.output_dir);
        debug!("Server port: {}", app_config.server.port);
        debug!(
            "GitHub authenticated: {}",
            app_config.github.is_authenticated()
        );

        Ok(app_config)
    }

    /// Create a builder for custom configuration
    pub fn builder() -> AppConfigurationBuilder {
        AppConfigurationBuilder::new()
    }

    /// Get all derived output paths
    pub fn output_paths(&self) -> OutputPaths {
        OutputPaths {
            html: self.paths.html_output(),
            pdf: self.paths.pdf_output(),
            typst_temp: self.paths.typst_temp(),
        }
    }
}

/// Derived output paths
#[derive(Debug, Clone)]
pub struct OutputPaths {
    pub html: Utf8PathBuf,
    pub pdf: Utf8PathBuf,
    pub typst_temp: Utf8PathBuf,
}

/// Builder for AppConfiguration
#[derive(Debug, Default)]
pub struct AppConfigurationBuilder {
    config: AppConfiguration,
}

impl AppConfigurationBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfiguration::default(),
        }
    }

    pub fn data_file(mut self, path: impl Into<Utf8PathBuf>) -> Self {
        self.config.paths.data_file = path.into();
        self
    }

    pub fn output_dir(mut self, path: impl Into<Utf8PathBuf>) -> Self {
        self.config.paths.output_dir = path.into();
        self
    }

    pub fn github_token(mut self, token: Option<String>) -> Self {
        self.config.github.token = token;
        self
    }

    pub fn server_port(mut self, port: u16) -> Self {
        self.config.server.port = port;
        self
    }

    pub fn dev_mode(mut self, enabled: bool) -> Self {
        self.config.server.dev_mode = enabled;
        self
    }

    pub fn build(self) -> AppConfiguration {
        self.config
    }
}

// Default value functions
fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_port() -> u16 {
    3000
}

fn default_max_port() -> u16 {
    3010
}

fn default_public_fields() -> String {
    "name,title,summary,experiences,education,skill_categories,projects,languages,certifications"
        .to_string()
}

fn default_db_fields() -> String {
    "personal_info,experiences,education,skill_categories,projects,languages,certifications,github_sources".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        let config = AppConfiguration::default();
        assert_eq!(config.paths.data_file, "data/cv_data.json");
        assert_eq!(config.paths.output_dir, "dist");
        assert_eq!(config.server.port, 3000);
        assert!(!config.github.is_authenticated());
    }

    #[test]
    fn test_builder_pattern() {
        let config = AppConfiguration::builder()
            .data_file("test/data.json")
            .output_dir("test/output")
            .server_port(8080)
            .dev_mode(true)
            .build();

        assert_eq!(config.paths.data_file, "test/data.json");
        assert_eq!(config.paths.output_dir, "test/output");
        assert_eq!(config.server.port, 8080);
        assert!(config.server.dev_mode);
    }

    #[test]
    fn test_output_paths() {
        let config = AppConfiguration::builder().output_dir("build").build();

        let paths = config.output_paths();
        assert_eq!(paths.html, "build/cv.html");
        assert_eq!(paths.pdf, "build/cv.pdf");
    }

    #[test]
    fn test_data_config_parsing() {
        let data_config = DataConfig {
            public_fields: "name, title, summary".to_string(),
            database_fields: "personal_info, experiences".to_string(),
        };

        let public_fields = data_config.public_fields_list();
        assert_eq!(public_fields, vec!["name", "title", "summary"]);

        assert!(data_config.is_public_field("name"));
        assert!(!data_config.is_public_field("private_field"));
    }

    #[test]
    fn test_github_config() {
        let mut github_config = GitHubConfig::default();
        assert!(!github_config.is_authenticated());

        github_config.token = Some("test_token".to_string());
        assert!(github_config.is_authenticated());

        assert_eq!(github_config.cache_duration(), Duration::from_secs(3600));
    }
}
