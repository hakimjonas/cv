/// Blog post validation and sanitization module
///
/// This module provides validation and sanitization functions for blog post content
/// to prevent XSS attacks and ensure data integrity.
use crate::blog_data::{BlogPost, Tag};
use ammonia::{Builder, Url};
use regex::Regex;
use std::collections::HashSet;
use thiserror::Error;
use tracing::debug;
/// Blog validation errors
#[derive(Debug, Error)]
pub enum BlogValidationError {
    /// Validation error with details
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Sanitization error
    #[error("Sanitization error: {0}")]
    SanitizationError(String),
}

/// Validation rules for blog posts
#[derive(Debug)]
pub struct BlogPostValidation {
    /// Title must be between 3 and 200 characters
    pub title: String,

    /// Slug must be between 3 and 100 characters and contain only lowercase letters, numbers, and hyphens
    pub slug: String,

    /// Date must be in YYYY-MM-DD format
    pub date: String,

    /// Author must be between 2 and 100 characters
    pub author: String,

    /// Excerpt must be between 10 and 500 characters
    pub excerpt: String,

    /// Content must be between 10 and 100000 characters
    pub content: String,
}

// Regular expressions for validation
lazy_static::lazy_static! {
    static ref SLUG_REGEX: Regex = Regex::new(r"^[a-z0-9-]+$").unwrap();
    static ref DATE_REGEX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
}

/// Validates a blog post
///
/// # Arguments
///
/// * `post` - The blog post to validate
///
/// # Returns
///
/// A Result containing () if validation passes, or a BlogValidationError if validation fails
pub fn validate_blog_post(post: &BlogPost) -> Result<(), BlogValidationError> {
    // Validate title
    if post.title.len() < 3 || post.title.len() > 200 {
        return Err(BlogValidationError::ValidationError(
            "Title must be between 3 and 200 characters".to_string(),
        ));
    }

    // Validate slug
    if post.slug.len() < 3 || post.slug.len() > 100 {
        return Err(BlogValidationError::ValidationError(
            "Slug must be between 3 and 100 characters".to_string(),
        ));
    }
    if !SLUG_REGEX.is_match(&post.slug) {
        return Err(BlogValidationError::ValidationError(
            "Slug must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    // Validate date
    if !DATE_REGEX.is_match(&post.date) {
        return Err(BlogValidationError::ValidationError(
            "Date must be in YYYY-MM-DD format".to_string(),
        ));
    }

    // Validate author
    if post.author.len() < 2 || post.author.len() > 100 {
        return Err(BlogValidationError::ValidationError(
            "Author must be between 2 and 100 characters".to_string(),
        ));
    }

    // Validate excerpt
    if post.excerpt.len() < 10 || post.excerpt.len() > 500 {
        return Err(BlogValidationError::ValidationError(
            "Excerpt must be between 10 and 500 characters".to_string(),
        ));
    }

    // Validate content
    if post.content.len() < 10 || post.content.len() > 100000 {
        return Err(BlogValidationError::ValidationError(
            "Content must be between 10 and 100000 characters".to_string(),
        ));
    }

    // Validate tags
    for tag in &post.tags {
        validate_tag(tag)?;
    }

    debug!("Blog post validation passed");
    Ok(())
}

/// Validates a tag
///
/// # Arguments
///
/// * `tag` - The tag to validate
///
/// # Returns
///
/// A Result containing () if validation passes, or a BlogValidationError if validation fails
pub fn validate_tag(tag: &Tag) -> Result<(), BlogValidationError> {
    // Validate tag name
    if tag.name.is_empty() || tag.name.len() > 50 {
        return Err(BlogValidationError::ValidationError(
            "Tag name must be between 1 and 50 characters".to_string(),
        ));
    }

    // Validate tag slug
    if tag.slug.is_empty() || tag.slug.len() > 50 {
        return Err(BlogValidationError::ValidationError(
            "Tag slug must be between 1 and 50 characters".to_string(),
        ));
    }

    // Validate tag slug format
    if !SLUG_REGEX.is_match(&tag.slug) {
        return Err(BlogValidationError::ValidationError(
            "Tag slug must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    Ok(())
}

/// Sanitizes a blog post to prevent XSS attacks
///
/// # Arguments
///
/// * `post` - The blog post to sanitize
///
/// # Returns
///
/// A sanitized blog post
pub fn sanitize_blog_post(post: &BlogPost) -> BlogPost {
    let mut sanitized_post = post.clone();

    // Sanitize title (no HTML allowed)
    sanitized_post.title = sanitize_text(&post.title);

    // Sanitize excerpt (no HTML allowed)
    sanitized_post.excerpt = sanitize_text(&post.excerpt);

    // Sanitize content (limited HTML allowed)
    sanitized_post.content = sanitize_html(&post.content);

    // Sanitize author (no HTML allowed)
    sanitized_post.author = sanitize_text(&post.author);

    // Sanitize image URL if present
    if let Some(image) = &post.image {
        sanitized_post.image = Some(sanitize_url(image));
    }

    // Sanitize tags
    sanitized_post.tags = post.tags.iter().map(sanitize_tag).collect();

    sanitized_post
}

/// Sanitizes a tag to prevent XSS attacks
///
/// # Arguments
///
/// * `tag` - The tag to sanitize
///
/// # Returns
///
/// A sanitized tag
pub fn sanitize_tag(tag: &Tag) -> Tag {
    Tag {
        id: tag.id,
        name: sanitize_text(&tag.name),
        slug: sanitize_text(&tag.slug),
    }
}

/// Sanitizes text to prevent XSS attacks by removing all HTML
///
/// # Arguments
///
/// * `text` - The text to sanitize
///
/// # Returns
///
/// Sanitized text with all HTML removed
pub fn sanitize_text(text: &str) -> String {
    ammonia::clean_text(text)
}

/// Sanitizes HTML to prevent XSS attacks by allowing only safe tags and attributes
///
/// # Arguments
///
/// * `html` - The HTML to sanitize
///
/// # Returns
///
/// Sanitized HTML with only safe tags and attributes
pub fn sanitize_html(html: &str) -> String {
    // Create a custom ammonia builder with safe defaults
    let mut builder = Builder::default();

    // Allow common formatting tags
    let allowed_tags = HashSet::from([
        "a",
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "blockquote",
        "pre",
        "code",
        "em",
        "strong",
        "i",
        "b",
        "ul",
        "ol",
        "li",
        "br",
        "hr",
        "img",
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
        "caption",
        "colgroup",
        "col",
        "span",
        "div",
        "strike",
        "del",
        "sup",
        "sub",
        "kbd",
    ]);
    builder.tags(allowed_tags);

    // Allow common attributes
    builder.add_generic_attributes(&[
        "id",
        "class",
        "name",
        "title",
        "alt",
        "width",
        "height",
        "style",
        "data-language",
        "data-lang",
    ]);

    // Allow specific attributes for specific tags
    builder.link_rel(Some("noopener noreferrer"));
    builder.add_tag_attributes("a", &["target"]);
    builder.add_tag_attributes("img", &["alt", "width", "height"]);

    // Allow safe URL schemes
    let url_schemes = HashSet::from(["http", "https", "mailto", "tel", "ftp"]);
    builder.url_schemes(url_schemes);

    // Clean the HTML
    builder.clean(html).to_string()
}

/// Sanitizes a URL to prevent XSS attacks
///
/// # Arguments
///
/// * `url` - The URL to sanitize
///
/// # Returns
///
/// A sanitized URL
pub fn sanitize_url(url: &str) -> String {
    // Parse the URL
    match Url::parse(url) {
        Ok(parsed_url) => {
            // Check if the URL scheme is safe
            match parsed_url.scheme() {
                "http" | "https" | "mailto" | "tel" | "ftp" => url.to_string(),
                _ => "#".to_string(), // Replace unsafe URLs with a harmless placeholder
            }
        }
        Err(_) => {
            // If the URL is invalid, return a harmless placeholder
            "#".to_string()
        }
    }
}

/// Validates and sanitizes a blog post
///
/// # Arguments
///
/// * `post` - The blog post to validate and sanitize
///
/// # Returns
///
/// A Result containing the sanitized blog post if validation passes,
/// or a BlogValidationError if validation fails
pub fn validate_and_sanitize_blog_post(post: &BlogPost) -> Result<BlogPost, BlogValidationError> {
    // Validate the blog post
    validate_blog_post(post)?;

    // Sanitize the blog post
    let sanitized_post = sanitize_blog_post(post);

    Ok(sanitized_post)
}
