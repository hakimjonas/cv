/// CV Application library
// Core modules
pub mod config;
pub mod cv_data;
pub mod cv_db;
pub mod github;
pub mod html_generator;
pub mod language_icons;
pub mod typst_generator;

// Database module
pub mod db;

// Blog-related modules
pub mod blog_api;
pub mod blog_data;
pub mod blog_error;
pub mod blog_utils;
pub mod check_db_permissions;

// Utility modules
pub mod logging;

// Re-export dependencies that are commonly used in the codebase
pub use anyhow;
pub use rusqlite;
// Re-export types for axum compatibility
pub use axum;
