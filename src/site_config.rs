use anyhow::{Context, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use std::fs;

/// Valid paper sizes for Typst PDF generation
const VALID_PAPER_SIZES: &[&str] = &["a4", "letter", "legal", "a3", "a5"];

/// Site configuration including menu and navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Site title (defaults to personal name)
    pub title: Option<String>,
    /// Main navigation menu items
    pub menu: Vector<MenuItem>,
    /// Typst PDF generation configuration
    pub typst: Option<TypstConfig>,
    /// Static pages configuration
    pub pages: Option<PagesConfig>,
    /// Blog posts configuration
    pub blog: Option<BlogConfig>,
    /// Font configuration
    pub fonts: Option<FontConfig>,
    /// Colorscheme configuration
    pub colorscheme: Option<ColorschemeConfig>,
}

/// Configuration for blog posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogConfig {
    /// Directory path for blog posts (default: "content/blog")
    pub directory: Option<String>,
    /// Number of posts per page for pagination
    pub posts_per_page: Option<usize>,
    /// Whether to generate RSS feed
    pub rss_feed: Option<bool>,
}

/// Configuration for static markdown pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesConfig {
    /// Directory path for markdown pages (default: "content/pages")
    pub directory: Option<String>,
    /// Whether to automatically add pages to menu
    pub auto_menu: Option<bool>,
}

/// A navigation menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// Display label for the menu item
    pub label: String,
    /// URL path (e.g., "/", "/cv.html", "/about.html")
    pub path: String,
    /// Optional menu item type for special handling
    pub item_type: Option<String>,
    /// Whether this item opens in a new tab
    pub external: Option<bool>,
}

/// Font configuration for website styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Primary font family (e.g., "JetBrainsMono", "FiraCode")
    pub primary: Option<String>,
    /// Fallback font family
    pub fallback: Option<String>,
    /// Font source (e.g., "nerd-fonts", "google-fonts", "local")
    pub source: Option<String>,
    /// Font size base (e.g., "16px", "1rem")
    pub base_size: Option<String>,
    /// Font weight for regular text
    pub weight_regular: Option<u16>,
    /// Font weight for bold text
    pub weight_bold: Option<u16>,
}

/// Colorscheme configuration for website theming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorschemeConfig {
    /// Name of the colorscheme (e.g., "tokyonight", "catppuccin", "dracula")
    pub name: String,
    /// Source of colorscheme definitions (e.g., "ghostty-colors", "base16", "custom")
    pub source: Option<String>,
    /// Optional URL for the colorscheme source
    pub url: Option<String>,
    /// Variant of the colorscheme (e.g., "dark", "light", "storm", "moon")
    pub variant: Option<String>,
    /// Custom color overrides
    pub custom_colors: Option<ColorOverrides>,
}

/// Custom color overrides for fine-tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorOverrides {
    /// Background color
    pub background: Option<String>,
    /// Foreground/text color
    pub foreground: Option<String>,
    /// Primary accent color
    pub primary: Option<String>,
    /// Secondary accent color
    pub secondary: Option<String>,
    /// Error/danger color
    pub error: Option<String>,
    /// Warning color
    pub warning: Option<String>,
    /// Success color
    pub success: Option<String>,
}

/// Typst PDF generation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypstConfig {
    /// Theme configuration for PDF generation
    pub theme: TypstTheme,
    /// Customization options for the theme
    pub customization: TypstCustomization,
}

/// Typst theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypstTheme {
    /// Theme name (e.g., "grotesk-cv", "modern-cv")
    pub name: String,
    /// Theme version (e.g., "1.0.2")
    pub version: String,
    /// Theme source path (e.g., "@preview/grotesk-cv")
    pub source: String,
}

/// Typst customization options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypstCustomization {
    /// Color scheme customization
    pub colors: TypstColors,
    /// Layout customization
    pub layout: TypstLayout,
}

/// Typst color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypstColors {
    /// Background fill color
    pub fill: String,
    /// Accent color for highlights
    pub accent: String,
    /// Light text color
    pub text_light: String,
    /// Medium text color
    pub text_medium: String,
    /// Dark text color
    pub text_dark: String,
}

/// Typst layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypstLayout {
    /// Paper size (e.g., "a4", "letter")
    pub paper_size: String,
    /// Left pane width percentage
    pub left_pane_width: String,
    /// Font family name
    pub font: String,
    /// Font size
    pub font_size: String,
}

impl Default for SiteConfig {
    fn default() -> Self {
        SiteConfig {
            title: None,
            menu: Vector::from(vec![
                MenuItem {
                    label: "CV".to_string(),
                    path: "/cv.html".to_string(),
                    item_type: Some("cv".to_string()),
                    external: None,
                },
                MenuItem {
                    label: "Projects".to_string(),
                    path: "/projects.html".to_string(),
                    item_type: Some("projects".to_string()),
                    external: None,
                },
                MenuItem {
                    label: "Blog".to_string(),
                    path: "/blog.html".to_string(),
                    item_type: Some("blog".to_string()),
                    external: None,
                },
            ]),
            typst: Some(TypstConfig::default()),
            pages: None,
            blog: None,
            fonts: None,
            colorscheme: None,
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        FontConfig {
            primary: Some("Inter".to_string()),
            fallback: Some("system-ui".to_string()),
            source: Some("google-fonts".to_string()),
            base_size: Some("16px".to_string()),
            weight_regular: Some(400),
            weight_bold: Some(700),
        }
    }
}

impl Default for TypstTheme {
    fn default() -> Self {
        TypstTheme {
            name: "grotesk-cv".to_string(),
            version: "1.0.2".to_string(),
            source: "@preview/grotesk-cv".to_string(),
        }
    }
}

impl Default for TypstColors {
    fn default() -> Self {
        TypstColors {
            fill: "#f4f1eb".to_string(),
            accent: "#d4d2cc".to_string(),
            text_light: "#ededef".to_string(),
            text_medium: "#78787e".to_string(),
            text_dark: "#3c3c42".to_string(),
        }
    }
}

impl Default for TypstLayout {
    fn default() -> Self {
        TypstLayout {
            paper_size: "a4".to_string(),
            left_pane_width: "71%".to_string(),
            font: "HK Grotesk".to_string(),
            font_size: "9pt".to_string(),
        }
    }
}

impl SiteConfig {
    /// Load site configuration from JSON file
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the config JSON file
    ///
    /// # Returns
    ///
    /// A Result containing the SiteConfig or an error
    pub fn from_json(config_path: &str) -> Result<Self> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read site config from {}", config_path))?;

        let config: SiteConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse site config JSON: {}", config_path))?;

        // Validate Typst configuration if present
        if let Some(ref typst_config) = config.typst {
            typst_config
                .validate()
                .with_context(|| "Invalid Typst configuration in site config")?;
        }

        Ok(config)
    }

    /// Get Typst configuration with validation
    ///
    /// # Returns
    ///
    /// A Result containing the validated TypstConfig or an error
    pub fn get_typst_config(&self) -> Result<TypstConfig> {
        let config = self
            .typst
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Typst configuration not found"))?;

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Get the site title, falling back to a default if not configured
    ///
    /// # Arguments
    ///
    /// * `fallback` - The fallback title to use if no title is configured
    ///
    /// # Returns
    ///
    /// The configured title or the fallback
    #[allow(dead_code)]
    pub fn get_title(&self, fallback: &str) -> String {
        self.title.as_deref().unwrap_or(fallback).to_string()
    }
}

impl TypstConfig {
    /// Validate Typst configuration
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub fn validate(&self) -> Result<()> {
        // Validate theme
        self.theme.validate()?;

        // Validate customization
        self.customization.validate()?;

        Ok(())
    }
}

impl TypstTheme {
    /// Validate theme configuration
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!(
                "Typst theme name cannot be empty. Please specify a valid theme name (e.g., 'grotesk-cv', 'modern-cv')"
            ));
        }

        if self.version.is_empty() {
            return Err(anyhow::anyhow!(
                "Typst theme version cannot be empty. Please specify a valid version (e.g., '1.0.2', '0.1.0')"
            ));
        }

        if self.source.is_empty() {
            return Err(anyhow::anyhow!(
                "Typst theme source cannot be empty. Please specify a valid source path (e.g., '@preview/grotesk-cv')"
            ));
        }

        // Validate version format (semantic versioning)
        if !self.version.chars().any(|c| c.is_ascii_digit()) {
            return Err(anyhow::anyhow!(
                "Typst theme version '{}' must contain at least one digit. Expected format: '1.0.2' or '0.1.0'",
                self.version
            ));
        }

        Ok(())
    }
}

impl TypstCustomization {
    /// Validate customization configuration
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub fn validate(&self) -> Result<()> {
        self.colors.validate()?;
        self.layout.validate()?;
        Ok(())
    }
}

impl TypstColors {
    /// Validate color configuration
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub fn validate(&self) -> Result<()> {
        // Validate hex color format
        let hex_colors = vec![
            ("fill", &self.fill),
            ("accent", &self.accent),
            ("text_light", &self.text_light),
            ("text_medium", &self.text_medium),
            ("text_dark", &self.text_dark),
        ];

        for (field_name, color) in hex_colors {
            if !color.starts_with('#') || color.len() != 7 {
                return Err(anyhow::anyhow!(
                    "Invalid hex color format '{}' for Typst {} color. Expected format: #RRGGBB (e.g., #ffffff)",
                    color, field_name
                ));
            }

            // Validate hex characters
            if !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(anyhow::anyhow!(
                    "Invalid hex color characters '{}' for Typst {} color. Only hexadecimal digits (0-9, A-F) are allowed",
                    color, field_name
                ));
            }
        }

        Ok(())
    }
}

impl TypstLayout {
    /// Validate layout configuration
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub fn validate(&self) -> Result<()> {
        // Validate paper size
        if !VALID_PAPER_SIZES.contains(&self.paper_size.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid Typst paper size '{}'. Valid options: {:?}",
                self.paper_size,
                VALID_PAPER_SIZES
            ));
        }

        // Validate left pane width (should be percentage)
        if !self.left_pane_width.ends_with('%') {
            return Err(anyhow::anyhow!(
                "Typst left pane width must be a percentage (e.g., '71%'), got: '{}'",
                self.left_pane_width
            ));
        }

        // Validate font size
        if !self.font_size.ends_with("pt") {
            return Err(anyhow::anyhow!(
                "Typst font size must be in points (e.g., '9pt'), got: '{}'",
                self.font_size
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_site_config() {
        let config = SiteConfig::default();
        assert_eq!(config.menu.len(), 3);
        assert_eq!(config.menu[0].label, "CV");
        assert_eq!(config.menu[0].path, "/cv.html");
        assert!(config.typst.is_some());
    }

    #[test]
    fn test_site_config_title_fallback() {
        let config = SiteConfig::default();
        assert_eq!(config.get_title("John Doe"), "John Doe");

        let config_with_title = SiteConfig {
            title: Some("My Portfolio".to_string()),
            menu: Vector::new(),
            typst: None,
            pages: None,
            blog: None,
            fonts: None,
            colorscheme: None,
        };
        assert_eq!(config_with_title.get_title("John Doe"), "My Portfolio");
    }

    #[test]
    fn test_default_typst_config() {
        let config = TypstConfig::default();
        assert_eq!(config.theme.name, "grotesk-cv");
        assert_eq!(config.theme.version, "1.0.2");
        assert_eq!(config.theme.source, "@preview/grotesk-cv");
        assert_eq!(config.customization.colors.fill, "#f4f1eb");
        assert_eq!(config.customization.layout.paper_size, "a4");
    }

    #[test]
    fn test_typst_config_validation_success() {
        let config = TypstConfig::default();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_typst_theme_validation_empty_name() {
        let theme = TypstTheme {
            name: "".to_string(),
            version: "1.0.2".to_string(),
            source: "@preview/grotesk-cv".to_string(),
        };
        let result = theme.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Typst theme name cannot be empty"));
    }

    #[test]
    fn test_typst_theme_validation_empty_version() {
        let theme = TypstTheme {
            name: "grotesk-cv".to_string(),
            version: "".to_string(),
            source: "@preview/grotesk-cv".to_string(),
        };
        let result = theme.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Typst theme version cannot be empty"));
    }

    #[test]
    fn test_typst_theme_validation_invalid_version() {
        let theme = TypstTheme {
            name: "grotesk-cv".to_string(),
            version: "abc".to_string(),
            source: "@preview/grotesk-cv".to_string(),
        };
        let result = theme.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Typst theme version"));
    }

    #[test]
    fn test_typst_colors_validation_success() {
        let colors = TypstColors::default();
        let result = colors.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_typst_colors_validation_invalid_format() {
        let colors = TypstColors {
            fill: "invalid".to_string(),
            accent: "#d4d2cc".to_string(),
            text_light: "#ededef".to_string(),
            text_medium: "#78787e".to_string(),
            text_dark: "#3c3c42".to_string(),
        };
        let result = colors.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid hex color format"));
    }

    #[test]
    fn test_typst_colors_validation_invalid_characters() {
        let colors = TypstColors {
            fill: "#gggggg".to_string(),
            accent: "#d4d2cc".to_string(),
            text_light: "#ededef".to_string(),
            text_medium: "#78787e".to_string(),
            text_dark: "#3c3c42".to_string(),
        };
        let result = colors.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid hex color characters"));
    }

    #[test]
    fn test_typst_layout_validation_success() {
        let layout = TypstLayout::default();
        let result = layout.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_typst_layout_validation_invalid_paper_size() {
        let layout = TypstLayout {
            paper_size: "invalid".to_string(),
            left_pane_width: "71%".to_string(),
            font: "HK Grotesk".to_string(),
            font_size: "9pt".to_string(),
        };
        let result = layout.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Typst paper size"));
    }

    #[test]
    fn test_typst_layout_validation_invalid_width() {
        let layout = TypstLayout {
            paper_size: "a4".to_string(),
            left_pane_width: "71".to_string(), // Missing %
            font: "HK Grotesk".to_string(),
            font_size: "9pt".to_string(),
        };
        let result = layout.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Typst left pane width must be a percentage"));
    }

    #[test]
    fn test_typst_layout_validation_invalid_font_size() {
        let layout = TypstLayout {
            paper_size: "a4".to_string(),
            left_pane_width: "71%".to_string(),
            font: "HK Grotesk".to_string(),
            font_size: "9".to_string(), // Missing pt
        };
        let result = layout.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Typst font size must be in points"));
    }

    #[test]
    fn test_site_config_get_typst_config_success() {
        let config = SiteConfig::default();
        let result = config.get_typst_config();
        assert!(result.is_ok());
        let typst_config = result.unwrap();
        assert_eq!(typst_config.theme.name, "grotesk-cv");
    }

    #[test]
    fn test_site_config_get_typst_config_missing() {
        let config = SiteConfig {
            title: None,
            menu: Vector::new(),
            typst: None,
            pages: None,
            blog: None,
            fonts: None,
            colorscheme: None,
        };
        let result = config.get_typst_config();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Typst configuration not found"));
    }
}
