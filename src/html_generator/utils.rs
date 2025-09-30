//! Utility functions for HTML generation
//!
//! This module provides common utilities used across the HTML generation system,
//! including file operations, content minification, and directory management.

use anyhow::{Context, Result};
use minify_html::{minify, Cfg};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Ensures the parent directory of a file path exists, creating it if necessary
///
/// # Arguments
///
/// * `file_path` - Path to the file whose parent directory should exist
///
/// # Returns
///
/// A Result indicating success or failure
pub fn ensure_parent_dir_exists(file_path: &str) -> Result<()> {
    Path::new(file_path)
        .parent()
        .map(fs::create_dir_all)
        .transpose()
        .context("Failed to create parent directory")?;

    Ok(())
}

/// Writes content to a file, with optional minification and compression in release mode
///
/// # Arguments
///
/// * `path` - Path where the content will be written
/// * `content` - Content to write to the file
///
/// # Returns
///
/// A Result indicating success or failure
pub fn write_file(path: &str, content: &str) -> Result<()> {
    // In debug mode, just write the file as is
    if cfg!(debug_assertions) {
        return fs::write(path, content).with_context(|| format!("Failed to write to {path}"));
    }

    // In release mode, apply optimizations based on file extension
    let path_obj = Path::new(path);
    let extension = path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "html" => {
            // Minify HTML content
            let minified_bytes = minify_html_content(content)?;
            fs::write(path, minified_bytes)
                .with_context(|| format!("Failed to write to {path}"))?;
        }
        "css" => {
            // Minify CSS content
            let minified_content = minify_css_content(content)?;
            fs::write(path, minified_content)
                .with_context(|| format!("Failed to write to {path}"))?;
        }
        _ => {
            // Write other files as-is
            fs::write(path, content).with_context(|| format!("Failed to write to {path}"))?;
        }
    }

    Ok(())
}

/// Minifies HTML content using minify-html
///
/// # Arguments
///
/// * `content` - HTML content to minify
///
/// # Returns
///
/// A Result containing the minified HTML as bytes
pub fn minify_html_content(content: &str) -> Result<Vec<u8>> {
    let cfg = Cfg {
        minify_css: true,
        minify_js: false, // Disabled JS minification to avoid panic in minify-js
        ..Cfg::default()
    };

    let minified = minify(content.as_bytes(), &cfg);
    Ok(minified)
}

/// Minifies CSS content (placeholder implementation)
///
/// # Arguments
///
/// * `content` - CSS content to minify
///
/// # Returns
///
/// A Result containing the minified CSS content
pub fn minify_css_content(content: &str) -> Result<String> {
    // Basic CSS minification: remove comments and extra whitespace
    let mut minified = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("/*"))
        .collect::<Vec<&str>>()
        .join(" ");

    // Remove spaces around certain characters
    minified = minified.replace(" { ", "{");
    minified = minified.replace(" } ", "}");
    minified = minified.replace("; ", ";");
    minified = minified.replace(": ", ":");

    Ok(minified)
}

/// Writes gzipped content to a file (currently disabled)
///
/// # Arguments
///
/// * `_path` - Path where the gzipped content would be written
/// * `_content` - Content to compress and write
///
/// # Returns
///
/// A Result indicating success or failure
#[allow(dead_code)]
pub fn write_gzipped_file(_path: &str, _content: &[u8]) -> Result<()> {
    // Gzip compression is currently disabled
    // Uncomment when flate2 dependency is re-enabled
    /*
    let file = fs::File::create(path)
        .with_context(|| format!("Failed to create gzipped file: {path}"))?;

    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(content)
        .with_context(|| format!("Failed to write gzipped content to {path}"))?;

    encoder.finish()
        .with_context(|| format!("Failed to finish gzipped file: {path}"))?;
    */

    Ok(())
}

/// Gets the current git commit hash for cache busting
///
/// Returns a short git commit hash (8 characters) to use as a version string
/// for CSS and JS URLs. If git is not available or fails, returns a timestamp
/// as fallback.
///
/// # Returns
///
/// A String containing the version identifier
pub fn get_cache_version() -> String {
    // Try to get git commit hash
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
    {
        if output.status.success() {
            if let Ok(hash) = String::from_utf8(output.stdout) {
                let trimmed = hash.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // Fallback: use build timestamp
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    format!("{}", timestamp)
}
