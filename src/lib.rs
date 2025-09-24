pub mod config;
/// CV Application library
// Core modules
pub mod cv_data;
pub mod cv_db;
pub mod git_identity;
pub mod github;
pub mod github_cache;
pub mod html_generator;
pub mod language_icons;
pub mod migrate;
pub mod typst_generator;
pub mod unified_config;

// Database module
pub mod db;

// Blog-related modules
pub mod api_docs;
pub mod api_models;
pub mod auth;
pub mod blog_converters;
pub mod blog_data;
pub mod blog_error;
pub mod blog_utils;
pub mod blog_validation;
pub mod check_db_permissions;
pub mod content_security_policy;
pub mod csrf_protection;
pub mod feature_flags;
pub mod feed;
pub mod image_api;
pub mod image_storage;
pub mod markdown_editor;
pub mod rate_limiter;
pub mod simple_auth;
pub mod simple_auth_middleware;
pub mod simple_blog_api;

// Utility modules
pub mod logging;

// Re-export dependencies that are commonly used in the codebase
pub use anyhow;
pub use rusqlite;
// Re-export types for axum compatibility
pub use axum;
