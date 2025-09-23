use crate::cv_data::Cv;
use crate::static_blog::{BlogPost, get_all_tags, get_published_posts};
use anyhow::{Context, Result};
use askama::Template;
use std::fs;
use std::path::Path;

/// Template context for the blog list page
#[derive(Template)]
#[template(path = "static_blog.html")]
pub struct BlogListTemplate {
    pub cv: Cv,
    pub posts: Vec<BlogPost>,
    pub tags: Vec<String>,
}

/// Template context for individual blog post pages
#[derive(Template)]
#[template(path = "blog_post.html")]
pub struct BlogPostTemplate {
    pub cv: Cv,
    pub post: BlogPost,
    pub related_posts: Vec<BlogPost>,
}

/// Generate static blog HTML files
pub fn generate_blog_html(posts: &[BlogPost], cv: &Cv, output_dir: &str) -> Result<()> {
    let published_posts = get_published_posts(posts);

    if published_posts.is_empty() {
        return Ok(()); // No posts to generate
    }

    // Convert references to owned for template
    let posts_owned: Vec<BlogPost> = published_posts.into_iter().cloned().collect();
    let tags = get_all_tags(&posts_owned);

    // Generate main blog list page
    let blog_template = BlogListTemplate {
        cv: cv.clone(),
        posts: posts_owned.clone(),
        tags,
    };

    let blog_html = blog_template
        .render()
        .context("Failed to render blog list template")?;

    let blog_path = Path::new(output_dir).join("blog.html");
    fs::write(&blog_path, blog_html)
        .with_context(|| format!("Failed to write blog list to {:?}", blog_path))?;

    // Generate individual post pages
    for post in &posts_owned {
        // Find related posts (same tags, excluding current post)
        let related_posts: Vec<BlogPost> = posts_owned
            .iter()
            .filter(|p| p.slug != post.slug && p.tags.iter().any(|tag| post.tags.contains(tag)))
            .take(3)
            .cloned()
            .collect();

        let post_template = BlogPostTemplate {
            cv: cv.clone(),
            post: post.clone(),
            related_posts,
        };

        let post_html = post_template
            .render()
            .with_context(|| format!("Failed to render template for post: {}", post.slug))?;

        let post_path = Path::new(output_dir)
            .join("blog")
            .join(format!("{}.html", post.slug));

        // Ensure blog directory exists
        if let Some(parent) = post_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        fs::write(&post_path, post_html)
            .with_context(|| format!("Failed to write post to {:?}", post_path))?;
    }

    Ok(())
}
