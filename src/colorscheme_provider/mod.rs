use anyhow::Result;

// Module declarations
pub mod cached;
pub mod github;
pub mod local;
pub mod palette;

// Re-exports for convenience
pub use cached::CachedProvider;
pub use github::{GitHubSchemeProvider, SchemeFormat};
#[allow(unused_imports)] // Public API export
pub use local::LocalSchemeProvider;
pub use palette::{ColorPalette, ToCss};

/// Trait for color scheme providers - the "type class" pattern
pub trait ColorSchemeProvider {
    /// Fetch color scheme from the provider
    fn fetch(&self, name: &str, variant: Option<&str>) -> Result<ColorPalette>;

    /// List available schemes
    #[allow(dead_code)]
    fn list_available(&self) -> Result<Vec<String>>;

    /// Get provider name
    fn provider_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_color_palette_to_css() {
        let palette = ColorPalette {
            black: "#000000".to_string(),
            red: "#ff0000".to_string(),
            green: "#00ff00".to_string(),
            yellow: "#ffff00".to_string(),
            blue: "#0000ff".to_string(),
            magenta: "#ff00ff".to_string(),
            cyan: "#00ffff".to_string(),
            white: "#ffffff".to_string(),
            bright_black: "#808080".to_string(),
            bright_red: "#ff8080".to_string(),
            bright_green: "#80ff80".to_string(),
            bright_yellow: "#ffff80".to_string(),
            bright_blue: "#8080ff".to_string(),
            bright_magenta: "#ff80ff".to_string(),
            bright_cyan: "#80ffff".to_string(),
            bright_white: "#ffffff".to_string(),
            background: "#1a1a1a".to_string(),
            foreground: "#e0e0e0".to_string(),
            cursor: Some("#ffffff".to_string()),
            selection: Some("#444444".to_string()),
        };

        let css = palette.to_css_variables();
        assert!(css.contains(":root {"));
        assert!(css.contains("--color-background: #1a1a1a;"));
        assert!(css.contains("--color-text: #e0e0e0;"));
        assert!(css.contains("--color-cursor: #ffffff;"));
    }

    #[test]
    fn test_cached_provider() {
        let temp_dir = tempdir().unwrap();
        let cache_path = temp_dir.path().to_str().unwrap();

        // Create a simple test provider
        struct TestProvider;
        impl ColorSchemeProvider for TestProvider {
            fn fetch(&self, _name: &str, _variant: Option<&str>) -> Result<ColorPalette> {
                Ok(ColorPalette {
                    black: "#000000".to_string(),
                    red: "#ff0000".to_string(),
                    green: "#00ff00".to_string(),
                    yellow: "#ffff00".to_string(),
                    blue: "#0000ff".to_string(),
                    magenta: "#ff00ff".to_string(),
                    cyan: "#00ffff".to_string(),
                    white: "#ffffff".to_string(),
                    bright_black: "#808080".to_string(),
                    bright_red: "#ff8080".to_string(),
                    bright_green: "#80ff80".to_string(),
                    bright_yellow: "#ffff80".to_string(),
                    bright_blue: "#8080ff".to_string(),
                    bright_magenta: "#ff80ff".to_string(),
                    bright_cyan: "#80ffff".to_string(),
                    bright_white: "#ffffff".to_string(),
                    background: "#000000".to_string(),
                    foreground: "#ffffff".to_string(),
                    cursor: None,
                    selection: None,
                })
            }

            fn list_available(&self) -> Result<Vec<String>> {
                Ok(vec!["test".to_string()])
            }

            fn provider_name(&self) -> &str {
                "Test"
            }
        }

        let cached_provider = CachedProvider::new(TestProvider, cache_path);

        // First fetch - should hit provider
        let _palette = cached_provider
            .fetch("Test Theme", Some("variant"))
            .unwrap();

        // Verify cache file includes variant in name
        let cache_file = format!("{}/Test Theme-variant.json", cache_path);
        assert!(
            std::path::Path::new(&cache_file).exists(),
            "Cache file with variant should exist"
        );

        // Test without variant (should default to "default")
        let _palette = cached_provider.fetch("Test Theme", None).unwrap();

        let default_cache_file = format!("{}/Test Theme-default.json", cache_path);
        assert!(
            std::path::Path::new(&default_cache_file).exists(),
            "Default cache file should exist"
        );
    }

    #[test]
    fn test_github_scheme_provider_creation() {
        let ghostty_provider = GitHubSchemeProvider::ghostty_colors();
        assert_eq!(ghostty_provider.repo, "ghostty-org/ghostty-colors");
        assert_eq!(ghostty_provider.branch, "main");
        assert_eq!(ghostty_provider.path, "themes");
        assert!(matches!(ghostty_provider.format, SchemeFormat::Toml));

        let iterm2_provider = GitHubSchemeProvider::iterm2_schemes();
        assert_eq!(iterm2_provider.repo, "mbadolato/iTerm2-Color-Schemes");
        assert_eq!(iterm2_provider.branch, "master");
        assert_eq!(iterm2_provider.path, "schemes");
        assert!(matches!(iterm2_provider.format, SchemeFormat::ITerm2));

        let base16_provider = GitHubSchemeProvider::base16_schemes();
        assert_eq!(base16_provider.repo, "chriskempson/base16-schemes-source");
        assert_eq!(base16_provider.branch, "main");
        assert_eq!(base16_provider.path, "list.yaml");
        assert!(matches!(base16_provider.format, SchemeFormat::Yaml));
    }

    #[test]
    fn test_local_scheme_provider_creation() {
        let provider = LocalSchemeProvider {
            directory: "/test/dir".to_string(),
            format: SchemeFormat::XResources,
        };

        assert_eq!(provider.provider_name(), "Local");

        // Test error case for unsupported format with temporary file
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.json");
        std::fs::write(&test_file, "test content").unwrap();

        let unsupported_provider = LocalSchemeProvider {
            directory: temp_dir.path().to_str().unwrap().to_string(),
            format: SchemeFormat::Json,
        };

        let result = unsupported_provider.fetch("test.json", None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Local format not yet implemented"));
    }

    #[test]
    fn test_scheme_format_types() {
        // Test that all scheme formats can be created
        let _iterm2 = SchemeFormat::ITerm2;
        let _json = SchemeFormat::Json;
        let _yaml = SchemeFormat::Yaml;
        let _toml = SchemeFormat::Toml;
        let _xresources = SchemeFormat::XResources;

        // The formats should be different
        assert!(!matches!(SchemeFormat::ITerm2, SchemeFormat::Json));
        assert!(!matches!(SchemeFormat::Yaml, SchemeFormat::Toml));
    }
}
