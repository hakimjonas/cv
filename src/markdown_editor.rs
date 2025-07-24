/*!
 * Markdown editor module
 * This module provides functionality for Markdown editing and preview
 */

use ammonia::clean;
use pulldown_cmark::{Options, Parser, html};
use tracing::{debug, instrument};

/// Configuration for Markdown rendering
#[derive(Debug, Clone)]
pub struct MarkdownConfig {
    /// Whether to enable GitHub flavored Markdown
    pub github_flavored: bool,
    /// Whether to enable tables
    pub enable_tables: bool,
    /// Whether to enable footnotes
    pub enable_footnotes: bool,
    /// Whether to enable strikethrough
    pub enable_strikethrough: bool,
    /// Whether to enable tasklists
    pub enable_tasklists: bool,
    /// Whether to enable smart punctuation
    pub enable_smart_punctuation: bool,
    /// Whether to enable heading attributes
    pub enable_heading_attributes: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            github_flavored: true,
            enable_tables: true,
            enable_footnotes: true,
            enable_strikethrough: true,
            enable_tasklists: true,
            enable_smart_punctuation: true,
            enable_heading_attributes: true,
        }
    }
}

/// Markdown renderer
#[derive(Debug, Clone)]
pub struct MarkdownRenderer {
    config: MarkdownConfig,
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new(MarkdownConfig::default())
    }
}

impl MarkdownRenderer {
    /// Create a new Markdown renderer with the given configuration
    pub fn new(config: MarkdownConfig) -> Self {
        Self { config }
    }

    /// Render Markdown to HTML
    #[instrument(skip(self, markdown), err)]
    pub fn render_to_html(&self, markdown: &str) -> Result<String, anyhow::Error> {
        // Configure parser options based on config
        let mut options = Options::empty();

        if self.config.github_flavored {
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TASKLISTS);
        } else {
            if self.config.enable_tables {
                options.insert(Options::ENABLE_TABLES);
            }
            if self.config.enable_footnotes {
                options.insert(Options::ENABLE_FOOTNOTES);
            }
            if self.config.enable_strikethrough {
                options.insert(Options::ENABLE_STRIKETHROUGH);
            }
            if self.config.enable_tasklists {
                options.insert(Options::ENABLE_TASKLISTS);
            }
        }

        if self.config.enable_smart_punctuation {
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
        }

        if self.config.enable_heading_attributes {
            options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        }

        // Parse the Markdown
        let parser = Parser::new_ext(markdown, options);

        // Render to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Sanitize the HTML
        let clean_html = clean(&html_output);

        debug!("Rendered Markdown to HTML");
        Ok(clean_html)
    }

    /// Sanitize HTML
    #[instrument(skip(self, html))]
    pub fn sanitize_html(&self, html: &str) -> String {
        clean(html)
    }
}

/// Markdown editor state
#[derive(Debug, Clone)]
pub struct MarkdownEditor {
    /// Current Markdown content
    pub content: String,
    /// Rendered HTML preview
    pub preview: String,
    /// Markdown renderer
    renderer: MarkdownRenderer,
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        Self::new(MarkdownConfig::default())
    }
}

impl MarkdownEditor {
    /// Create a new Markdown editor with the given configuration
    pub fn new(config: MarkdownConfig) -> Self {
        let renderer = MarkdownRenderer::new(config);
        Self {
            content: String::new(),
            preview: String::new(),
            renderer,
        }
    }

    /// Set the Markdown content and update the preview
    pub fn set_content(&mut self, content: &str) -> Result<(), anyhow::Error> {
        self.content = content.to_string();
        self.update_preview()?;
        Ok(())
    }

    /// Update the preview based on the current content
    pub fn update_preview(&mut self) -> Result<(), anyhow::Error> {
        self.preview = self.renderer.render_to_html(&self.content)?;
        Ok(())
    }

    /// Get the current Markdown content
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Get the current HTML preview
    pub fn get_preview(&self) -> &str {
        &self.preview
    }
}

/// Utility functions for working with Markdown
pub mod utils {
    use super::*;

    /// Convert Markdown to HTML
    pub fn markdown_to_html(markdown: &str) -> Result<String, anyhow::Error> {
        let renderer: MarkdownRenderer = Default::default();
        renderer.render_to_html(markdown)
    }

    /// Extract a summary from Markdown content
    pub fn extract_summary(markdown: &str, max_length: usize) -> String {
        // Get the first paragraph
        let first_para = markdown
            .lines()
            .skip_while(|line| line.trim().is_empty())
            .take_while(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // Truncate to max_length
        if first_para.len() <= max_length {
            first_para
        } else {
            // Find the last space before max_length
            let truncated = match first_para[..max_length].rfind(' ') {
                Some(pos) => &first_para[..pos],
                None => &first_para[..max_length],
            };

            format!("{truncated}...")
        }
    }
}
