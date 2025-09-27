use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

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

/// Universal color palette that all providers convert to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // Base colors (0-7)
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,

    // Bright colors (8-15)
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,

    // Special colors
    pub background: String,
    pub foreground: String,
    pub cursor: Option<String>,
    pub selection: Option<String>,
}

/// Trait for converting color palettes to CSS
pub trait ToCss {
    fn to_css_variables(&self) -> String;
    fn to_theme_css(&self, theme_class: &str) -> String;
}

impl ToCss for ColorPalette {
    fn to_css_variables(&self) -> String {
        let mut css = String::from(":root {\n");

        // Map terminal colors to CSS variables
        css.push_str(&format!("  --color-background: {};\n", self.background));
        css.push_str(&format!("  --color-text: {};\n", self.foreground));
        css.push_str(&format!("  --color-primary: {};\n", self.blue));
        css.push_str(&format!("  --color-secondary: {};\n", self.cyan));
        css.push_str(&format!("  --color-accent: {};\n", self.magenta));
        css.push_str(&format!("  --color-error: {};\n", self.red));
        css.push_str(&format!("  --color-warning: {};\n", self.yellow));
        css.push_str(&format!("  --color-success: {};\n", self.green));

        // Surface colors
        css.push_str(&format!("  --color-surface: {};\n", self.bright_black));
        css.push_str(&format!("  --color-overlay: {};\n", self.black));
        css.push_str(&format!("  --color-muted: {};\n", self.bright_black));
        css.push_str(&format!("  --color-subtle: {};\n", self.white));

        // Additional mappings
        css.push_str(&format!("  --color-border: {};\n", self.bright_black));
        css.push_str(&format!("  --color-text-light: {};\n", self.white));
        css.push_str(&format!("  --color-background-light: {};\n", self.black));
        css.push_str(&format!(
            "  --color-card-background: {};\n",
            self.background
        ));

        if let Some(cursor) = &self.cursor {
            css.push_str(&format!("  --color-cursor: {};\n", cursor));
        }
        if let Some(selection) = &self.selection {
            css.push_str(&format!("  --color-selection: {};\n", selection));
        }

        css.push_str("}\n");
        css
    }

    fn to_theme_css(&self, theme_class: &str) -> String {
        format!(
            "{} {{\n  /* Generated from terminal color scheme */\n{}}}\n",
            theme_class,
            self.to_css_variables()
                .lines()
                .skip(1) // Skip ":root {"
                .take_while(|line| !line.contains('}'))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// GitHub-hosted color scheme provider (e.g., iTerm2-Color-Schemes)
pub struct GitHubSchemeProvider {
    pub repo: String,
    pub branch: String,
    pub path: String,
    pub format: SchemeFormat,
}

#[derive(Debug, Clone)]
pub enum SchemeFormat {
    ITerm2,     // .itermcolors plist format
    Json,       // JSON format
    Yaml,       // YAML format
    Toml,       // TOML format (like Alacritty)
    XResources, // Xresources format
}

impl GitHubSchemeProvider {
    pub fn new(repo: &str, format: SchemeFormat) -> Self {
        GitHubSchemeProvider {
            repo: repo.to_string(),
            branch: "master".to_string(),
            path: "".to_string(),
            format,
        }
    }

    pub fn ghostty_colors() -> Self {
        GitHubSchemeProvider {
            repo: "ghostty-org/ghostty-colors".to_string(),
            branch: "main".to_string(),
            path: "themes".to_string(),
            format: SchemeFormat::Toml,
        }
    }

    pub fn iterm2_schemes() -> Self {
        GitHubSchemeProvider {
            repo: "mbadolato/iTerm2-Color-Schemes".to_string(),
            branch: "master".to_string(),
            path: "schemes".to_string(),
            format: SchemeFormat::ITerm2,
        }
    }

    pub fn base16_schemes() -> Self {
        GitHubSchemeProvider {
            repo: "chriskempson/base16-schemes-source".to_string(),
            branch: "main".to_string(),
            path: "list.yaml".to_string(),
            format: SchemeFormat::Yaml,
        }
    }
}

impl ColorSchemeProvider for GitHubSchemeProvider {
    fn fetch(&self, name: &str, _variant: Option<&str>) -> Result<ColorPalette> {
        let file_name = match self.format {
            SchemeFormat::ITerm2 => format!("{}.itermcolors", name),
            SchemeFormat::Json => format!("{}.json", name),
            SchemeFormat::Yaml => format!("{}.yaml", name),
            SchemeFormat::Toml => format!("{}.toml", name),
            SchemeFormat::XResources => format!("{}.Xresources", name),
        };

        let path = if self.path.is_empty() {
            file_name
        } else {
            format!("{}/{}", self.path, file_name)
        };

        // Use gh CLI to fetch the raw file
        let output = Command::new("gh")
            .args([
                "api",
                &format!("/repos/{}/contents/{}?ref={}", self.repo, path, self.branch),
                "--jq",
                ".content",
            ])
            .output()
            .context("Failed to fetch color scheme from GitHub")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch scheme: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let base64_content =
            String::from_utf8(output.stdout).context("Invalid UTF-8 in response")?;

        // Decode base64 (remove newlines and whitespace)
        use base64::Engine;
        let cleaned_base64: String = base64_content
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        let content = base64::engine::general_purpose::STANDARD
            .decode(cleaned_base64)
            .context("Failed to decode base64 content")?;

        let content_str = String::from_utf8(content).context("Invalid UTF-8 in decoded content")?;

        // Parse based on format
        match self.format {
            SchemeFormat::ITerm2 => self.parse_iterm2(&content_str),
            SchemeFormat::Json => self.parse_json(&content_str),
            SchemeFormat::Toml => self.parse_toml(&content_str),
            SchemeFormat::Yaml => self.parse_yaml(&content_str),
            SchemeFormat::XResources => self.parse_xresources(&content_str),
        }
    }

    fn list_available(&self) -> Result<Vec<String>> {
        // Use gh CLI to list files in the directory
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "/repos/{}/contents/{}?ref={}",
                    self.repo, self.path, self.branch
                ),
                "--jq",
                ".[].name",
            ])
            .output()
            .context("Failed to list schemes from GitHub")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to list schemes: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let names = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in response")?
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.ends_with(".itermcolors")
                    || line.ends_with(".json")
                    || line.ends_with(".yaml")
                    || line.ends_with(".toml")
                {
                    Some(line.rsplit('.').nth(1).unwrap_or(line).to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(names)
    }

    fn provider_name(&self) -> &str {
        "GitHub"
    }
}

impl GitHubSchemeProvider {
    fn parse_iterm2(&self, content: &str) -> Result<ColorPalette> {
        // Parse the plist XML format
        // iTerm2 colors are in a plist with color components as real numbers 0-1

        fn extract_color(content: &str, color_name: &str) -> Option<String> {
            // Find the color key
            let key_pattern = format!("<key>{}</key>", color_name);
            let start = content.find(&key_pattern)?;
            let section = &content[start..];

            // Extract RGB components
            let mut components = Vec::new();
            let component_names = ["Red Component", "Green Component", "Blue Component"];

            for component in &component_names {
                let key = format!("<key>{}</key>", component);
                if let Some(idx) = section.find(&key) {
                    let after_key = &section[idx + key.len()..];
                    if let Some(real_start) = after_key.find("<real>") {
                        let real_content = &after_key[real_start + 6..];
                        if let Some(real_end) = real_content.find("</real>") {
                            let value = &real_content[..real_end];
                            if let Ok(num) = value.parse::<f64>() {
                                components.push((num * 255.0) as u8);
                            }
                        }
                    }
                }
            }

            if components.len() == 3 {
                Some(format!(
                    "#{:02X}{:02X}{:02X}",
                    components[0], components[1], components[2]
                ))
            } else {
                None
            }
        }

        // Extract all ANSI colors and special colors
        let black = extract_color(content, "Ansi 0 Color").unwrap_or_else(|| "#000000".to_string());
        let red = extract_color(content, "Ansi 1 Color").unwrap_or_else(|| "#800000".to_string());
        let green = extract_color(content, "Ansi 2 Color").unwrap_or_else(|| "#008000".to_string());
        let yellow =
            extract_color(content, "Ansi 3 Color").unwrap_or_else(|| "#808000".to_string());
        let blue = extract_color(content, "Ansi 4 Color").unwrap_or_else(|| "#000080".to_string());
        let magenta =
            extract_color(content, "Ansi 5 Color").unwrap_or_else(|| "#800080".to_string());
        let cyan = extract_color(content, "Ansi 6 Color").unwrap_or_else(|| "#008080".to_string());
        let white = extract_color(content, "Ansi 7 Color").unwrap_or_else(|| "#c0c0c0".to_string());

        let bright_black =
            extract_color(content, "Ansi 8 Color").unwrap_or_else(|| "#808080".to_string());
        let bright_red =
            extract_color(content, "Ansi 9 Color").unwrap_or_else(|| "#ff0000".to_string());
        let bright_green =
            extract_color(content, "Ansi 10 Color").unwrap_or_else(|| "#00ff00".to_string());
        let bright_yellow =
            extract_color(content, "Ansi 11 Color").unwrap_or_else(|| "#ffff00".to_string());
        let bright_blue =
            extract_color(content, "Ansi 12 Color").unwrap_or_else(|| "#0000ff".to_string());
        let bright_magenta =
            extract_color(content, "Ansi 13 Color").unwrap_or_else(|| "#ff00ff".to_string());
        let bright_cyan =
            extract_color(content, "Ansi 14 Color").unwrap_or_else(|| "#00ffff".to_string());
        let bright_white =
            extract_color(content, "Ansi 15 Color").unwrap_or_else(|| "#ffffff".to_string());

        let background =
            extract_color(content, "Background Color").unwrap_or_else(|| black.clone());
        let foreground =
            extract_color(content, "Foreground Color").unwrap_or_else(|| white.clone());
        let cursor = extract_color(content, "Cursor Color");
        let selection = extract_color(content, "Selection Color");

        Ok(ColorPalette {
            black,
            red,
            green,
            yellow,
            blue,
            magenta,
            cyan,
            white,
            bright_black,
            bright_red,
            bright_green,
            bright_yellow,
            bright_blue,
            bright_magenta,
            bright_cyan,
            bright_white,
            background,
            foreground,
            cursor,
            selection,
        })
    }

    fn parse_json(&self, content: &str) -> Result<ColorPalette> {
        let json: serde_json::Value =
            serde_json::from_str(content).context("Failed to parse JSON color scheme")?;

        // Convert JSON structure to ColorPalette
        // This depends on the specific JSON schema used
        self.json_to_palette(&json)
    }

    fn parse_toml(&self, content: &str) -> Result<ColorPalette> {
        let toml: toml::Value =
            toml::from_str(content).context("Failed to parse TOML color scheme")?;

        // For Ghostty format
        if self.repo.contains("ghostty") {
            return self.parse_ghostty_toml(&toml);
        }

        // Generic TOML parsing
        self.toml_to_palette(&toml)
    }

    fn parse_yaml(&self, content: &str) -> Result<ColorPalette> {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(content).context("Failed to parse YAML color scheme")?;

        self.yaml_to_palette(&yaml)
    }

    fn parse_xresources(&self, content: &str) -> Result<ColorPalette> {
        let mut colors = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('!') || line.starts_with('#') || line.is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().replace("*.", "");
                let value = value.trim();
                colors.insert(key, value.to_string());
            }
        }

        Ok(ColorPalette {
            black: colors
                .get("color0")
                .unwrap_or(&"#000000".to_string())
                .clone(),
            red: colors
                .get("color1")
                .unwrap_or(&"#ff0000".to_string())
                .clone(),
            green: colors
                .get("color2")
                .unwrap_or(&"#00ff00".to_string())
                .clone(),
            yellow: colors
                .get("color3")
                .unwrap_or(&"#ffff00".to_string())
                .clone(),
            blue: colors
                .get("color4")
                .unwrap_or(&"#0000ff".to_string())
                .clone(),
            magenta: colors
                .get("color5")
                .unwrap_or(&"#ff00ff".to_string())
                .clone(),
            cyan: colors
                .get("color6")
                .unwrap_or(&"#00ffff".to_string())
                .clone(),
            white: colors
                .get("color7")
                .unwrap_or(&"#ffffff".to_string())
                .clone(),
            bright_black: colors
                .get("color8")
                .unwrap_or(&"#808080".to_string())
                .clone(),
            bright_red: colors
                .get("color9")
                .unwrap_or(&"#ff8080".to_string())
                .clone(),
            bright_green: colors
                .get("color10")
                .unwrap_or(&"#80ff80".to_string())
                .clone(),
            bright_yellow: colors
                .get("color11")
                .unwrap_or(&"#ffff80".to_string())
                .clone(),
            bright_blue: colors
                .get("color12")
                .unwrap_or(&"#8080ff".to_string())
                .clone(),
            bright_magenta: colors
                .get("color13")
                .unwrap_or(&"#ff80ff".to_string())
                .clone(),
            bright_cyan: colors
                .get("color14")
                .unwrap_or(&"#80ffff".to_string())
                .clone(),
            bright_white: colors
                .get("color15")
                .unwrap_or(&"#ffffff".to_string())
                .clone(),
            background: colors
                .get("background")
                .unwrap_or(&"#000000".to_string())
                .clone(),
            foreground: colors
                .get("foreground")
                .unwrap_or(&"#ffffff".to_string())
                .clone(),
            cursor: colors.get("cursor").cloned(),
            selection: colors.get("selection").cloned(),
        })
    }

    fn json_to_palette(&self, _json: &serde_json::Value) -> Result<ColorPalette> {
        // Placeholder - would need to handle specific JSON schema
        Err(anyhow::anyhow!("JSON parsing not yet fully implemented"))
    }

    fn toml_to_palette(&self, _toml: &toml::Value) -> Result<ColorPalette> {
        // Placeholder
        Err(anyhow::anyhow!("TOML parsing not yet fully implemented"))
    }

    fn yaml_to_palette(&self, _yaml: &serde_yaml::Value) -> Result<ColorPalette> {
        // Placeholder
        Err(anyhow::anyhow!("YAML parsing not yet fully implemented"))
    }

    fn parse_ghostty_toml(&self, toml: &toml::Value) -> Result<ColorPalette> {
        // Parse Ghostty-specific TOML format
        let get_color = |name: &str| -> String {
            toml.get(name)
                .and_then(|v| v.as_str())
                .unwrap_or("#000000")
                .to_string()
        };

        Ok(ColorPalette {
            black: get_color("palette_black"),
            red: get_color("palette_red"),
            green: get_color("palette_green"),
            yellow: get_color("palette_yellow"),
            blue: get_color("palette_blue"),
            magenta: get_color("palette_magenta"),
            cyan: get_color("palette_cyan"),
            white: get_color("palette_white"),
            bright_black: get_color("palette_bright_black"),
            bright_red: get_color("palette_bright_red"),
            bright_green: get_color("palette_bright_green"),
            bright_yellow: get_color("palette_bright_yellow"),
            bright_blue: get_color("palette_bright_blue"),
            bright_magenta: get_color("palette_bright_magenta"),
            bright_cyan: get_color("palette_bright_cyan"),
            bright_white: get_color("palette_bright_white"),
            background: get_color("background"),
            foreground: get_color("foreground"),
            cursor: Some(get_color("cursor_color")),
            selection: Some(get_color("selection_background")),
        })
    }
}

/// Local file system provider for color schemes
#[allow(dead_code)]
pub struct LocalSchemeProvider {
    pub directory: String,
    pub format: SchemeFormat,
}

impl ColorSchemeProvider for LocalSchemeProvider {
    fn fetch(&self, name: &str, _variant: Option<&str>) -> Result<ColorPalette> {
        let file_path = format!("{}/{}", self.directory, name);
        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read scheme from {}", file_path))?;

        match self.format {
            SchemeFormat::XResources => {
                GitHubSchemeProvider::new("", self.format.clone()).parse_xresources(&content)
            }
            _ => Err(anyhow::anyhow!("Local format not yet implemented")),
        }
    }

    fn list_available(&self) -> Result<Vec<String>> {
        let entries =
            std::fs::read_dir(&self.directory).context("Failed to read schemes directory")?;

        let schemes = entries
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|e| e.file_name().to_str().map(|s| s.to_string()))
            })
            .collect();

        Ok(schemes)
    }

    fn provider_name(&self) -> &str {
        "Local"
    }
}

/// Cache wrapper for any provider
pub struct CachedProvider<P: ColorSchemeProvider> {
    provider: P,
    cache_dir: String,
}

impl<P: ColorSchemeProvider> CachedProvider<P> {
    pub fn new(provider: P, cache_dir: &str) -> Self {
        CachedProvider {
            provider,
            cache_dir: cache_dir.to_string(),
        }
    }
}

impl<P: ColorSchemeProvider> ColorSchemeProvider for CachedProvider<P> {
    fn fetch(&self, name: &str, variant: Option<&str>) -> Result<ColorPalette> {
        // Check cache first
        let cache_key = format!("{}-{}", name, variant.unwrap_or("default"));
        let cache_path = format!("{}/{}.json", self.cache_dir, cache_key);

        if let Ok(cached) = std::fs::read_to_string(&cache_path) {
            if let Ok(palette) = serde_json::from_str(&cached) {
                return Ok(palette);
            }
        }

        // Fetch from provider
        let palette = self.provider.fetch(name, variant)?;

        // Save to cache
        if let Ok(json) = serde_json::to_string_pretty(&palette) {
            std::fs::create_dir_all(&self.cache_dir).ok();
            std::fs::write(cache_path, json).ok();
        }

        Ok(palette)
    }

    fn list_available(&self) -> Result<Vec<String>> {
        self.provider.list_available()
    }

    fn provider_name(&self) -> &str {
        self.provider.provider_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Mock provider for testing
    struct MockProvider {
        name: String,
        schemes: std::collections::HashMap<String, ColorPalette>,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            let mut schemes = std::collections::HashMap::new();

            // Add a test scheme
            schemes.insert(
                "Test Theme".to_string(),
                ColorPalette {
                    black: "#000000".to_string(),
                    red: "#FF0000".to_string(),
                    green: "#00FF00".to_string(),
                    yellow: "#FFFF00".to_string(),
                    blue: "#0000FF".to_string(),
                    magenta: "#FF00FF".to_string(),
                    cyan: "#00FFFF".to_string(),
                    white: "#FFFFFF".to_string(),
                    bright_black: "#808080".to_string(),
                    bright_red: "#FF8080".to_string(),
                    bright_green: "#80FF80".to_string(),
                    bright_yellow: "#FFFF80".to_string(),
                    bright_blue: "#8080FF".to_string(),
                    bright_magenta: "#FF80FF".to_string(),
                    bright_cyan: "#80FFFF".to_string(),
                    bright_white: "#FFFFFF".to_string(),
                    background: "#000000".to_string(),
                    foreground: "#FFFFFF".to_string(),
                    cursor: Some("#FFFFFF".to_string()),
                    selection: Some("#404040".to_string()),
                },
            );

            MockProvider {
                name: name.to_string(),
                schemes,
            }
        }
    }

    impl ColorSchemeProvider for MockProvider {
        fn fetch(&self, name: &str, _variant: Option<&str>) -> Result<ColorPalette> {
            self.schemes
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Scheme '{}' not found", name))
        }

        fn list_available(&self) -> Result<Vec<String>> {
            Ok(self.schemes.keys().cloned().collect())
        }

        fn provider_name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_color_palette_to_css_variables() {
        let palette = ColorPalette {
            black: "#000000".to_string(),
            red: "#FF0000".to_string(),
            green: "#00FF00".to_string(),
            yellow: "#FFFF00".to_string(),
            blue: "#0000FF".to_string(),
            magenta: "#FF00FF".to_string(),
            cyan: "#00FFFF".to_string(),
            white: "#FFFFFF".to_string(),
            bright_black: "#808080".to_string(),
            bright_red: "#FF8080".to_string(),
            bright_green: "#80FF80".to_string(),
            bright_yellow: "#FFFF80".to_string(),
            bright_blue: "#8080FF".to_string(),
            bright_magenta: "#FF80FF".to_string(),
            bright_cyan: "#80FFFF".to_string(),
            bright_white: "#FFFFFF".to_string(),
            background: "#1a1a1a".to_string(),
            foreground: "#f0f0f0".to_string(),
            cursor: Some("#ffffff".to_string()),
            selection: Some("#404040".to_string()),
        };

        let css = palette.to_css_variables();

        assert!(css.contains(":root {"), "Should contain root selector");
        assert!(
            css.contains("--color-background: #1a1a1a"),
            "Should contain background color"
        );
        assert!(
            css.contains("--color-text: #f0f0f0"),
            "Should contain text color"
        );
        assert!(
            css.contains("--color-primary: #0000FF"),
            "Should contain primary color mapped to blue"
        );
        assert!(
            css.contains("--color-secondary: #00FFFF"),
            "Should contain secondary color mapped to cyan"
        );
        assert!(
            css.contains("--color-accent: #FF00FF"),
            "Should contain accent color mapped to magenta"
        );
        assert!(
            css.contains("--color-cursor: #ffffff"),
            "Should contain cursor color"
        );
        assert!(
            css.contains("--color-selection: #404040"),
            "Should contain selection color"
        );
        assert!(css.contains("}"), "Should close root selector");
    }

    #[test]
    fn test_color_palette_to_theme_css() {
        let palette = ColorPalette {
            black: "#000000".to_string(),
            red: "#FF0000".to_string(),
            green: "#00FF00".to_string(),
            yellow: "#FFFF00".to_string(),
            blue: "#0000FF".to_string(),
            magenta: "#FF00FF".to_string(),
            cyan: "#00FFFF".to_string(),
            white: "#FFFFFF".to_string(),
            bright_black: "#808080".to_string(),
            bright_red: "#FF8080".to_string(),
            bright_green: "#80FF80".to_string(),
            bright_yellow: "#FFFF80".to_string(),
            bright_blue: "#8080FF".to_string(),
            bright_magenta: "#FF80FF".to_string(),
            bright_cyan: "#80FFFF".to_string(),
            bright_white: "#FFFFFF".to_string(),
            background: "#1a1a1a".to_string(),
            foreground: "#f0f0f0".to_string(),
            cursor: None,
            selection: None,
        };

        let css = palette.to_theme_css(".theme-dark");

        assert!(
            css.contains(".theme-dark {"),
            "Should contain theme class selector"
        );
        assert!(
            css.contains("--color-background: #1a1a1a"),
            "Should contain background color"
        );
        assert!(
            css.contains("--color-text: #f0f0f0"),
            "Should contain text color"
        );
        assert!(!css.contains(":root"), "Should not contain root selector");
    }

    #[test]
    fn test_mock_provider_basic_functionality() {
        let provider = MockProvider::new("Test Provider");

        assert_eq!(provider.provider_name(), "Test Provider");

        let schemes = provider.list_available().unwrap();
        assert!(schemes.contains(&"Test Theme".to_string()));

        let palette = provider.fetch("Test Theme", None).unwrap();
        assert_eq!(palette.background, "#000000");
        assert_eq!(palette.foreground, "#FFFFFF");

        // Test missing scheme
        let result = provider.fetch("Non-existent Theme", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_cached_provider_functionality() {
        let temp_dir = tempdir().unwrap();
        let cache_path = temp_dir.path().to_str().unwrap();

        let base_provider = MockProvider::new("Base Provider");
        let cached_provider = CachedProvider::new(base_provider, cache_path);

        // First fetch should go to base provider
        let palette1 = cached_provider.fetch("Test Theme", None).unwrap();
        assert_eq!(palette1.background, "#000000");

        // Verify cache file was created
        let cache_file = format!("{}/Test Theme-default.json", cache_path);
        assert!(
            std::path::Path::new(&cache_file).exists(),
            "Cache file should exist"
        );

        // Second fetch should use cache
        let palette2 = cached_provider.fetch("Test Theme", None).unwrap();
        assert_eq!(palette2.background, "#000000");
        assert_eq!(palette1.foreground, palette2.foreground);

        // Provider name should pass through
        assert_eq!(cached_provider.provider_name(), "Base Provider");
    }

    #[test]
    fn test_cached_provider_variant_handling() {
        let temp_dir = tempdir().unwrap();
        let cache_path = temp_dir.path().to_str().unwrap();

        let base_provider = MockProvider::new("Base Provider");
        let cached_provider = CachedProvider::new(base_provider, cache_path);

        // Test with variant
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
