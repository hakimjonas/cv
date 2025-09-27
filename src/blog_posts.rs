use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use gray_matter::Matter;
use im::Vector;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a blog post with front matter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    /// Post title
    pub title: String,
    /// Publication date
    pub date: DateTime<Utc>,
    /// Post tags
    pub tags: Vector<String>,
    /// Post excerpt/summary
    pub excerpt: Option<String>,
    /// Post slug/URL path
    pub slug: String,
    /// HTML content rendered from markdown
    pub content: String,
    /// Reading time in minutes
    pub reading_time: Option<u32>,
    /// Whether the post is published
    pub published: Option<bool>,
    /// Author name (optional, defaults to CV name)
    pub author: Option<String>,
}

/// Front matter structure for blog posts
#[derive(Debug, Deserialize)]
struct BlogFrontMatter {
    title: String,
    date: String,
    tags: Option<Vec<String>>,
    excerpt: Option<String>,
    reading_time: Option<u32>,
    published: Option<bool>,
    author: Option<String>,
}

impl BlogPost {
    /// Create a new blog post from markdown file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the markdown file
    ///
    /// # Returns
    ///
    /// Result containing the parsed BlogPost or an error
    pub fn from_markdown_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read blog post file: {}", path.display()))?;

        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("post")
            .to_string();

        Self::from_markdown(&content, slug)
    }

    /// Parse markdown content with front matter
    ///
    /// # Arguments
    ///
    /// * `content` - Markdown content with optional front matter
    /// * `slug` - URL slug for the post
    ///
    /// # Returns
    ///
    /// Result containing the parsed BlogPost or an error
    pub fn from_markdown(content: &str, slug: String) -> Result<Self> {
        let matter = Matter::<gray_matter::engine::YAML>::new();
        let parsed = matter
            .parse::<BlogFrontMatter>(content)
            .context("Failed to parse blog post markdown")?;

        // Extract front matter, returning an error if it's missing
        let front_matter = parsed.data.context("Blog post requires front matter")?;

        // Parse date
        let date = DateTime::parse_from_rfc3339(&front_matter.date)
            .or_else(|_| {
                // Try parsing as date only (YYYY-MM-DD)
                let date_str = format!("{}T00:00:00Z", front_matter.date);
                DateTime::parse_from_rfc3339(&date_str)
            })
            .context("Failed to parse blog post date")?
            .with_timezone(&Utc);

        // Convert markdown to HTML
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(&parsed.content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Calculate reading time if not provided
        let reading_time = front_matter.reading_time.or_else(|| {
            // Rough estimate: 200 words per minute
            let word_count = parsed.content.split_whitespace().count() as u32;
            Some((word_count / 200).max(1))
        });

        Ok(BlogPost {
            title: front_matter.title,
            date,
            tags: front_matter
                .tags
                .map(|t| t.into_iter().collect())
                .unwrap_or_default(),
            excerpt: front_matter.excerpt,
            slug,
            content: html_output,
            reading_time,
            published: front_matter.published,
            author: front_matter.author,
        })
    }

    /// Check if the post is published
    pub fn is_published(&self) -> bool {
        self.published.unwrap_or(true)
    }
}

/// Loads all blog posts from a directory
///
/// # Arguments
///
/// * `dir_path` - Path to the directory containing markdown files
///
/// # Returns
///
/// Result containing a Vector of BlogPosts or an error
pub fn load_posts_from_directory(dir_path: &Path) -> Result<Vector<BlogPost>> {
    if !dir_path.exists() {
        return Ok(Vector::new());
    }

    let mut posts = Vector::new();

    let entries = fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read blog directory: {}", dir_path.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            match BlogPost::from_markdown_file(&path) {
                Ok(post) => {
                    // Only include published posts
                    if post.is_published() {
                        posts.push_back(post);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load blog post {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }

    // Sort posts by date (newest first)
    let mut posts_vec: Vec<_> = posts.into_iter().collect();
    posts_vec.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(posts_vec.into_iter().collect())
}

/// Group blog posts by tags
///
/// # Arguments
///
/// * `posts` - Vector of blog posts
///
/// # Returns
///
/// HashMap of tag to Vector of posts with that tag
pub fn group_posts_by_tags(posts: &Vector<BlogPost>) -> im::HashMap<String, Vector<BlogPost>> {
    let mut tag_map = im::HashMap::new();

    for post in posts.iter() {
        for tag in post.tags.iter() {
            tag_map
                .entry(tag.clone())
                .or_insert_with(Vector::new)
                .push_back(post.clone());
        }
    }

    tag_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blog_post_with_front_matter() {
        let content = r#"---
title: "Building a Functional Language in Rust"
date: "2024-01-15"
tags: ["rust", "compilers", "functional-programming"]
excerpt: "After years of working with Scala..."
---

# Building a Functional Language in Rust

After years of working with Scala and functional programming..."#;

        let post = BlogPost::from_markdown(content, "functional-lang".to_string()).unwrap();

        assert_eq!(post.title, "Building a Functional Language in Rust");
        assert_eq!(post.slug, "functional-lang");
        assert_eq!(post.tags.len(), 3);
        assert!(post.tags.contains(&"rust".to_string()));
        assert_eq!(
            post.excerpt,
            Some("After years of working with Scala...".to_string())
        );
        assert!(post
            .content
            .contains("<h1>Building a Functional Language in Rust</h1>"));
    }

    #[test]
    fn test_blog_post_reading_time() {
        let content = r#"---
title: "Test Post"
date: "2024-01-15"
---

This is a test post with approximately 50 words of content.
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.
Nisi ut aliquip ex ea commodo consequat."#;

        let post = BlogPost::from_markdown(content, "test".to_string()).unwrap();

        // Should calculate reading time automatically
        assert_eq!(post.reading_time, Some(1));
    }

    #[test]
    fn test_unpublished_post() {
        let content = r#"---
title: "Draft Post"
date: "2024-01-15"
published: false
---

This is a draft post."#;

        let post = BlogPost::from_markdown(content, "draft".to_string()).unwrap();

        assert!(!post.is_published());
    }
}
