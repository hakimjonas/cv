//! HTML page generators
//!
//! This module contains functions for generating specific HTML pages including
//! CV, index, projects, blog, and static pages.

use anyhow::{Context, Result};
use askama::Template;
use im::{HashMap, Vector};

use super::utils::{ensure_parent_dir_exists, get_cache_version, write_file};
use crate::blog_posts::BlogPost;
use crate::cv_data::Cv;
use crate::dependencies::Dependency;
use crate::markdown_pages::Page;
use crate::site_config::SiteConfig;

/// Template for the CV HTML page
#[derive(Template)]
#[template(path = "cv.html")]
struct CvTemplate<'a> {
    cv: &'a Cv,
    site_config: &'a SiteConfig,
    version: &'a str,
    dependencies: &'a [Dependency],
}

/// Template for the index HTML page
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    cv: &'a Cv,
}

/// Template for the projects HTML page
#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate<'a> {
    cv: &'a Cv,
    site_config: &'a SiteConfig,
    version: &'a str,
    dependencies: &'a [Dependency],
}

/// Template for the blog HTML page
#[derive(Template)]
#[template(path = "blog.html")]
struct BlogTemplate<'a> {
    cv: &'a Cv,
    site_config: &'a SiteConfig,
    version: &'a str,
    dependencies: &'a [Dependency],
}

/// Template for static pages
#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    cv: &'a Cv,
    site_config: &'a SiteConfig,
    page: &'a Page,
    version: &'a str,
    dependencies: &'a [Dependency],
}

/// Template for blog list page
#[derive(Template)]
#[template(path = "blog_list.html")]
struct BlogListTemplate<'a> {
    cv: &'a Cv,
    site_config: &'a SiteConfig,
    posts: &'a Vector<BlogPost>,
    tag_groups: &'a HashMap<String, Vector<BlogPost>>,
    version: &'a str,
    dependencies: &'a [Dependency],
}

/// Template for individual blog post
#[derive(Template)]
#[template(path = "blog_post.html")]
struct BlogPostTemplate<'a> {
    cv: &'a Cv,
    site_config: &'a SiteConfig,
    post: &'a BlogPost,
    version: &'a str,
    dependencies: &'a [Dependency],
}

/// Generates the main CV HTML page
///
/// # Arguments
///
/// * `cv` - CV data
/// * `site_config` - Site configuration
/// * `dependencies` - Project dependencies from Cargo.toml
/// * `output_path` - Path where the CV HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_cv_html(
    cv: &Cv,
    site_config: &SiteConfig,
    dependencies: &[Dependency],
    output_path: &str,
) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;

    let version = get_cache_version();
    let template = CvTemplate {
        cv,
        site_config,
        version: &version,
        dependencies,
    };
    let rendered = template.render().context("Failed to render CV template")?;

    write_file(output_path, &rendered)?;
    println!("Generated CV HTML: {output_path}");

    Ok(())
}

/// Generates the index HTML page
///
/// # Arguments
///
/// * `cv` - CV data
/// * `_site_config` - Site configuration (unused but kept for consistency)
/// * `output_path` - Path where the index HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_index_html(cv: &Cv, _site_config: &SiteConfig, output_path: &str) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;

    let template = IndexTemplate { cv };
    let rendered = template
        .render()
        .context("Failed to render index template")?;

    write_file(output_path, &rendered)?;
    println!("Generated index HTML: {output_path}");

    Ok(())
}

/// Generates the projects HTML page
///
/// # Arguments
///
/// * `cv` - CV data
/// * `site_config` - Site configuration
/// * `dependencies` - Project dependencies from Cargo.toml
/// * `output_path` - Path where the projects HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_projects_html(
    cv: &Cv,
    site_config: &SiteConfig,
    dependencies: &[Dependency],
    output_path: &str,
) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;

    let version = get_cache_version();
    let template = ProjectsTemplate {
        cv,
        site_config,
        version: &version,
        dependencies,
    };
    let rendered = template
        .render()
        .context("Failed to render projects template")?;

    write_file(output_path, &rendered)?;
    println!("Generated projects HTML: {output_path}");

    Ok(())
}

/// Generates the blog HTML page (legacy version)
///
/// # Arguments
///
/// * `cv` - CV data
/// * `site_config` - Site configuration
/// * `dependencies` - Project dependencies from Cargo.toml
/// * `output_path` - Path where the blog HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_blog_html(
    cv: &Cv,
    site_config: &SiteConfig,
    dependencies: &[Dependency],
    output_path: &str,
) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;

    let version = get_cache_version();
    let template = BlogTemplate {
        cv,
        site_config,
        version: &version,
        dependencies,
    };
    let rendered = template
        .render()
        .context("Failed to render blog template")?;

    write_file(output_path, &rendered)?;
    println!("Generated blog HTML: {output_path}");

    Ok(())
}

/// Generates a static page from markdown
///
/// # Arguments
///
/// * `cv` - CV data
/// * `site_config` - Site configuration
/// * `page` - Page data with content and metadata
/// * `dependencies` - Project dependencies from Cargo.toml
/// * `output_path` - Path where the page HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_page_html(
    cv: &Cv,
    site_config: &SiteConfig,
    page: &Page,
    dependencies: &[Dependency],
    output_path: &str,
) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;

    let version = get_cache_version();
    let template = PageTemplate {
        cv,
        site_config,
        page,
        version: &version,
        dependencies,
    };
    let rendered = template
        .render()
        .context("Failed to render page template")?;

    write_file(output_path, &rendered)?;
    println!("Generated page HTML: {output_path}");

    Ok(())
}

/// Generates the blog list page with all posts
///
/// # Arguments
///
/// * `cv` - CV data
/// * `site_config` - Site configuration
/// * `posts` - Vector of blog posts
/// * `tag_groups` - Posts grouped by tags
/// * `dependencies` - Project dependencies from Cargo.toml
/// * `output_path` - Path where the blog list HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_blog_list_html(
    cv: &Cv,
    site_config: &SiteConfig,
    posts: &Vector<BlogPost>,
    tag_groups: &HashMap<String, Vector<BlogPost>>,
    dependencies: &[Dependency],
    output_path: &str,
) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;

    let version = get_cache_version();
    let template = BlogListTemplate {
        cv,
        site_config,
        posts,
        tag_groups,
        version: &version,
        dependencies,
    };
    let rendered = template
        .render()
        .context("Failed to render blog list template")?;

    write_file(output_path, &rendered)?;
    println!("Generated blog list HTML: {output_path}");

    Ok(())
}

/// Generates an individual blog post page
///
/// # Arguments
///
/// * `cv` - CV data
/// * `site_config` - Site configuration
/// * `post` - Blog post data
/// * `dependencies` - Project dependencies from Cargo.toml
/// * `output_path` - Path where the blog post HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_blog_post_html(
    cv: &Cv,
    site_config: &SiteConfig,
    post: &BlogPost,
    dependencies: &[Dependency],
    output_path: &str,
) -> Result<()> {
    ensure_parent_dir_exists(output_path)?;

    let version = get_cache_version();
    let template = BlogPostTemplate {
        cv,
        site_config,
        post,
        version: &version,
        dependencies,
    };
    let rendered = template
        .render()
        .context("Failed to render blog post template")?;

    write_file(output_path, &rendered)?;
    println!("Generated blog post HTML: {output_path}");

    Ok(())
}
