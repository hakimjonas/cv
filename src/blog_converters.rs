//! Conversion functions between blog_data and repository BlogPost types
//!
//! This module provides functions to convert between the BlogPost type defined in blog_data.rs
//! and the BlogPost type defined in db/repository.rs. This allows the BlogManager to use
//! the BlogRepository while preserving the existing API.

use crate::blog_data;
use crate::db::repository;

/// Convert a blog_data::BlogPost to a repository::BlogPost
pub fn data_to_repo(post: &blog_data::BlogPost) -> repository::BlogPost {
    repository::BlogPost {
        id: post.id,
        title: post.title.clone(),
        slug: post.slug.clone(),
        date: post.date.clone(),
        user_id: post.user_id,
        author: post.author.clone(),
        excerpt: post.excerpt.clone(),
        content: post.content.clone(),
        published: post.published,
        featured: post.featured,
        image: post.image.clone(),
        tags: convert_tags_to_repo(&post.tags),
        metadata: convert_metadata_to_repo(&post.metadata),
    }
}

/// Convert a repository::BlogPost to a blog_data::BlogPost
pub fn repo_to_data(post: &repository::BlogPost) -> blog_data::BlogPost {
    // Determine content format from metadata or default to HTML
    let content_format = if let Some(format) = post.metadata.get("content_format") {
        if format == "markdown" {
            blog_data::ContentFormat::Markdown
        } else {
            blog_data::ContentFormat::HTML
        }
    } else {
        blog_data::ContentFormat::HTML
    };

    blog_data::BlogPost {
        id: post.id,
        title: post.title.clone(),
        slug: post.slug.clone(),
        date: post.date.clone(),
        user_id: post.user_id,
        author: post.author.clone(),
        excerpt: post.excerpt.clone(),
        content: post.content.clone(),
        content_format,
        published: post.published,
        featured: post.featured,
        image: post.image.clone(),
        tags: convert_tags_to_data(&post.tags),
        metadata: convert_metadata_to_data(&post.metadata),
    }
}

/// Convert a Vector of blog_data::Tag to a Vector of repository::Tag
pub fn convert_tags_to_repo(tags: &im::Vector<blog_data::Tag>) -> im::Vector<repository::Tag> {
    tags.iter()
        .map(|tag| repository::Tag {
            id: tag.id,
            name: tag.name.clone(),
            slug: tag.slug.clone(),
        })
        .collect()
}

/// Convert a Vector of repository::Tag to a Vector of blog_data::Tag
pub fn convert_tags_to_data(tags: &im::Vector<repository::Tag>) -> im::Vector<blog_data::Tag> {
    tags.iter()
        .map(|tag| blog_data::Tag {
            id: tag.id,
            name: tag.name.clone(),
            slug: tag.slug.clone(),
        })
        .collect()
}

/// Convert an im::HashMap of metadata to an im::HashMap for repository
pub fn convert_metadata_to_repo(
    metadata: &im::HashMap<String, String>,
) -> im::HashMap<String, String> {
    metadata
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Convert an im::HashMap of metadata from repository to an im::HashMap for blog_data
pub fn convert_metadata_to_data(
    metadata: &im::HashMap<String, String>,
) -> im::HashMap<String, String> {
    metadata
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Convert a Vector of blog_data::BlogPost to a Vector of repository::BlogPost
pub fn convert_posts_to_repo(
    posts: &im::Vector<blog_data::BlogPost>,
) -> im::Vector<repository::BlogPost> {
    posts.iter().map(data_to_repo).collect()
}

/// Convert a Vector of repository::BlogPost to a Vector of blog_data::BlogPost
pub fn convert_posts_to_data(
    posts: &im::Vector<repository::BlogPost>,
) -> im::Vector<blog_data::BlogPost> {
    posts.iter().map(repo_to_data).collect()
}
