use axum::Router;
/// OpenAPI documentation for the blog API
///
/// This module provides OpenAPI documentation for the blog API using utoipa.
/// It defines the OpenAPI document and provides a Swagger UI for exploring the API.
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api_models::{ApiBlogPost, ApiTag};

/// Generate the OpenAPI document for the blog API
#[derive(OpenApi)]
#[openapi(
    components(
        schemas(ApiBlogPost, ApiTag)
    ),
    tags(
        (name = "blog", description = "Blog API endpoints")
    ),
    info(
        title = "Blog API",
        version = "1.0.0",
        description = "API for managing blog posts and tags",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        ),
        contact(
            name = "API Support",
            email = "support@example.com",
            url = "https://example.com/support"
        )
    )
)]
pub struct ApiDoc;

/// Add Swagger UI to the router
///
/// This function adds the Swagger UI to the router, making it available at /api-docs.
///
/// # Arguments
///
/// * `router` - The router to add the Swagger UI to
///
/// # Returns
///
/// The router with the Swagger UI added
pub fn add_swagger_ui(router: Router) -> Router {
    // Create the Swagger UI
    let swagger_ui = SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi());

    // Add the Swagger UI to the router
    router.merge(swagger_ui)
}
