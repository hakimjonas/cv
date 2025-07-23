/// API models for OpenAPI documentation
///
/// This module contains models specifically designed for API documentation using utoipa.
/// These models use standard Rust types (Vec, HashMap) instead of immutable data structures
/// from the im crate to ensure compatibility with utoipa's OpenAPI schema generation.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::blog_data;

/// Represents a blog post tag for API documentation
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(description = "A tag for categorizing blog posts")]
pub struct ApiTag {
    /// Unique identifier for the tag (null for new tags)
    #[schema(example = 1)]
    pub id: Option<i64>,
    
    /// Display name of the tag
    #[schema(example = "Technology")]
    pub name: String,
    
    /// URL-friendly version of the name
    #[schema(example = "technology")]
    pub slug: String,
}

/// Represents a blog post for API documentation
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(description = "A blog post with content, metadata, and tags")]
pub struct ApiBlogPost {
    /// Unique identifier for the blog post (null for new posts)
    #[schema(example = 1)]
    pub id: Option<i64>,
    
    /// Title of the blog post
    #[schema(example = "Getting Started with Rust")]
    pub title: String,
    
    /// URL-friendly version of the title
    #[schema(example = "getting-started-with-rust")]
    pub slug: String,
    
    /// Publication date in ISO format
    #[schema(example = "2025-07-23")]
    pub date: String,
    
    /// Author of the blog post
    #[schema(example = "Jane Doe")]
    pub author: String,
    
    /// Short summary of the blog post
    #[schema(example = "A beginner's guide to getting started with Rust programming language")]
    pub excerpt: String,
    
    /// Full content of the blog post
    #[schema(example = "# Getting Started with Rust\n\nRust is a systems programming language...")]
    pub content: String,
    
    /// Whether the post is published (true) or draft (false)
    #[schema(example = true)]
    pub published: bool,
    
    /// Whether the post is featured (true) or not (false)
    #[schema(example = false)]
    pub featured: bool,
    
    /// Optional URL or path to a featured image
    #[schema(example = "/images/rust-logo.png")]
    pub image: Option<String>,
    
    /// List of tags associated with the blog post
    pub tags: Vec<ApiTag>,
    
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Convert a domain Tag to an API Tag
pub fn domain_to_api_tag(tag: &blog_data::Tag) -> ApiTag {
    ApiTag {
        id: tag.id,
        name: tag.name.clone(),
        slug: tag.slug.clone(),
    }
}

/// Convert an API Tag to a domain Tag
pub fn api_to_domain_tag(tag: &ApiTag) -> blog_data::Tag {
    blog_data::Tag {
        id: tag.id,
        name: tag.name.clone(),
        slug: tag.slug.clone(),
    }
}

/// Convert a domain BlogPost to an API BlogPost
pub fn domain_to_api_post(post: &blog_data::BlogPost) -> ApiBlogPost {
    ApiBlogPost {
        id: post.id,
        title: post.title.clone(),
        slug: post.slug.clone(),
        date: post.date.clone(),
        author: post.author.clone(),
        excerpt: post.excerpt.clone(),
        content: post.content.clone(),
        published: post.published,
        featured: post.featured,
        image: post.image.clone(),
        tags: post.tags.iter().map(domain_to_api_tag).collect(),
        metadata: post.metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
    }
}

/// Convert an API BlogPost to a domain BlogPost
pub fn api_to_domain_post(post: &ApiBlogPost) -> blog_data::BlogPost {
    blog_data::BlogPost {
        id: post.id,
        title: post.title.clone(),
        slug: post.slug.clone(),
        date: post.date.clone(),
        author: post.author.clone(),
        excerpt: post.excerpt.clone(),
        content: post.content.clone(),
        published: post.published,
        featured: post.featured,
        image: post.image.clone(),
        tags: post.tags.iter().map(api_to_domain_tag).collect(),
        metadata: post.metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
    }
}

/// Convert a Vector of domain BlogPosts to a Vec of API BlogPosts
pub fn domain_to_api_posts(posts: &im::Vector<blog_data::BlogPost>) -> Vec<ApiBlogPost> {
    posts.iter().map(domain_to_api_post).collect()
}

/// Convert a Vec of API BlogPosts to a Vector of domain BlogPosts
pub fn api_to_domain_posts(posts: &[ApiBlogPost]) -> im::Vector<blog_data::BlogPost> {
    posts.iter().map(api_to_domain_post).collect()
}

/// Convert a Vector of domain Tags to a Vec of API Tags
pub fn domain_to_api_tags(tags: &im::Vector<blog_data::Tag>) -> Vec<ApiTag> {
    tags.iter().map(domain_to_api_tag).collect()
}

/// Convert a Vec of API Tags to a Vector of domain Tags
pub fn api_to_domain_tags(tags: &[ApiTag]) -> im::Vector<blog_data::Tag> {
    tags.iter().map(api_to_domain_tag).collect()
}