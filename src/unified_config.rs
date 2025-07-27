/// Unified configuration module for the CV generator
///
/// This module provides a centralized place for all configuration settings
/// used throughout the application, using the `config` crate to load settings
/// from multiple sources (files, environment variables, command-line arguments).
use anyhow::{Context, Result};
use config::{Config, Environment, File};
use im::{HashMap, Vector};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Configuration key for the GitHub API token
#[allow(dead_code)]
pub const GITHUB_TOKEN_KEY: &str = "github_token";

/// Configuration key for the GitHub cache file
pub const GITHUB_CACHE_KEY: &str = "github_cache";

/// Default path for the GitHub cache file
pub const DEFAULT_GITHUB_CACHE_PATH: &str = "data/github_cache.json";

/// Configuration key for the GitHub cache TTL (Time To Live) in seconds
#[allow(dead_code)]
pub const GITHUB_CACHE_TTL_KEY: &str = "github_cache_ttl";

/// Default TTL for GitHub cache in seconds (1 hour)
pub const DEFAULT_GITHUB_CACHE_TTL: u64 = 3600;

/// Configuration key for the GitHub cache refresh strategy
#[allow(dead_code)]
pub const GITHUB_CACHE_REFRESH_STRATEGY_KEY: &str = "github_cache_refresh_strategy";

/// Default refresh strategy for GitHub cache
pub const DEFAULT_GITHUB_CACHE_REFRESH_STRATEGY: &str = "lazy";

/// Configuration key for the GitHub API rate limit handling strategy
#[allow(dead_code)]
pub const GITHUB_RATE_LIMIT_STRATEGY_KEY: &str = "github_rate_limit_strategy";

/// Default strategy for handling GitHub API rate limits
pub const DEFAULT_GITHUB_RATE_LIMIT_STRATEGY: &str = "backoff";

/// Configuration key for the GitHub OAuth client ID
pub const GITHUB_OAUTH_CLIENT_ID_KEY: &str = "github_oauth_client_id";

/// Configuration key for the GitHub OAuth client secret
pub const GITHUB_OAUTH_CLIENT_SECRET_KEY: &str = "github_oauth_client_secret";

/// Configuration key for the GitHub OAuth redirect URL
pub const GITHUB_OAUTH_REDIRECT_URL_KEY: &str = "github_oauth_redirect_url";

/// Default GitHub OAuth redirect URL
pub const DEFAULT_GITHUB_OAUTH_REDIRECT_URL: &str =
    "http://localhost:3002/api/auth/github/callback";

/// Default path for the SQLite database file
pub const DEFAULT_DB_PATH: &str = "data/cv.db";

/// Configuration key for the database path
pub const DB_PATH_KEY: &str = "db_path";

/// Configuration key for controlling what data is publicly visible
pub const PUBLIC_DATA_KEY: &str = "public_data";

/// Configuration key for controlling what data is stored in the database
pub const DB_STORAGE_KEY: &str = "db_storage";

/// Default public data settings (comma-separated list of fields)
pub const DEFAULT_PUBLIC_DATA: &str =
    "name,title,summary,experiences,education,skill_categories,projects,languages,certifications";

/// Default database storage settings (comma-separated list of fields)
pub const DEFAULT_DB_STORAGE: &str = "personal_info,experiences,education,skill_categories,projects,languages,certifications,github_sources";

/// Default port for the blog API server
pub const DEFAULT_API_PORT: u16 = 3000;

/// Default maximum port for the blog API server
pub const DEFAULT_API_MAX_PORT: u16 = 3010;

/// Represents the application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Path to the CV data JSON file
    #[serde(default = "default_data_path")]
    pub data_path: PathBuf,

    /// Directory containing static assets
    #[serde(default = "default_static_dir")]
    pub static_dir: PathBuf,

    /// Directory where output files will be written
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Path where the HTML CV will be written
    #[serde(skip)]
    pub html_output: PathBuf,

    /// Temporary file for Typst markup
    #[serde(default = "default_typst_temp")]
    pub typst_temp: PathBuf,

    /// Path where the PDF CV will be written
    #[serde(skip)]
    pub pdf_output: PathBuf,

    /// Path to the GitHub cache file
    #[serde(default = "default_github_cache_path")]
    pub github_cache_path: PathBuf,

    /// Path to the SQLite database file
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    /// GitHub API token
    #[serde(default)]
    pub github_token: Option<String>,

    /// Time To Live for GitHub cache in seconds
    #[serde(default = "default_github_cache_ttl")]
    pub github_cache_ttl: u64,

    /// Refresh strategy for GitHub cache ("lazy", "background", "eager")
    #[serde(default = "default_github_cache_refresh_strategy")]
    pub github_cache_refresh_strategy: String,

    /// Strategy for handling GitHub API rate limits ("backoff", "wait", "error")
    #[serde(default = "default_github_rate_limit_strategy")]
    pub github_rate_limit_strategy: String,

    /// GitHub OAuth client ID
    #[serde(default)]
    pub github_oauth_client_id: Option<String>,

    /// GitHub OAuth client secret
    #[serde(default)]
    pub github_oauth_client_secret: Option<String>,

    /// GitHub OAuth redirect URL
    #[serde(default = "default_github_oauth_redirect_url")]
    pub github_oauth_redirect_url: String,

    /// Fields that should be publicly visible (comma-separated)
    #[serde(default = "default_public_data")]
    pub public_data: String,

    /// Fields that should be stored in the database (comma-separated)
    #[serde(default = "default_db_storage")]
    pub db_storage: String,

    /// Port for the blog API server
    #[serde(default = "default_api_port")]
    pub api_port: u16,

    /// Maximum port for the blog API server
    #[serde(default = "default_api_max_port")]
    pub api_max_port: u16,

    /// Additional configuration options
    #[serde(skip)]
    pub options: HashMap<String, String>,
}

// Default functions for serde
fn default_data_path() -> PathBuf {
    PathBuf::from("data/cv_data.json")
}

fn default_static_dir() -> PathBuf {
    PathBuf::from("static")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("dist")
}

fn default_typst_temp() -> PathBuf {
    PathBuf::from("temp_cv.typ")
}

fn default_github_cache_path() -> PathBuf {
    PathBuf::from(DEFAULT_GITHUB_CACHE_PATH)
}

fn default_db_path() -> PathBuf {
    PathBuf::from(DEFAULT_DB_PATH)
}

fn default_public_data() -> String {
    DEFAULT_PUBLIC_DATA.to_string()
}

fn default_db_storage() -> String {
    DEFAULT_DB_STORAGE.to_string()
}

fn default_api_port() -> u16 {
    DEFAULT_API_PORT
}

fn default_api_max_port() -> u16 {
    DEFAULT_API_MAX_PORT
}

fn default_github_cache_ttl() -> u64 {
    DEFAULT_GITHUB_CACHE_TTL
}

fn default_github_cache_refresh_strategy() -> String {
    DEFAULT_GITHUB_CACHE_REFRESH_STRATEGY.to_string()
}

fn default_github_rate_limit_strategy() -> String {
    DEFAULT_GITHUB_RATE_LIMIT_STRATEGY.to_string()
}

fn default_github_oauth_redirect_url() -> String {
    DEFAULT_GITHUB_OAUTH_REDIRECT_URL.to_string()
}

impl Default for AppConfig {
    /// Creates a default configuration
    fn default() -> Self {
        let output_dir = PathBuf::from("dist");
        let html_output = output_dir.join("cv.html");
        let pdf_output = output_dir.join("cv.pdf");

        Self {
            data_path: default_data_path(),
            static_dir: default_static_dir(),
            output_dir,
            html_output,
            typst_temp: default_typst_temp(),
            pdf_output,
            github_cache_path: default_github_cache_path(),
            db_path: default_db_path(),
            github_token: None,
            github_cache_ttl: default_github_cache_ttl(),
            github_cache_refresh_strategy: default_github_cache_refresh_strategy(),
            github_rate_limit_strategy: default_github_rate_limit_strategy(),
            github_oauth_client_id: None,
            github_oauth_client_secret: None,
            github_oauth_redirect_url: default_github_oauth_redirect_url(),
            public_data: default_public_data(),
            db_storage: default_db_storage(),
            api_port: default_api_port(),
            api_max_port: default_api_max_port(),
            options: HashMap::new(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from all available sources
    ///
    /// Order of precedence (highest to lowest):
    /// 1. Command-line arguments
    /// 2. Environment variables
    /// 3. Configuration file
    /// 4. Default values
    pub fn load() -> Result<Self> {
        info!("Loading application configuration from all available sources");
        let mut builder = Config::builder();

        // Start with default config
        let default_config = Self::default();
        debug!("Starting with default configuration values");
        debug!("Default data path: {}", default_config.data_path.display());
        debug!(
            "Default output directory: {}",
            default_config.output_dir.display()
        );
        debug!(
            "Default static directory: {}",
            default_config.static_dir.display()
        );

        // Add configuration from file if it exists
        let config_file = PathBuf::from("config.toml");
        if config_file.exists() {
            info!("Found configuration file: {}", config_file.display());
            builder = builder.add_source(File::from(config_file.clone()));
            debug!("Loading configuration from {}", config_file.display());
        } else {
            info!(
                "Configuration file {} not found, using defaults",
                config_file.display()
            );
            debug!("To create a configuration file, create a config.toml file in the project root");
        }

        // Add configuration from environment variables
        // Format: CV_SECTION__KEY (e.g., CV_PATHS__DATA_PATH)
        builder = builder.add_source(Environment::with_prefix("CV").separator("__"));
        info!("Checking for environment variables with prefix CV__");
        debug!("Environment variable format example: CV_PATHS__DATA_PATH=/path/to/data.json");

        // Build the configuration
        let config = builder.build()
            .context("Failed to build configuration from sources. Check configuration file syntax and environment variables")?;

        // Deserialize into AppConfig
        let mut app_config: AppConfig = config.try_deserialize()
            .context("Failed to deserialize configuration into AppConfig struct. This may indicate a mismatch between configuration schema and the AppConfig struct")?;

        info!("Configuration successfully loaded and deserialized");
        debug!("Loaded data path: {}", app_config.data_path.display());
        debug!(
            "Loaded output directory: {}",
            app_config.output_dir.display()
        );
        debug!(
            "Loaded static directory: {}",
            app_config.static_dir.display()
        );
        debug!(
            "API port range: {}-{}",
            app_config.api_port, app_config.api_max_port
        );

        // Set derived paths
        app_config.html_output = app_config.output_dir.join("cv.html");
        app_config.pdf_output = app_config.output_dir.join("cv.pdf");
        info!("Set derived output paths");
        debug!("HTML output path: {}", app_config.html_output.display());
        debug!("PDF output path: {}", app_config.pdf_output.display());

        // Try to get GitHub token from secure storage if not set
        if app_config.github_token.is_none() {
            match crate::credentials::get_github_token()
                .context("Failed to access secure storage for GitHub token")
            {
                Ok(Some(token)) => {
                    info!("Using GitHub API token from secure storage");
                    app_config.github_token = Some(token);
                    debug!("GitHub token successfully retrieved from secure storage");
                }
                Ok(None) => {
                    debug!("No GitHub API token found in secure storage");
                    info!("Consider setting a GitHub token to avoid API rate limiting");
                    info!("You can set a token with: cv --set-token <your-token>");
                }
                Err(e) => {
                    warn!("Failed to read GitHub token from secure storage: {}", e);
                    debug!("Will continue without GitHub token, which may result in rate limiting");
                    info!(
                        "To fix this issue, check your keyring system or set the token via environment variable GITHUB_TOKEN"
                    );
                }
            }
        } else {
            debug!("Using GitHub token provided in configuration");
        }

        debug!("Configuration loaded successfully");
        Ok(app_config)
    }

    /// Adds an option to the configuration
    pub fn with_option(self, key: &str, value: &str) -> Self {
        let mut options = self.options.clone();
        options.insert(key.to_string(), value.to_string());

        Self { options, ..self }
    }

    /// Gets a path as a string, with proper error handling
    pub fn path_to_string(&self, path: &Path) -> Result<String> {
        path.to_str()
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("Path contains invalid UTF-8 characters: {}", path.display()))
            .with_context(|| format!("Failed to convert path to string. This may indicate a path with non-UTF-8 characters: {}", path.display()))
    }

    /// Gets the HTML output path as a string
    pub fn html_output_str(&self) -> Result<String> {
        debug!(
            "Converting HTML output path to string: {}",
            self.html_output.display()
        );
        self.path_to_string(&self.html_output).with_context(|| {
            format!(
                "Failed to get HTML output path as string: {}",
                self.html_output.display()
            )
        })
    }

    /// Gets the PDF output path as a string
    pub fn pdf_output_str(&self) -> Result<String> {
        debug!(
            "Converting PDF output path to string: {}",
            self.pdf_output.display()
        );
        self.path_to_string(&self.pdf_output).with_context(|| {
            format!(
                "Failed to get PDF output path as string: {}",
                self.pdf_output.display()
            )
        })
    }

    /// Gets the Typst temp path as a string
    pub fn typst_temp_str(&self) -> Result<String> {
        debug!(
            "Converting Typst temp path to string: {}",
            self.typst_temp.display()
        );
        self.path_to_string(&self.typst_temp).with_context(|| {
            format!(
                "Failed to get Typst temp path as string: {}",
                self.typst_temp.display()
            )
        })
    }

    /// Gets the static directory path as a string
    pub fn static_dir_str(&self) -> Result<String> {
        debug!(
            "Converting static directory path to string: {}",
            self.static_dir.display()
        );
        self.path_to_string(&self.static_dir).with_context(|| {
            format!(
                "Failed to get static directory path as string: {}",
                self.static_dir.display()
            )
        })
    }

    /// Gets the output directory path as a string
    pub fn output_dir_str(&self) -> Result<String> {
        debug!(
            "Converting output directory path to string: {}",
            self.output_dir.display()
        );
        self.path_to_string(&self.output_dir).with_context(|| {
            format!(
                "Failed to get output directory path as string: {}",
                self.output_dir.display()
            )
        })
    }

    /// Gets the data path as a string
    pub fn data_path_str(&self) -> Result<String> {
        debug!(
            "Converting data path to string: {}",
            self.data_path.display()
        );
        self.path_to_string(&self.data_path).with_context(|| {
            format!(
                "Failed to get data path as string: {}",
                self.data_path.display()
            )
        })
    }

    /// Gets the GitHub cache path as a string
    pub fn github_cache_path_str(&self) -> Result<String> {
        if let Some(path) = self.options.get(GITHUB_CACHE_KEY) {
            debug!("Using GitHub cache path from options: {}", path);
            Ok(path.clone())
        } else {
            debug!(
                "Using default GitHub cache path: {}",
                self.github_cache_path.display()
            );
            self.path_to_string(&self.github_cache_path)
                .with_context(|| {
                    format!(
                        "Failed to get GitHub cache path as string: {}",
                        self.github_cache_path.display()
                    )
                })
        }
    }

    /// Gets the database path as a string
    pub fn db_path_str(&self) -> Result<String> {
        if let Some(path) = self.options.get(DB_PATH_KEY) {
            debug!("Using database path from options: {}", path);
            Ok(path.clone())
        } else {
            debug!("Using default database path: {}", self.db_path.display());
            self.path_to_string(&self.db_path).with_context(|| {
                format!(
                    "Failed to get database path as string: {}",
                    self.db_path.display()
                )
            })
        }
    }

    /// Gets the list of fields that should be publicly visible
    pub fn public_data(&self) -> Vector<String> {
        let public_data = if let Some(value) = self.options.get(PUBLIC_DATA_KEY) {
            debug!("Using public_data from options: {}", value);
            value.as_str()
        } else {
            debug!("Using default public_data: {}", self.public_data);
            &self.public_data
        };

        let fields: Vector<String> = public_data
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        debug!("Public data fields ({}): {:?}", fields.len(), fields);
        fields
    }

    /// Checks if a field should be publicly visible
    #[allow(dead_code)]
    pub fn is_public(&self, field: &str) -> bool {
        let result = self.public_data().contains(&field.to_string());
        debug!("Checking if field '{}' is public: {}", field, result);
        result
    }

    /// Gets the list of fields that should be stored in the database
    #[allow(dead_code)]
    pub fn db_storage(&self) -> Vector<String> {
        let db_storage = if let Some(value) = self.options.get(DB_STORAGE_KEY) {
            debug!("Using db_storage from options: {}", value);
            value.as_str()
        } else {
            debug!("Using default db_storage: {}", self.db_storage);
            &self.db_storage
        };

        let fields: Vector<String> = db_storage
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        debug!("Database storage fields ({}): {:?}", fields.len(), fields);
        fields
    }

    /// Checks if a field should be stored in the database
    #[allow(dead_code)]
    pub fn store_in_db(&self, field: &str) -> bool {
        let result = self.db_storage().contains(&field.to_string());
        debug!(
            "Checking if field '{}' should be stored in DB: {}",
            field, result
        );
        result
    }

    /// Gets the GitHub API token, if available
    pub fn github_token(&self) -> Option<&str> {
        let result = self.github_token.as_deref();
        if result.is_some() {
            debug!("GitHub token is available");
        } else {
            debug!("No GitHub token available, API requests may be rate limited");
        }
        result
    }

    /// Gets the GitHub OAuth client ID, if available
    pub fn github_oauth_client_id(&self) -> Option<&str> {
        if let Some(client_id) = self.options.get(GITHUB_OAUTH_CLIENT_ID_KEY) {
            debug!("Using GitHub OAuth client ID from options");
            Some(client_id)
        } else {
            let result = self.github_oauth_client_id.as_deref();
            if let Some(client_id) = result {
                if client_id == "your-github-client-id" {
                    warn!(
                        "Using placeholder GitHub OAuth client ID. This will not work for authentication."
                    );
                    warn!("Please set a real GitHub OAuth client ID in your configuration.");
                    debug!(
                        "You can create a GitHub OAuth app at https://github.com/settings/developers"
                    );
                } else {
                    debug!("GitHub OAuth client ID is available");
                }
            } else {
                debug!("No GitHub OAuth client ID available");
            }
            result
        }
    }

    /// Gets the GitHub OAuth client secret, if available
    pub fn github_oauth_client_secret(&self) -> Option<&str> {
        if let Some(client_secret) = self.options.get(GITHUB_OAUTH_CLIENT_SECRET_KEY) {
            debug!("Using GitHub OAuth client secret from options");
            Some(client_secret)
        } else {
            let result = self.github_oauth_client_secret.as_deref();
            if let Some(client_secret) = result {
                if client_secret == "your-github-client-secret" {
                    warn!(
                        "Using placeholder GitHub OAuth client secret. This will not work for authentication."
                    );
                    warn!("Please set a real GitHub OAuth client secret in your configuration.");
                    debug!(
                        "You can create a GitHub OAuth app at https://github.com/settings/developers"
                    );
                } else {
                    debug!("GitHub OAuth client secret is available");
                }
            } else {
                debug!("No GitHub OAuth client secret available");
            }
            result
        }
    }

    /// Gets the GitHub OAuth redirect URL
    pub fn github_oauth_redirect_url(&self) -> &str {
        if let Some(redirect_url) = self.options.get(GITHUB_OAUTH_REDIRECT_URL_KEY) {
            debug!(
                "Using GitHub OAuth redirect URL from options: {}",
                redirect_url
            );
            redirect_url
        } else {
            debug!(
                "Using default GitHub OAuth redirect URL: {}",
                self.github_oauth_redirect_url
            );
            &self.github_oauth_redirect_url
        }
    }

    /// Checks if GitHub OAuth is properly configured
    pub fn is_github_oauth_configured(&self) -> bool {
        match (
            self.github_oauth_client_id(),
            self.github_oauth_client_secret(),
        ) {
            (Some(client_id), Some(client_secret)) => {
                if client_id == "your-github-client-id"
                    || client_secret == "your-github-client-secret"
                {
                    warn!("GitHub OAuth is not properly configured. Using placeholder values.");
                    false
                } else {
                    debug!("GitHub OAuth is properly configured");
                    true
                }
            }
            _ => {
                debug!("GitHub OAuth is not configured (missing client ID or client secret)");
                false
            }
        }
    }
}

/// Extension trait to enable method chaining with pipe
#[allow(dead_code)]
pub trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

// Implement Pipe for AppConfig to enable method chaining
impl Pipe for AppConfig {}

// Implement Pipe for other types that might benefit from it
impl<T> Pipe for Option<T> {}
impl<T, E> Pipe for Result<T, E> {}
impl<T> Pipe for Vec<T> {}
impl<T> Pipe for Vector<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.data_path, PathBuf::from("data/cv_data.json"));
        assert_eq!(config.static_dir, PathBuf::from("static"));
        assert_eq!(config.output_dir, PathBuf::from("dist"));
        assert_eq!(config.html_output, PathBuf::from("dist/cv.html"));
        assert_eq!(config.pdf_output, PathBuf::from("dist/cv.pdf"));
    }

    #[test]
    fn test_with_option() {
        let config = AppConfig::default().with_option("test_key", "test_value");
        assert_eq!(config.options.get("test_key").unwrap(), "test_value");
    }

    #[test]
    fn test_public_data() {
        let config = AppConfig::default();
        let public_data = config.public_data();
        assert!(public_data.contains(&"name".to_string()));
        assert!(public_data.contains(&"title".to_string()));
        assert!(public_data.contains(&"experiences".to_string()));
    }

    #[test]
    fn test_is_public() {
        let config = AppConfig::default();
        assert!(config.is_public("name"));
        assert!(config.is_public("title"));
        assert!(config.is_public("experiences"));
    }

    #[test]
    fn test_db_storage() {
        let config = AppConfig::default();
        let db_storage = config.db_storage();
        assert!(db_storage.contains(&"personal_info".to_string()));
        assert!(db_storage.contains(&"experiences".to_string()));
        assert!(db_storage.contains(&"education".to_string()));
    }

    #[test]
    fn test_store_in_db() {
        let config = AppConfig::default();
        assert!(config.store_in_db("personal_info"));
        assert!(config.store_in_db("experiences"));
        assert!(config.store_in_db("education"));
    }
}
