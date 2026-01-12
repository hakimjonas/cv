//! # CV Generator Library
//!
//! A blazing-fast, functional CV/portfolio generator built in Rust that creates
//! beautiful HTML and PDF outputs from JSON data.
//!
//! ## Key Features
//!
//! - **⚡ Sub-second builds** with intelligent caching (77% performance improvement)
//! - **🧠 Smart GitHub integration** with TTL-based API caching
//! - **🏗️ Modular architecture** following functional programming principles
//! - **📊 Built-in profiling** for optimization and performance monitoring
//! - **🎨 Dynamic CSS generation** with colorscheme support
//!
//! ## Architecture
//!
//! The library is organized into focused modules following single-responsibility principles:
//!
//! - [`cv_data`] - Core data structures and JSON parsing
//! - [`github`] - GitHub API integration with intelligent caching
//! - [`html_generator`] - Modular HTML generation system
//! - [`performance`] - Build profiling and optimization tools
//! - [`github_cache`] - TTL-based caching system for API responses
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use cv_generator::{cv_data::Cv, site_config::SiteConfig, html_generator::generate_html};
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     let cv = Cv::from_json("data/cv_data.json")?;
//!     let site_config = SiteConfig::from_json("config/site.json")?;
//!     generate_html(&cv, &site_config, "dist/cv.html")?;
//!     Ok(())
//! }
//! ```
//!
//! ## Performance
//!
//! This generator achieves sub-second builds through:
//! - Intelligent GitHub API caching (100% cache hits after first run)
//! - Configuration-based CSS cache invalidation
//! - Parallel asset processing
//! - Built-in performance profiling
//!
// Core CV generation modules
pub mod blog_posts;
pub mod colorscheme_provider; // Now a directory module
pub mod cover_letter;
pub mod css_generator;
pub mod cv_data;
pub mod dependencies;
pub mod github;
pub mod github_cache;
pub mod html_generator;
pub mod language_icons;
pub mod markdown_pages;
pub mod optimization;
pub mod performance;
pub mod site_config;
pub mod typst_generator;
pub mod unified_config;
pub mod validation;

// Utility modules
// pub mod logging; // Disabled for now

// Re-export core dependencies
pub use anyhow;
