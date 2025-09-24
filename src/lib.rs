/// CV Application library - Lean static website generator
/// Built with functional programming principles and immutable data structures

// Core CV generation modules
pub mod cv_data;
pub mod git_identity;
pub mod github;
pub mod github_cache;
pub mod html_generator;
pub mod language_icons;
pub mod typst_generator;
pub mod unified_config;

// Utility modules
pub mod logging;

// Re-export core dependencies
pub use anyhow;
