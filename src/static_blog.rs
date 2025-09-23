use anyhow::{Context, Result};
use chrono::NaiveDate;
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a blog post with frontmatter and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub title: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub slug: String,
    pub published: bool,
    pub excerpt: String,
    pub content: String,
    pub html_content: String,
}

/// Represents frontmatter parsed from markdown files
#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    date: NaiveDate,
    tags: Vec<String>,
    slug: String,
    published: bool,
    excerpt: String,
}

/// Load and parse all blog posts from the blog directory
pub fn load_blog_posts<P: AsRef<Path>>(blog_dir: P) -> Result<Vec<BlogPost>> {
    let posts_dir = blog_dir.as_ref().join("posts");

    if !posts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut posts = Vec::new();

    for entry in fs::read_dir(posts_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            match parse_blog_post(&path) {
                Ok(post) => posts.push(post),
                Err(e) => {
                    eprintln!("Warning: Failed to parse blog post {:?}: {}", path, e);
                    continue;
                }
            }
        }
    }

    // Sort posts by date (newest first)
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(posts)
}

/// Parse a single blog post from a markdown file
fn parse_blog_post<P: AsRef<Path>>(path: P) -> Result<BlogPost> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {:?}", path.as_ref()))?;

    let (frontmatter_str, markdown_content) = parse_frontmatter(&content)?;

    // Parse frontmatter
    let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_str)
        .with_context(|| format!("Failed to parse frontmatter in {:?}", path.as_ref()))?;

    // Remove the first H1 from markdown content to avoid title duplication
    let content_without_first_h1 = remove_first_h1(markdown_content);

    // Convert markdown to HTML
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&content_without_first_h1, options);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);

    Ok(BlogPost {
        title: frontmatter.title,
        date: frontmatter.date,
        tags: frontmatter.tags,
        slug: frontmatter.slug,
        published: frontmatter.published,
        excerpt: frontmatter.excerpt,
        content: markdown_content.to_string(),
        html_content,
    })
}

/// Parse frontmatter from markdown content
fn parse_frontmatter(content: &str) -> Result<(&str, &str)> {
    if !content.starts_with("---\n") {
        return Err(anyhow::anyhow!("Missing frontmatter delimiter"));
    }

    let content = &content[4..]; // Skip initial "---\n"

    if let Some(end_pos) = content.find("\n---\n") {
        let frontmatter = &content[..end_pos];
        let markdown_content = &content[end_pos + 5..]; // Skip "\n---\n"
        Ok((frontmatter, markdown_content))
    } else {
        Err(anyhow::anyhow!("Missing closing frontmatter delimiter"))
    }
}

/// Get published blog posts only
pub fn get_published_posts(posts: &[BlogPost]) -> Vec<&BlogPost> {
    posts.iter().filter(|post| post.published).collect()
}

/// Get all unique tags from posts
pub fn get_all_tags(posts: &[BlogPost]) -> Vec<String> {
    let mut tags: Vec<String> = posts.iter().flat_map(|post| &post.tags).cloned().collect();

    tags.sort();
    tags.dedup();
    tags
}

/// Get posts by tag
#[allow(dead_code)]
pub fn get_posts_by_tag<'a>(posts: &'a [BlogPost], tag: &str) -> Vec<&'a BlogPost> {
    posts
        .iter()
        .filter(|post| post.tags.iter().any(|t| t == tag))
        .collect()
}

/// Remove the first H1 heading from markdown content to avoid title duplication
fn remove_first_h1(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines = Vec::new();
    let mut found_first_h1 = false;

    for line in lines {
        let trimmed = line.trim();

        // Check if this is the first H1 (starts with single #)
        if !found_first_h1 && trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            found_first_h1 = true;
            // Skip this line (the first H1)
            continue;
        }

        result_lines.push(line);
    }

    result_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: "Test Post"
date: 2024-01-01
tags: ["rust", "test"]
slug: "test-post"
published: true
excerpt: "A test post"
---

# Test Content

This is a test post."#;

        let (frontmatter, markdown) = parse_frontmatter(content).unwrap();
        assert!(frontmatter.contains("title: \"Test Post\""));
        assert!(markdown.contains("# Test Content"));
    }

    #[test]
    fn test_load_empty_blog_dir() {
        let temp_dir = TempDir::new().unwrap();
        let posts = load_blog_posts(temp_dir.path()).unwrap();
        assert!(posts.is_empty());
    }
}
