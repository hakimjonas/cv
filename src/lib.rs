/// CV Application library - Lean static website generator
/// Built with functional programming principles and immutable data structures
// Core CV generation modules
pub mod cv_data;
pub mod git_config;
pub mod github;
pub mod html_generator;
pub mod language_icons;
pub mod site_config;
pub mod typst_generator;
pub mod unified_config;

// Utility modules
// pub mod logging; // Disabled for now

// Re-export core dependencies
pub use anyhow;
