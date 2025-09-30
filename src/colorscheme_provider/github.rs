use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;

use super::{ColorPalette, ColorSchemeProvider};

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

    pub fn parse_xresources(&self, content: &str) -> Result<ColorPalette> {
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
