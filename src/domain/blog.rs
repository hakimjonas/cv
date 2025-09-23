//! Blog domain models
//!
//! This module contains unified blog-related domain entities.
//! All blog operations should use these types as the single source of truth.

use anyhow::{Context, Result};
use im::{HashMap, Vector};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Unique identifier for blog posts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PostId(pub i64);

/// Unique identifier for tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagId(pub i64);

/// Unique identifier for users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

/// Content format for blog posts
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[schema(description = "Format of the blog post content")]
pub enum ContentFormat {
    /// HTML content
    HTML,
    /// Markdown content
    Markdown,
}

impl Default for ContentFormat {
    fn default() -> Self {
        Self::HTML
    }
}

/// Represents a blog post tag
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Unique identifier for the tag (None for new tags)
    pub id: Option<TagId>,

    /// Display name of the tag
    pub name: String,

    /// URL-friendly version of the name
    pub slug: String,
}

impl Tag {
    /// Creates a new tag with auto-generated slug
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            slug: generate_slug(name),
        }
    }

    /// Creates a tag with a specific slug
    pub fn with_slug(name: &str, slug: &str) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            slug: slug.to_string(),
        }
    }
}

/// Represents a blog post with immutable data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlogPost {
    /// Unique identifier for the post (None for new posts)
    pub id: Option<PostId>,

    /// Title of the blog post
    pub title: String,

    /// URL-friendly version of the title
    pub slug: String,

    /// Publication date in ISO format
    pub date: String,

    /// ID of the user who authored the post
    pub user_id: Option<UserId>,

    /// Display name of the author (for backward compatibility)
    pub author: String,

    /// Brief summary of the post
    pub excerpt: String,

    /// Full content of the post
    pub content: String,

    /// Format of the content
    pub content_format: ContentFormat,

    /// Whether the post is published
    pub published: bool,

    /// Whether the post is featured
    pub featured: bool,

    /// Optional featured image URL
    pub image: Option<String>,

    /// Tags associated with the post
    pub tags: Vector<Tag>,

    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}

impl BlogPost {
    /// Creates a new empty blog post with default values
    pub fn new() -> Self {
        Self {
            id: None,
            title: String::new(),
            slug: String::new(),
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            user_id: None,
            author: String::new(),
            excerpt: String::new(),
            content: String::new(),
            content_format: ContentFormat::default(),
            published: false,
            featured: false,
            image: None,
            tags: Vector::new(),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new blog post with the specified title and auto-generated slug
    pub fn with_title(title: &str) -> Self {
        Self {
            title: title.to_string(),
            slug: generate_slug(title),
            ..Self::new()
        }
    }

    /// Builder pattern methods for immutable updates
    pub fn with_content(self, content: &str) -> Self {
        Self {
            content: content.to_string(),
            ..self
        }
    }

    pub fn with_content_format(self, format: ContentFormat) -> Self {
        Self {
            content_format: format,
            ..self
        }
    }

    pub fn with_excerpt(self, excerpt: &str) -> Self {
        Self {
            excerpt: excerpt.to_string(),
            ..self
        }
    }

    pub fn with_author(self, author: &str) -> Self {
        Self {
            author: author.to_string(),
            ..self
        }
    }

    pub fn with_user_id(self, user_id: UserId) -> Self {
        Self {
            user_id: Some(user_id),
            ..self
        }
    }

    pub fn with_published(self, published: bool) -> Self {
        Self { published, ..self }
    }

    pub fn with_featured(self, featured: bool) -> Self {
        Self { featured, ..self }
    }

    pub fn with_image(self, image: Option<String>) -> Self {
        Self { image, ..self }
    }

    pub fn with_tags(self, tags: Vector<Tag>) -> Self {
        Self { tags, ..self }
    }

    pub fn with_added_tag(self, tag: Tag) -> Self {
        let mut new_tags = self.tags.clone();
        new_tags.push_back(tag);
        Self {
            tags: new_tags,
            ..self
        }
    }

    pub fn with_metadata(self, metadata: HashMap<String, String>) -> Self {
        Self { metadata, ..self }
    }

    pub fn with_metadata_entry(self, key: &str, value: &str) -> Self {
        Self {
            metadata: self.metadata.update(key.to_string(), value.to_string()),
            ..self
        }
    }

    /// Renders the content based on its format
    ///
    /// If the content is in Markdown format, it will be converted to HTML.
    /// If the content is already in HTML format, it will be returned as is.
    pub fn render_content(&self) -> Result<String> {
        match self.content_format {
            ContentFormat::HTML => Ok(self.content.clone()),
            ContentFormat::Markdown => {
                crate::markdown_editor::utils::markdown_to_html(&self.content)
                    .context("Failed to render Markdown content")
            }
        }
    }

    /// Validates the blog post data
    pub fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(anyhow::anyhow!("Title cannot be empty"));
        }

        if self.slug.trim().is_empty() {
            return Err(anyhow::anyhow!("Slug cannot be empty"));
        }

        if self.content.trim().is_empty() {
            return Err(anyhow::anyhow!("Content cannot be empty"));
        }

        if self.author.trim().is_empty() {
            return Err(anyhow::anyhow!("Author cannot be empty"));
        }

        // Validate date format
        chrono::NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .context("Invalid date format, expected YYYY-MM-DD")?;

        Ok(())
    }
}

impl Default for BlogPost {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a URL-friendly slug from text
fn generate_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
        assert_eq!(
            generate_slug("Rust & Web Development"),
            "rust-web-development"
        );
        assert_eq!(generate_slug("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(
            generate_slug("Special!@#$%Characters"),
            "special-characters"
        );
    }

    #[test]
    fn test_blog_post_builder() {
        let post = BlogPost::with_title("Test Post")
            .with_content("Test content")
            .with_author("John Doe")
            .with_published(true);

        assert_eq!(post.title, "Test Post");
        assert_eq!(post.slug, "test-post");
        assert_eq!(post.content, "Test content");
        assert_eq!(post.author, "John Doe");
        assert!(post.published);
    }

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Technology");
        assert_eq!(tag.name, "Technology");
        assert_eq!(tag.slug, "technology");
        assert!(tag.id.is_none());
    }

    #[test]
    fn test_blog_post_validation() {
        let valid_post = BlogPost::with_title("Valid Post")
            .with_content("Valid content")
            .with_author("Valid Author");

        assert!(valid_post.validate().is_ok());

        let invalid_post = BlogPost::new();
        assert!(invalid_post.validate().is_err());
    }
}
