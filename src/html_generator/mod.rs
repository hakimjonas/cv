//! HTML Generation Module
//!
//! This module provides comprehensive HTML generation for the CV generator,
//! organized into focused submodules for maintainability and clarity.
//!
//! ## Architecture
//!
//! The HTML generation system is organized into several specialized modules:
//!
//! - [`html_generators`] - Core HTML page generation functions
//! - [`config_generators`] - Server and deployment configuration files
//! - [`asset_processor`] - Static asset copying and file operations
//! - [`utils`] - Shared utilities for file operations and content processing
//!
//! ## Usage
//!
//! The main entry point is [`generate_html`], which orchestrates the entire
//! HTML generation process including all pages, configurations, and assets.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::blog_posts::{group_posts_by_tags, load_posts_from_directory};
use crate::css_generator::generate_colorscheme_css;
use crate::cv_data::Cv;
use crate::dependencies::parse_dependencies;
use crate::markdown_pages::load_pages_from_directory;
use crate::optimization::{optimize_css_file, optimize_js_file};
use crate::site_config::SiteConfig;

// Re-export public functions from submodules
pub use asset_processor::copy_static_assets_except;
pub use config_generators::*;
pub use html_generators::*;

// Submodule declarations
pub mod asset_processor;
pub mod config_generators;
pub mod html_generators;
pub mod utils;

/// Main HTML generation function that coordinates all HTML output
///
/// This function generates all HTML pages, configuration files, and handles
/// asset copying for a complete static site.
///
/// # Arguments
///
/// * `cv` - CV data containing all personal and professional information
/// * `site_config` - Site configuration with styling and navigation settings
/// * `output_path` - Base path for the main CV HTML file (other files derive from this)
///
/// # Returns
///
/// A Result indicating success or failure of the entire generation process
///
/// # Examples
///
/// ```rust,no_run
/// use cv_generator::html_generator::generate_html;
/// use cv_generator::{cv_data::Cv, site_config::SiteConfig};
///
/// fn main() -> anyhow::Result<()> {
///     let cv = Cv::from_json("data/cv_data.json")?;
///     let site_config = SiteConfig::from_json("config/site.json")?;
///     generate_html(&cv, &site_config, "dist/cv.html")?;
///     Ok(())
/// }
/// ```
pub fn generate_html(cv: &Cv, site_config: &SiteConfig, output_path: &str) -> Result<()> {
    // Parse dependencies from Cargo.toml
    let dependencies = parse_dependencies("Cargo.toml").unwrap_or_default();

    // Generate main CV HTML
    generate_cv_html(cv, site_config, &dependencies, output_path)?;

    // Get parent directory for other HTML files
    let parent_dir = Path::new(output_path)
        .parent()
        .context("Failed to get parent directory")?;

    // Generate index HTML
    let index_path = parent_dir
        .join("index.html")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();

    generate_index_html(cv, site_config, &index_path)?;

    // Generate projects HTML
    let projects_path = parent_dir
        .join("projects.html")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();

    generate_projects_html(cv, site_config, &dependencies, &projects_path)?;

    // Generate blog HTML
    let blog_path = parent_dir
        .join("blog.html")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();

    // Generate static blog posts from markdown if configured
    if let Some(blog_config) = &site_config.blog {
        let blog_dir = blog_config.directory.as_deref().unwrap_or("content/blog");

        let blog_path_dir = Path::new(blog_dir);
        if blog_path_dir.exists() {
            let posts = load_posts_from_directory(blog_path_dir)?;
            let tag_groups = group_posts_by_tags(&posts);

            // Generate blog list page
            generate_blog_list_html(
                cv,
                site_config,
                &posts,
                &tag_groups,
                &dependencies,
                &blog_path,
            )?;

            // Create blog subdirectory for individual posts
            let blog_posts_dir = parent_dir.join("blog");
            fs::create_dir_all(&blog_posts_dir)?;

            // Generate individual blog post pages
            for post in posts.iter() {
                let post_path = blog_posts_dir
                    .join(format!("{}.html", post.slug))
                    .to_str()
                    .context("Failed to convert path to string")?
                    .to_string();

                generate_blog_post_html(cv, site_config, post, &dependencies, &post_path)?;
            }
        }
    } else {
        // Fallback to old blog template if not configured
        generate_blog_html(cv, site_config, &dependencies, &blog_path)?;
    }

    // Generate static pages from markdown if configured
    if let Some(pages_config) = &site_config.pages {
        let pages_dir = pages_config.directory.as_deref().unwrap_or("content/pages");

        let pages_path_dir = Path::new(pages_dir);
        if pages_path_dir.exists() {
            let pages = load_pages_from_directory(pages_path_dir)?;

            // Generate each static page
            for page in pages.iter() {
                let page_path = parent_dir
                    .join(format!("{}.html", page.slug))
                    .to_str()
                    .context("Failed to convert path to string")?
                    .to_string();

                generate_page_html(cv, site_config, page, &dependencies, &page_path)?;
            }
        }
    }

    // Generate dynamic CSS files if configurations are present
    if let Some(fonts_config) = &site_config.fonts {
        let font_css_path = parent_dir
            .join("css")
            .join("generated")
            .join("fonts.css")
            .to_str()
            .context("Failed to convert path to string")?
            .to_string();

        // Ensure the generated CSS directory exists
        if let Some(css_parent) = Path::new(&font_css_path).parent() {
            fs::create_dir_all(css_parent)?;
        }

        generate_font_css(fonts_config, &font_css_path)?;
    }

    if let Some(colorscheme_config) = &site_config.colorscheme {
        let colorscheme_css_path = parent_dir
            .join("css")
            .join("generated")
            .join("colorscheme.css")
            .to_str()
            .context("Failed to convert path to string")?
            .to_string();

        // Ensure the generated CSS directory exists
        if let Some(css_parent) = Path::new(&colorscheme_css_path).parent() {
            fs::create_dir_all(css_parent)?;
        }

        generate_colorscheme_css(colorscheme_config, &colorscheme_css_path)?;
    }

    // Generate deployment and SEO configuration files
    generate_deployment_configs(parent_dir)?;

    println!("HTML generation completed successfully");
    Ok(())
}

/// Generates all deployment and SEO configuration files
///
/// # Arguments
///
/// * `parent_dir` - Base directory where configuration files will be written
///
/// # Returns
///
/// A Result indicating success or failure
fn generate_deployment_configs(parent_dir: &Path) -> Result<()> {
    // Generate .htaccess for Apache servers
    let htaccess_path = parent_dir
        .join(".htaccess")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_htaccess(&htaccess_path)?;

    // Generate web.config for IIS servers
    let web_config_path = parent_dir
        .join("web.config")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_web_config(&web_config_path)?;

    // Generate Netlify configuration files
    let netlify_headers_path = parent_dir
        .join("_headers")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_netlify_headers(&netlify_headers_path)?;

    let netlify_redirects_path = parent_dir
        .join("_redirects")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_netlify_redirects(&netlify_redirects_path)?;

    // Generate SEO files
    let robots_path = parent_dir
        .join("robots.txt")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_robots_txt(&robots_path)?;

    // Generate PWA files
    let manifest_path = parent_dir
        .join("manifest.json")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_manifest_json(&manifest_path)?;

    let sw_path = parent_dir
        .join("service-worker.js")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_service_worker(&sw_path)?;

    Ok(())
}

/// Optimizes CSS and JavaScript assets for production
///
/// # Arguments
///
/// * `parent_dir` - Base directory containing assets to optimize
///
/// # Returns
///
/// A Result indicating success or failure
pub fn optimize_assets(parent_dir: &Path) -> Result<()> {
    let css_dir = parent_dir.join("css");
    let js_dir = parent_dir.join("js");

    // Optimize main CSS file if it exists
    let main_css = css_dir.join("main.css");
    if main_css.exists() {
        let main_css_min = css_dir.join("main.min.css");
        optimize_css_file(&main_css, &main_css_min)?;
        println!("Optimized main.css → main.min.css");
    }

    // Optimize main JS file if it exists
    let main_js = js_dir.join("scripts.js");
    if main_js.exists() {
        let main_js_min = js_dir.join("scripts.min.js");
        optimize_js_file(&main_js, &main_js_min)?;
        println!("Optimized scripts.js → scripts.min.js");
    }

    Ok(())
}
