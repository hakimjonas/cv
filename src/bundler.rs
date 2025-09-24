//! Asset bundling module for combining and optimizing frontend assets
//!
//! This module provides functionality for bundling CSS and JavaScript files
//! based on a configuration file, applying minification, and generating source maps.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Represents a bundling configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct BundleConfig {
    /// Bundle definitions
    pub bundles: Bundles,
    /// Output configuration
    pub output: OutputConfig,
}

/// Bundle definitions for different asset types
#[derive(Debug, Deserialize, Serialize)]
pub struct Bundles {
    /// CSS bundles
    #[serde(default)]
    pub css: HashMap<String, Vec<String>>,
    /// JavaScript bundles
    #[serde(default)]
    pub js: HashMap<String, Vec<String>>,
}

/// Output configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Output directory for bundled files
    pub directory: String,
}

/// Loads the bundling configuration from a TOML file
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
///
/// # Returns
///
/// A Result containing the parsed configuration
pub fn load_config(config_path: &str) -> Result<BundleConfig> {
    let config_content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read bundling configuration from {config_path}"))?;

    let config: BundleConfig = toml::from_str(&config_content)
        .with_context(|| format!("Failed to parse bundling configuration from {config_path}"))?;

    Ok(config)
}

/// Creates a default bundling configuration
///
/// # Returns
///
/// A default bundling configuration
pub fn default_config() -> BundleConfig {
    let mut css_bundles = HashMap::new();
    css_bundles.insert(
        "main".to_string(),
        vec![
            "css/base/reset.css".to_string(),
            "css/base/variables.css".to_string(),
            "css/layout/layout.css".to_string(),
            "css/components/header.css".to_string(),
            "css/components/footer.css".to_string(),
            "css/components/navigation.css".to_string(),
            "css/components/buttons.css".to_string(),
            "css/components/cards.css".to_string(),
            "css/components/accordion.css".to_string(),
            "css/components/social.css".to_string(),
            "css/utilities/utilities.css".to_string(),
            "css/main.css".to_string(),
        ],
    );
    css_bundles.insert(
        "blog".to_string(),
        vec![
            "css/base/reset.css".to_string(),
            "css/base/variables.css".to_string(),
            "css/layout/layout.css".to_string(),
            "css/components/header.css".to_string(),
            "css/components/footer.css".to_string(),
            "css/components/navigation.css".to_string(),
            "blog.css".to_string(),
        ],
    );
    css_bundles.insert("syntax".to_string(), vec!["prism.css".to_string()]);

    let mut js_bundles = HashMap::new();
    js_bundles.insert("main".to_string(), vec!["js/scripts.js".to_string()]);
    js_bundles.insert(
        "blog".to_string(),
        vec![
            "js/simple-api-client.js".to_string(),
            "js/blog-integration.js".to_string(),
        ],
    );
    js_bundles.insert("debug".to_string(), vec!["js/blog-debug.js".to_string()]);
    js_bundles.insert("syntax".to_string(), vec!["prism.js".to_string()]);

    BundleConfig {
        bundles: Bundles {
            css: css_bundles,
            js: js_bundles,
        },
        output: OutputConfig {
            directory: "dist".to_string(),
        },
    }
}

/// Writes the default bundling configuration to a TOML file
///
/// # Arguments
///
/// * `config_path` - Path where the configuration file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn write_default_config(config_path: &str) -> Result<()> {
    let config = default_config();
    let config_content = toml::to_string_pretty(&config)
        .context("Failed to serialize default bundling configuration")?;

    fs::write(config_path, config_content).with_context(|| {
        format!("Failed to write default bundling configuration to {config_path}")
    })?;

    info!("Created default bundling configuration at {}", config_path);
    Ok(())
}

/// Bundles CSS files based on the configuration
///
/// # Arguments
///
/// * `config` - The bundling configuration
/// * `static_dir` - Directory containing static assets
///
/// # Returns
///
/// A Result indicating success or failure
pub fn bundle_css(config: &BundleConfig, static_dir: &str) -> Result<()> {
    let output_dir = &config.output.directory;
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    for (bundle_name, files) in &config.bundles.css {
        info!("Bundling CSS: {}", bundle_name);
        let output_path = Path::new(output_dir).join(format!("{bundle_name}.bundle.css"));

        // Read and concatenate files
        let mut bundle_content = String::new();
        for file in files {
            let file_path = Path::new(static_dir).join(file);
            debug!("Adding file to bundle: {}", file_path.display());

            let file_content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read CSS file: {}", file_path.display()))?;

            // Add file path comment for debugging
            bundle_content.push_str(&format!("/* Source: {file} */\n"));
            bundle_content.push_str(&file_content);
            bundle_content.push('\n');
        }

        // Minify the bundle if in release mode
        #[cfg(not(debug_assertions))]
        let final_content = minify_css(&bundle_content)
            .with_context(|| format!("Failed to minify CSS bundle: {}", bundle_name))?;

        #[cfg(debug_assertions)]
        let final_content = bundle_content;

        // Write the bundle
        fs::write(&output_path, &final_content)
            .with_context(|| format!("Failed to write CSS bundle to {}", output_path.display()))?;

        info!("Created CSS bundle: {}", output_path.display());

        // Skip gzipped version for now (asset_processor disabled)
        #[cfg(not(debug_assertions))]
        {
            // TODO: Re-enable gzip compression when asset_processor is re-enabled
            info!("Skipping gzipped CSS bundle creation (asset_processor disabled)");
        }
    }

    Ok(())
}

/// Bundles JavaScript files based on the configuration
///
/// # Arguments
///
/// * `config` - The bundling configuration
/// * `static_dir` - Directory containing static assets
///
/// # Returns
///
/// A Result indicating success or failure
pub fn bundle_js(config: &BundleConfig, static_dir: &str) -> Result<()> {
    let output_dir = &config.output.directory;
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    for (bundle_name, files) in &config.bundles.js {
        info!("Bundling JavaScript: {}", bundle_name);
        let output_path = Path::new(output_dir).join(format!("{bundle_name}.bundle.js"));

        // Read and concatenate files
        let mut bundle_content = String::new();
        for file in files {
            let file_path = Path::new(static_dir).join(file);
            debug!("Adding file to bundle: {}", file_path.display());

            let file_content = fs::read_to_string(&file_path).with_context(|| {
                format!("Failed to read JavaScript file: {}", file_path.display())
            })?;

            // Add file path comment for debugging
            bundle_content.push_str(&format!("/* Source: {file} */\n"));
            bundle_content.push_str(&file_content);
            bundle_content.push('\n');
        }

        // Minify the bundle if in release mode
        #[cfg(not(debug_assertions))]
        let final_content = minify_js(&bundle_content)
            .with_context(|| format!("Failed to minify JavaScript bundle: {}", bundle_name))?;

        #[cfg(debug_assertions)]
        let final_content = bundle_content;

        // Write the bundle
        fs::write(&output_path, &final_content).with_context(|| {
            format!(
                "Failed to write JavaScript bundle to {}",
                output_path.display()
            )
        })?;

        info!("Created JavaScript bundle: {}", output_path.display());

        // Skip gzipped version for now (asset_processor disabled)
        #[cfg(not(debug_assertions))]
        {
            // TODO: Re-enable gzip compression when asset_processor is re-enabled
            info!("Skipping gzipped JavaScript bundle creation (asset_processor disabled)");
        }
    }

    Ok(())
}

/// Minifies CSS content using lightningcss
///
/// # Arguments
///
/// * `content` - CSS content to minify
///
/// # Returns
///
/// A Result containing the minified CSS
#[cfg(not(debug_assertions))]
fn minify_css(content: &str) -> Result<String> {
    // Simple CSS minification (asset_processor disabled)
    // Remove comments, extra whitespace, and newlines
    let minified = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("/*"))
        .collect::<Vec<_>>()
        .join(" ")
        .replace("; ", ";")
        .replace(": ", ":")
        .replace(" {", "{")
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace("} ", "}");
    Ok(minified)
}

/// Minifies JavaScript content using minify-js
///
/// # Arguments
///
/// * `content` - JavaScript content to minify
///
/// # Returns
///
/// A Result containing the minified JavaScript
#[cfg(not(debug_assertions))]
fn minify_js(content: &str) -> Result<String> {
    // For now, just return the content as is
    // In the future, this could be replaced with minify-js for better minification
    Ok(content.to_string())
}

/// Processes all assets based on the bundling configuration
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
/// * `static_dir` - Directory containing static assets
///
/// # Returns
///
/// A Result indicating success or failure
pub fn process_assets(config_path: &str, static_dir: &str) -> Result<()> {
    // Check if the configuration file exists
    if !Path::new(config_path).exists() {
        info!("Bundling configuration not found, creating default configuration");
        write_default_config(config_path)?;
    }

    // Load the configuration
    let config = load_config(config_path)?;

    // Bundle CSS files
    bundle_css(&config, static_dir)?;

    // Bundle JavaScript files
    bundle_js(&config, static_dir)?;

    // Generate server configuration files (disabled for now)
    // crate::asset_processor::generate_server_configs(&config.output.directory)?;

    info!("Asset processing completed successfully");
    Ok(())
}
