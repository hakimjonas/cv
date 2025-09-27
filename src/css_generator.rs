//! CSS Generation Module
//!
//! This module provides unified CSS generation for the CV generator, with a focus on
//! performance, maintainability, and functional programming principles.
//!
//! ## Architecture
//!
//! The CSS generation system uses a **provider-based architecture** that:
//! - Fetches colorschemes from various sources (GitHub repositories, local files)
//! - Converts color palettes to CSS custom properties
//! - Caches both remote data and generated CSS for performance
//! - Supports multiple colorscheme sources (iTerm2, Ghostty, Base16)
//!
//! ## Key Features
//!
//! ### 1. Intelligent Caching
//! - **Remote data caching**: GitHub API responses are cached locally
//! - **CSS generation caching**: Generated CSS is cached based on configuration hash
//! - **Smart invalidation**: Cache is invalidated when configuration changes
//!
//! ### 2. Multiple Colorscheme Sources
//! - **iTerm2 Color Schemes**: Comprehensive collection from mbadolato/iTerm2-Color-Schemes
//! - **Ghostty Colors**: Modern terminal colorschemes from ghostty-org/ghostty-colors
//! - **Base16**: Classic 16-color schemes from base16-project
//! - **Custom Sources**: Support for any GitHub repository with colorscheme files
//!
//! ### 3. Performance Optimizations
//! - Configuration-based cache invalidation
//! - Minimal file I/O operations
//! - Efficient string manipulation
//! - Lazy loading of colorscheme data
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use cv_generator::css_generator::generate_colorscheme_css;
//! use cv_generator::site_config::ColorschemeConfig;
//!
//! fn main() -> anyhow::Result<()> {
//!     let config = ColorschemeConfig {
//!         name: "Rose Pine Moon".to_string(),
//!         source: Some("iterm2".to_string()),
//!         variant: Some("default".to_string()),
//!         url: None,
//!         custom_colors: None,
//!     };
//!
//!     generate_colorscheme_css(&config, "dist/css/generated/colorscheme.css")?;
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::colorscheme_provider::{
    CachedProvider, ColorSchemeProvider, GitHubSchemeProvider, SchemeFormat, ToCss,
};
use crate::site_config::ColorschemeConfig;

/// Check if CSS file needs to be regenerated based on config hash
fn needs_regeneration(config: &ColorschemeConfig, css_path: &str) -> Result<bool> {
    let css_file = Path::new(css_path);

    // If CSS file doesn't exist, regeneration is needed
    if !css_file.exists() {
        return Ok(true);
    }

    // Create a simple hash of the configuration to detect changes
    let config_hash = format!(
        "{}{}{}",
        config.name,
        config.variant.as_deref().unwrap_or("default"),
        config.source.as_deref().unwrap_or("iterm2")
    );

    // Check if hash comment is in the file
    let css_content = fs::read_to_string(css_path)
        .with_context(|| format!("Failed to read existing CSS file: {css_path}"))?;

    let expected_comment = format!("/* Config hash: {} */", config_hash);

    // If the hash comment is not found or doesn't match, regeneration is needed
    Ok(!css_content.contains(&expected_comment))
}

/// Generate CSS from a colorscheme configuration using providers
///
/// This is the smart, simple approach that leverages existing infrastructure
/// instead of hardcoding color values. Includes caching to improve build performance.
pub fn generate_colorscheme_css(config: &ColorschemeConfig, path: &str) -> Result<()> {
    // Check if regeneration is needed
    if !needs_regeneration(config, path)? {
        println!("Using cached colorscheme CSS: {path}");
        return Ok(());
    }
    // Select the appropriate provider based on source
    let provider: Box<dyn ColorSchemeProvider> = match config.source.as_deref() {
        Some("ghostty-colors") | Some("ghostty") => Box::new(CachedProvider::new(
            GitHubSchemeProvider::ghostty_colors(),
            ".cache/colorschemes",
        )),
        Some("iterm2") | Some("iTerm2-Color-Schemes") => Box::new(CachedProvider::new(
            GitHubSchemeProvider::iterm2_schemes(),
            ".cache/colorschemes",
        )),
        Some("base16") => Box::new(CachedProvider::new(
            GitHubSchemeProvider::base16_schemes(),
            ".cache/colorschemes",
        )),
        Some(custom_repo) if custom_repo.contains('/') => {
            // Custom GitHub repository
            let format = detect_format_from_url(config.url.as_deref());
            Box::new(CachedProvider::new(
                GitHubSchemeProvider::new(custom_repo, format),
                ".cache/colorschemes",
            ))
        }
        _ => {
            // Default to iTerm2 schemes (most comprehensive collection)
            Box::new(CachedProvider::new(
                GitHubSchemeProvider::iterm2_schemes(),
                ".cache/colorschemes",
            ))
        }
    };

    // Fetch the color palette
    let palette = provider
        .fetch(&config.name, config.variant.as_deref())
        .with_context(|| {
            format!(
                "Failed to fetch colorscheme '{}' from {}",
                config.name,
                provider.provider_name()
            )
        })?;

    // Generate CSS
    let mut css_content = String::new();

    // Add config hash for caching
    let config_hash = format!(
        "{}{}{}",
        config.name,
        config.variant.as_deref().unwrap_or("default"),
        config.source.as_deref().unwrap_or("iterm2")
    );
    css_content.push_str(&format!("/* Config hash: {} */\n", config_hash));

    // Add header comment
    css_content.push_str(&format!(
        "/* Colorscheme: {} {} */\n",
        config.name,
        config.variant.as_deref().unwrap_or("default")
    ));

    if let Some(url) = &config.url {
        css_content.push_str(&format!("/* Source: {} */\n", url));
    }

    css_content.push_str(&format!("/* Provider: {} */\n\n", provider.provider_name()));

    // Generate CSS variables
    css_content.push_str(&palette.to_css_variables());

    // Also generate for theme classes
    let is_dark = detect_if_dark(&palette);
    let theme_class = if is_dark {
        ".theme-dark"
    } else {
        ".theme-light"
    };
    css_content.push('\n');
    css_content.push_str(&palette.to_theme_css(theme_class));

    // Write the CSS file
    fs::write(path, css_content)
        .with_context(|| format!("Failed to write colorscheme CSS file to {path}"))?;

    println!("Generated colorscheme CSS: {path}");
    println!(
        "  Source: {}",
        config.source.as_deref().unwrap_or("default")
    );
    println!("  Provider: {}", provider.provider_name());

    Ok(())
}

/// Detect format from URL or repository structure
fn detect_format_from_url(url: Option<&str>) -> SchemeFormat {
    if let Some(url) = url {
        if url.contains("ghostty") {
            return SchemeFormat::Toml;
        }
        if url.contains("iterm") || url.contains("iTerm") {
            return SchemeFormat::ITerm2;
        }
        if url.contains("base16") {
            return SchemeFormat::Yaml;
        }
        if url.contains("alacritty") {
            return SchemeFormat::Toml;
        }
        if url.contains("xresources") || url.contains("Xresources") {
            return SchemeFormat::XResources;
        }
    }
    SchemeFormat::Json // Default fallback
}

/// Simple heuristic to detect if a color scheme is dark
fn detect_if_dark(palette: &crate::colorscheme_provider::ColorPalette) -> bool {
    // Parse the background color and check its luminance
    if let Some(bg) = parse_hex_color(&palette.background) {
        let luminance = (0.299 * bg.0 as f64 + 0.587 * bg.1 as f64 + 0.114 * bg.2 as f64) / 255.0;
        return luminance < 0.5;
    }
    true // Default to dark if we can't parse
}

/// Parse a hex color string to RGB values
fn parse_hex_color(color: &str) -> Option<(u8, u8, u8)> {
    let color = color.trim_start_matches('#');
    if color.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&color[0..2], 16).ok()?;
    let g = u8::from_str_radix(&color[2..4], 16).ok()?;
    let b = u8::from_str_radix(&color[4..6], 16).ok()?;

    Some((r, g, b))
}

/// List available color schemes from configured provider
#[allow(dead_code)]
pub fn list_available_schemes(source: Option<&str>) -> Result<Vec<String>> {
    let provider: Box<dyn ColorSchemeProvider> = match source {
        Some("ghostty-colors") | Some("ghostty") => {
            Box::new(GitHubSchemeProvider::ghostty_colors())
        }
        Some("iterm2") | Some("iTerm2-Color-Schemes") => {
            Box::new(GitHubSchemeProvider::iterm2_schemes())
        }
        Some("base16") => Box::new(GitHubSchemeProvider::base16_schemes()),
        _ => Box::new(GitHubSchemeProvider::iterm2_schemes()),
    };

    provider.list_available()
}
