/// Services layer module
///
/// This module contains all business logic services that operate on domain entities.
/// Services are pure async and use dependency injection for infrastructure concerns.
///
/// Design principles:
/// - Pure async operations (no sync wrappers)
/// - Dependency injection for repositories
/// - Domain entity-focused operations
/// - Functional programming patterns
/// - Clear separation of concerns
pub mod blog_service;
pub mod cv_service;

// Re-export commonly used services
pub use blog_service::BlogService;
pub use cv_service::CvService;
