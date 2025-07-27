/// CV Application library
// Core modules
pub mod cv_data;
pub mod cv_db;
pub mod github;
pub mod github_cache;
pub mod github_oauth;
pub mod html_generator;
pub mod language_icons;
pub mod typst_generator;
pub mod unified_config;

// Database module
pub mod db;

// Blog-related modules
pub mod api_docs;
pub mod api_models;
pub mod auth;
pub mod auth_middleware;
pub mod blog_api;
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

// Utility modules
pub mod credentials;
pub mod logging;

// Re-export dependencies that are commonly used in the codebase
pub use anyhow;
pub use rusqlite;
// Re-export types for axum compatibility
pub use axum;
