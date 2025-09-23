//! Domain models module
//!
//! This module contains all domain entities and value objects.
//! Each domain has its own module with pure data structures
//! and domain-specific business logic.
//!
//! Design principles:
//! - Single source of truth for each domain entity
//! - Use `im` collections for immutable data structures
//! - Type-safe identifiers where appropriate
//! - Clear separation between domain and infrastructure concerns

pub mod blog;
pub mod cv;
pub mod user;

// Re-export commonly used types for convenience
pub use blog::{BlogPost, ContentFormat, Tag};
pub use cv::{Cv, Education, Experience, GitHubSource, PersonalInfo, Project, SkillCategory};
pub use user::{User, UserRole};
