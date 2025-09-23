use crate::blog_error::BlogError;
use crate::simple_blog_api::create_simple_blog_api_router;
use axum::Router;
use std::path::PathBuf;

/// Creates a blog API router with default configuration
///
/// This is a wrapper around `create_simple_blog_api_router` that provides default values
/// for the JWT secret, token expiration, and dev mode parameters.
///
/// # Arguments
///
/// * `db_path` - The path to the database file
///
/// # Returns
///
/// A Result containing the Router on success, or a BlogError on failure
pub async fn create_blog_api_router<P: Into<PathBuf>>(db_path: P) -> Result<Router, BlogError> {
    // Default values for the other parameters
    let jwt_secret = "test_secret_key".to_string();
    let token_expiration = 3600; // 1 hour
    let dev_mode = true;

    // Call the actual implementation
    create_simple_blog_api_router(db_path.into(), jwt_secret, token_expiration, dev_mode).await
}
