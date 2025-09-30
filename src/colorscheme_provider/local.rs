use anyhow::{Context, Result};

use super::github::{GitHubSchemeProvider, SchemeFormat};
use super::{ColorPalette, ColorSchemeProvider};

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
