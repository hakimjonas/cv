use anyhow::{Context, Result};
use gray_matter::Matter;
use im::Vector;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a static page with front matter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Page title
    pub title: String,
    /// Page layout (default: "page")
    pub layout: String,
    /// Menu label for navigation
    pub menu_label: Option<String>,
    /// Page slug/URL path
    pub slug: String,
    /// HTML content rendered from markdown
    pub content: String,
    /// Optional custom CSS for the page
    pub custom_css: Option<String>,
    /// Optional custom JS for the page
    pub custom_js: Option<String>,
    /// Sort order for menu (lower numbers appear first)
    pub order: Option<i32>,
}

/// Front matter structure for markdown pages
#[derive(Debug, Deserialize)]
struct PageFrontMatter {
    title: String,
    #[serde(default = "default_layout")]
    layout: String,
    menu_label: Option<String>,
    custom_css: Option<String>,
    custom_js: Option<String>,
    order: Option<i32>,
}

fn default_layout() -> String {
    "page".to_string()
}

impl Page {
    /// Create a new page from markdown file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the markdown file
    ///
    /// # Returns
    ///
    /// Result containing the parsed Page or an error
    pub fn from_markdown_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read markdown file: {}", path.display()))?;

        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page")
            .to_string();

        Self::from_markdown(&content, slug)
    }

    /// Parse markdown content with front matter
    ///
    /// # Arguments
    ///
    /// * `content` - Markdown content with optional front matter
    /// * `slug` - URL slug for the page
    ///
    /// # Returns
    ///
    /// Result containing the parsed Page or an error
    pub fn from_markdown(content: &str, slug: String) -> Result<Self> {
        let matter = Matter::<gray_matter::engine::YAML>::new();
        let parsed = matter
            .parse::<Option<PageFrontMatter>>(content)
            .context("Failed to parse page markdown")?;

        // Extract front matter. If front matter is missing, create a default.
        let front_matter = parsed.data.flatten().unwrap_or_else(|| PageFrontMatter {
            title: slug.clone(),
            layout: default_layout(),
            menu_label: None,
            custom_css: None,
            custom_js: None,
            order: None,
        });

        // Convert markdown to HTML
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(&parsed.content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(Page {
            title: front_matter.title,
            layout: front_matter.layout,
            menu_label: front_matter.menu_label,
            slug,
            content: html_output,
            custom_css: front_matter.custom_css,
            custom_js: front_matter.custom_js,
            order: front_matter.order,
        })
    }
}

/// Loads all markdown pages from a directory
///
/// # Arguments
///
/// * `dir_path` - Path to the directory containing markdown files
///
/// # Returns
///
/// Result containing a Vector of Pages or an error
pub fn load_pages_from_directory(dir_path: &Path) -> Result<Vector<Page>> {
    if !dir_path.exists() {
        return Ok(Vector::new());
    }

    let mut pages = Vector::new();

    let entries = fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read pages directory: {}", dir_path.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            match Page::from_markdown_file(&path) {
                Ok(page) => pages.push_back(page),
                Err(e) => {
                    eprintln!("Warning: Failed to load page {}: {}", path.display(), e);
                }
            }
        }
    }

    // Sort pages by order field, then by title
    let mut pages_vec: Vec<_> = pages.into_iter().collect();
    pages_vec.sort_by(|a, b| match (a.order, b.order) {
        (Some(order_a), Some(order_b)) => order_a.cmp(&order_b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.title.cmp(&b.title),
    });

    Ok(pages_vec.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_with_front_matter() {
        let content = r#"---
title: "About Me"
layout: "page"
menu_label: "About"
order: 1
---

# About Me

I'm a data engineer passionate about functional programming."#;

        let page = Page::from_markdown(content, "about".to_string()).unwrap();

        assert_eq!(page.title, "About Me");
        assert_eq!(page.layout, "page");
        assert_eq!(page.menu_label, Some("About".to_string()));
        assert_eq!(page.slug, "about");
        assert_eq!(page.order, Some(1));
        assert!(page.content.contains("<h1>About Me</h1>"));
    }

    #[test]
    fn test_parse_markdown_without_front_matter() {
        let content = "# Simple Page\n\nThis is a simple page without front matter.";

        let page = Page::from_markdown(content, "simple".to_string()).unwrap();

        assert_eq!(page.title, "simple");
        assert_eq!(page.layout, "page");
        assert_eq!(page.menu_label, None);
        assert!(page.content.contains("<h1>Simple Page</h1>"));
    }
}
