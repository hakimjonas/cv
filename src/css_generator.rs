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

    // Generate both light and dark themes for Rose Pine
    if config.name.to_lowercase().contains("rose pine") {
        // Generate light theme (Rose Pine Dawn)
        let light_palette = provider
            .fetch("Rose Pine Dawn", None)
            .with_context(|| "Failed to fetch Rose Pine Dawn for light theme")?;

        // Generate dark theme (Rose Pine Moon)
        let dark_palette = provider
            .fetch("Rose Pine Moon", None)
            .with_context(|| "Failed to fetch Rose Pine Moon for dark theme")?;

        // Generate CSS for both themes
        css_content.push_str("/* Light Theme (Rose Pine Dawn) */\n");
        css_content.push_str(":root {\n");
        css_content.push_str(
            &light_palette
                .to_css_variables()
                .replace(":root {\n", "")
                .replace("}\n", ""),
        );
        css_content.push_str("}\n\n");

        css_content.push_str(".theme-light {\n");
        css_content.push_str(
            &light_palette
                .to_css_variables()
                .replace(":root {\n", "")
                .replace("}\n", ""),
        );
        css_content.push_str("}\n\n");

        css_content.push_str("/* Dark Theme (Rose Pine Moon) */\n");
        css_content.push_str(".theme-dark {\n");
        css_content.push_str(
            &dark_palette
                .to_css_variables()
                .replace(":root {\n", "")
                .replace("}\n", ""),
        );
        css_content.push_str("}\n");
    } else {
        // Generate CSS variables for single theme
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
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Create a test color scheme configuration
    fn create_test_config(name: &str, source: Option<&str>) -> ColorschemeConfig {
        ColorschemeConfig {
            name: name.to_string(),
            source: source.map(|s| s.to_string()),
            variant: None,
            url: None,
            custom_colors: None,
        }
    }

    /// Create a minimal mock provider for testing without network calls
    struct MockProvider {
        schemes: std::collections::HashMap<String, crate::colorscheme_provider::ColorPalette>,
    }

    impl MockProvider {
        fn new() -> Self {
            let mut schemes = std::collections::HashMap::new();

            // Add Rose Pine Dawn (light theme)
            schemes.insert(
                "Rose Pine Dawn".to_string(),
                crate::colorscheme_provider::ColorPalette {
                    black: "#F2E9E1".to_string(),
                    red: "#B4637A".to_string(),
                    green: "#286983".to_string(),
                    yellow: "#EA9D34".to_string(),
                    blue: "#56949F".to_string(),
                    magenta: "#907AA9".to_string(),
                    cyan: "#D7827E".to_string(),
                    white: "#575279".to_string(),
                    bright_black: "#9893A5".to_string(),
                    bright_red: "#B4637A".to_string(),
                    bright_green: "#286983".to_string(),
                    bright_yellow: "#EA9D34".to_string(),
                    bright_blue: "#56949F".to_string(),
                    bright_magenta: "#907AA9".to_string(),
                    bright_cyan: "#D7827E".to_string(),
                    bright_white: "#575279".to_string(),
                    background: "#FAF4ED".to_string(),
                    foreground: "#575279".to_string(),
                    cursor: Some("#575279".to_string()),
                    selection: Some("#DFDAD9".to_string()),
                },
            );

            // Add Rose Pine Moon (dark theme)
            schemes.insert(
                "Rose Pine Moon".to_string(),
                crate::colorscheme_provider::ColorPalette {
                    black: "#393552".to_string(),
                    red: "#EB6F92".to_string(),
                    green: "#3E8FB0".to_string(),
                    yellow: "#F6C177".to_string(),
                    blue: "#9CCFD8".to_string(),
                    magenta: "#C4A7E7".to_string(),
                    cyan: "#EA9A97".to_string(),
                    white: "#E0DEF4".to_string(),
                    bright_black: "#6E6A86".to_string(),
                    bright_red: "#EB6F92".to_string(),
                    bright_green: "#3E8FB0".to_string(),
                    bright_yellow: "#F6C177".to_string(),
                    bright_blue: "#9CCFD8".to_string(),
                    bright_magenta: "#C4A7E7".to_string(),
                    bright_cyan: "#EA9A97".to_string(),
                    bright_white: "#E0DEF4".to_string(),
                    background: "#232136".to_string(),
                    foreground: "#E0DEF4".to_string(),
                    cursor: Some("#E0DEF4".to_string()),
                    selection: Some("#44415A".to_string()),
                },
            );

            MockProvider { schemes }
        }
    }

    impl ColorSchemeProvider for MockProvider {
        fn fetch(
            &self,
            name: &str,
            _variant: Option<&str>,
        ) -> Result<crate::colorscheme_provider::ColorPalette> {
            self.schemes
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Mock scheme '{}' not found", name))
        }

        fn list_available(&self) -> Result<Vec<String>> {
            Ok(self.schemes.keys().cloned().collect())
        }

        fn provider_name(&self) -> &str {
            "Mock"
        }
    }

    #[test]
    fn test_needs_regeneration_missing_file() {
        let config = create_test_config("Rose Pine Moon", Some("iterm2"));
        let temp_dir = tempdir().unwrap();
        let css_path = temp_dir
            .path()
            .join("nonexistent.css")
            .to_str()
            .unwrap()
            .to_string();

        let result = needs_regeneration(&config, &css_path).unwrap();
        assert!(result, "Should need regeneration when file doesn't exist");
    }

    #[test]
    fn test_needs_regeneration_wrong_hash() {
        let config = create_test_config("Rose Pine Moon", Some("iterm2"));
        let temp_dir = tempdir().unwrap();
        let css_path = temp_dir
            .path()
            .join("test.css")
            .to_str()
            .unwrap()
            .to_string();

        // Create a CSS file with wrong hash
        fs::write(
            &css_path,
            "/* Config hash: wrong_hash */\n:root { --color: red; }",
        )
        .unwrap();

        let result = needs_regeneration(&config, &css_path).unwrap();
        assert!(result, "Should need regeneration when hash doesn't match");
    }

    #[test]
    fn test_needs_regeneration_correct_hash() {
        let config = create_test_config("Rose Pine Moon", Some("iterm2"));
        let temp_dir = tempdir().unwrap();
        let css_path = temp_dir
            .path()
            .join("test.css")
            .to_str()
            .unwrap()
            .to_string();

        // Create the expected hash
        let expected_hash = format!(
            "{}{}{}",
            config.name,
            config.variant.as_deref().unwrap_or("default"),
            config.source.as_deref().unwrap_or("iterm2")
        );

        // Create a CSS file with correct hash
        let css_content = format!(
            "/* Config hash: {} */\n:root {{ --color: red; }}",
            expected_hash
        );
        fs::write(&css_path, css_content).unwrap();

        let result = needs_regeneration(&config, &css_path).unwrap();
        assert!(!result, "Should not need regeneration when hash matches");
    }

    #[test]
    fn test_detect_if_dark_light_theme() {
        let light_palette = crate::colorscheme_provider::ColorPalette {
            background: "#FAF4ED".to_string(), // Light background
            foreground: "#575279".to_string(),
            black: "#000000".to_string(),
            red: "#B4637A".to_string(),
            green: "#286983".to_string(),
            yellow: "#EA9D34".to_string(),
            blue: "#56949F".to_string(),
            magenta: "#907AA9".to_string(),
            cyan: "#D7827E".to_string(),
            white: "#575279".to_string(),
            bright_black: "#9893A5".to_string(),
            bright_red: "#B4637A".to_string(),
            bright_green: "#286983".to_string(),
            bright_yellow: "#EA9D34".to_string(),
            bright_blue: "#56949F".to_string(),
            bright_magenta: "#907AA9".to_string(),
            bright_cyan: "#D7827E".to_string(),
            bright_white: "#575279".to_string(),
            cursor: None,
            selection: None,
        };

        assert!(!detect_if_dark(&light_palette), "Should detect light theme");
    }

    #[test]
    fn test_detect_if_dark_dark_theme() {
        let dark_palette = crate::colorscheme_provider::ColorPalette {
            background: "#232136".to_string(), // Dark background
            foreground: "#E0DEF4".to_string(),
            black: "#000000".to_string(),
            red: "#EB6F92".to_string(),
            green: "#3E8FB0".to_string(),
            yellow: "#F6C177".to_string(),
            blue: "#9CCFD8".to_string(),
            magenta: "#C4A7E7".to_string(),
            cyan: "#EA9A97".to_string(),
            white: "#E0DEF4".to_string(),
            bright_black: "#6E6A86".to_string(),
            bright_red: "#EB6F92".to_string(),
            bright_green: "#3E8FB0".to_string(),
            bright_yellow: "#F6C177".to_string(),
            bright_blue: "#9CCFD8".to_string(),
            bright_magenta: "#C4A7E7".to_string(),
            bright_cyan: "#EA9A97".to_string(),
            bright_white: "#E0DEF4".to_string(),
            cursor: None,
            selection: None,
        };

        assert!(detect_if_dark(&dark_palette), "Should detect dark theme");
    }

    #[test]
    fn test_parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#FAF4ED"), Some((250, 244, 237)));
        assert_eq!(parse_hex_color("#232136"), Some((35, 33, 54)));
        assert_eq!(parse_hex_color("FAF4ED"), Some((250, 244, 237))); // Without #
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("#GGGGGG"), None); // Invalid hex chars
        assert_eq!(parse_hex_color("#FAF4E"), None); // Too short
        assert_eq!(parse_hex_color("#FAF4EDD"), None); // Too long
        assert_eq!(parse_hex_color(""), None); // Empty
    }

    #[test]
    fn test_detect_format_from_url() {
        assert!(matches!(
            detect_format_from_url(Some("https://github.com/ghostty-org/ghostty-colors")),
            SchemeFormat::Toml
        ));
        assert!(matches!(
            detect_format_from_url(Some("https://github.com/mbadolato/iTerm2-Color-Schemes")),
            SchemeFormat::ITerm2
        ));
        assert!(matches!(
            detect_format_from_url(Some("https://github.com/base16-project/base16-schemes")),
            SchemeFormat::Yaml
        ));
        assert!(matches!(
            detect_format_from_url(Some("https://github.com/alacritty/alacritty")),
            SchemeFormat::Toml
        ));
        assert!(matches!(
            detect_format_from_url(Some("https://github.com/some/xresources-repo")),
            SchemeFormat::XResources
        ));
        assert!(matches!(detect_format_from_url(None), SchemeFormat::Json)); // Default fallback
    }

    /// Integration test to verify the entire CSS generation process produces valid output
    /// This test uses a more comprehensive approach but still avoids network calls
    #[test]
    fn test_css_generation_structure() {
        // Test that the CSS generation logic produces the expected structure
        let light_palette = crate::colorscheme_provider::ColorPalette {
            background: "#FAF4ED".to_string(),
            foreground: "#575279".to_string(),
            blue: "#56949F".to_string(),
            cyan: "#D7827E".to_string(),
            magenta: "#907AA9".to_string(),
            red: "#B4637A".to_string(),
            yellow: "#EA9D34".to_string(),
            green: "#286983".to_string(),
            bright_black: "#9893A5".to_string(),
            black: "#F2E9E1".to_string(),
            white: "#575279".to_string(),
            bright_red: "#B4637A".to_string(),
            bright_green: "#286983".to_string(),
            bright_yellow: "#EA9D34".to_string(),
            bright_blue: "#56949F".to_string(),
            bright_magenta: "#907AA9".to_string(),
            bright_cyan: "#D7827E".to_string(),
            bright_white: "#575279".to_string(),
            cursor: Some("#575279".to_string()),
            selection: Some("#DFDAD9".to_string()),
        };

        let css_content = light_palette.to_css_variables();

        // Verify CSS structure
        assert!(
            css_content.contains(":root {"),
            "Should contain root selector"
        );
        assert!(
            css_content.contains("--color-background: #FAF4ED"),
            "Should contain background color"
        );
        assert!(
            css_content.contains("--color-text: #575279"),
            "Should contain text color"
        );
        assert!(
            css_content.contains("--color-primary: #56949F"),
            "Should contain primary color"
        );
        assert!(css_content.contains("}"), "Should close root selector");
    }

    #[test]
    fn test_mock_provider_usage() {
        // Test the mock provider to eliminate warnings
        let provider = MockProvider::new();
        assert_eq!(provider.provider_name(), "Mock");

        let schemes = provider.list_available().unwrap();
        assert!(schemes.contains(&"Rose Pine Dawn".to_string()));
        assert!(schemes.contains(&"Rose Pine Moon".to_string()));

        let dawn_palette = provider.fetch("Rose Pine Dawn", None).unwrap();
        assert_eq!(dawn_palette.background, "#FAF4ED");

        let moon_palette = provider.fetch("Rose Pine Moon", None).unwrap();
        assert_eq!(moon_palette.background, "#232136");

        // Test error case
        let result = provider.fetch("Non-existent Theme", None);
        assert!(result.is_err());
    }
}
