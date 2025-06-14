// Re-export modules for use in tests
pub mod config;
pub mod cv_data;
pub mod github;
pub mod html_generator;
pub mod language_icons;
pub mod typst_generator;

// Re-export types for axum compatibility
pub use axum;
pub mod blog_api;
pub mod check_db_permissions;
pub mod blog_data;
pub mod blog_utils;
