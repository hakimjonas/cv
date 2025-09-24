use anyhow::{Context, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use std::fs;

/// Site configuration including menu and navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Site title (defaults to personal name)
    pub title: Option<String>,
    /// Main navigation menu items
    pub menu: Vector<MenuItem>,
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

        Ok(config)
    }

    /// Get the site title, using personal name as fallback
    pub fn get_title(&self, fallback_name: &str) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| fallback_name.to_string())
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
    }

    #[test]
    fn test_site_config_title_fallback() {
        let config = SiteConfig::default();
        assert_eq!(config.get_title("John Doe"), "John Doe");

        let config_with_title = SiteConfig {
            title: Some("My Portfolio".to_string()),
            menu: Vector::new(),
        };
        assert_eq!(config_with_title.get_title("John Doe"), "My Portfolio");
    }
}