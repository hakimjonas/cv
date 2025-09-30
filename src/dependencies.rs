//! Cargo.toml dependency parsing
//!
//! This module provides functionality to parse Cargo.toml and extract
//! dependency information for display in the application footer.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Represents a single dependency with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Version string
    pub version: String,
    /// Optional description (can be added manually)
    pub description: Option<String>,
    /// Optional URL to documentation or repository
    pub url: Option<String>,
}

/// Raw Cargo.toml structure for parsing
#[derive(Debug, Deserialize)]
struct CargoToml {
    dependencies: HashMap<String, toml::Value>,
}

/// Parses Cargo.toml and extracts dependency information
///
/// # Arguments
///
/// * `cargo_toml_path` - Path to Cargo.toml file
///
/// # Returns
///
/// A vector of dependencies with their metadata
pub fn parse_dependencies(cargo_toml_path: &str) -> Result<Vec<Dependency>> {
    let content = fs::read_to_string(cargo_toml_path)
        .context("Failed to read Cargo.toml")?;

    let cargo_toml: CargoToml = toml::from_str(&content)
        .context("Failed to parse Cargo.toml")?;

    let mut dependencies = Vec::new();

    for (name, value) in cargo_toml.dependencies {
        let version = extract_version(&value);
        let (url, description) = get_dependency_metadata(&name);

        dependencies.push(Dependency {
            name,
            version,
            description: Some(description.to_string()),
            url: Some(url.to_string()),
        });
    }

    // Sort by name for consistent display
    dependencies.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(dependencies)
}

/// Extracts version string from toml::Value
fn extract_version(value: &toml::Value) -> String {
    match value {
        toml::Value::String(v) => v.clone(),
        toml::Value::Table(t) => {
            t.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string()
        }
        _ => "*".to_string(),
    }
}

/// Returns metadata (URL and description) for well-known dependencies
fn get_dependency_metadata(name: &str) -> (&'static str, &'static str) {
    match name {
        "serde" => ("https://serde.rs/", "Serialization framework"),
        "serde_json" => ("https://github.com/serde-rs/json", "JSON support"),
        "serde_yaml" => ("https://github.com/dtolnay/serde-yaml", "YAML support"),
        "askama" => ("https://github.com/djc/askama", "Templating engine"),
        "im" => ("https://github.com/bodil/im-rs", "Immutable data structures"),
        "anyhow" => ("https://github.com/dtolnay/anyhow", "Error handling"),
        "reqwest" => ("https://github.com/seanmonstar/reqwest", "HTTP client"),
        "tokio" => ("https://tokio.rs/", "Async runtime"),
        "git2" => ("https://github.com/rust-lang/git2-rs", "Git operations"),
        "config" => ("https://github.com/mehcode/config-rs", "Configuration management"),
        "toml" => ("https://github.com/toml-rs/toml", "TOML parsing"),
        "minify-html" => ("https://github.com/wilsonzlin/minify-html", "HTML/CSS optimization"),
        "pulldown-cmark" => ("https://github.com/raphlinus/pulldown-cmark", "Markdown processing"),
        "gray_matter" => ("https://github.com/acheronfail/gray_matter", "Front matter parsing"),
        "tracing" => ("https://github.com/tokio-rs/tracing", "Logging and diagnostics"),
        "tracing-subscriber" => ("https://github.com/tokio-rs/tracing", "Logging subscriber"),
        "chrono" => ("https://github.com/chronotope/chrono", "Date and time"),
        "regex" => ("https://github.com/rust-lang/regex", "Regular expressions"),
        "base64" => ("https://github.com/marshallpierce/rust-base64", "Base64 encoding"),
        "tempfile" => ("https://github.com/Stebalien/tempfile", "Temporary files"),
        _ => ("https://crates.io/", "Rust crate"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_string() {
        let value = toml::Value::String("1.0.0".to_string());
        assert_eq!(extract_version(&value), "1.0.0");
    }

    #[test]
    fn test_extract_version_table() {
        let mut table = toml::value::Table::new();
        table.insert("version".to_string(), toml::Value::String("2.0.0".to_string()));
        let value = toml::Value::Table(table);
        assert_eq!(extract_version(&value), "2.0.0");
    }

    #[test]
    fn test_dependency_metadata() {
        let (url, desc) = get_dependency_metadata("serde");
        assert_eq!(url, "https://serde.rs/");
        assert_eq!(desc, "Serialization framework");
    }
}
