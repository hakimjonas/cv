/// CSRF protection module
///
/// This module provides CSRF protection for the blog API using the axum-csrf crate.
/// It includes token generation, validation, and middleware for CSRF token verification.
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken, SameSite};
use tracing::{debug, warn};

/// CSRF configuration
#[derive(Debug, Clone)]
pub struct CsrfProtectionConfig {
    /// Token validity duration in seconds
    pub token_validity_seconds: u64,
    /// Whether to include the CSRF token in the response headers
    pub include_headers: bool,
    /// Cookie name for the CSRF token
    pub cookie_name: String,
    /// Cookie path
    pub cookie_path: String,
    /// Whether the cookie is secure (HTTPS only)
    pub cookie_secure: bool,
    /// Whether the cookie is HTTP only
    pub cookie_http_only: bool,
    /// Same site policy for the cookie
    pub cookie_same_site: SameSite,
}

impl Default for CsrfProtectionConfig {
    fn default() -> Self {
        Self {
            token_validity_seconds: 3600, // 1 hour
            include_headers: true,
            cookie_name: "csrf_token".to_string(),
            cookie_path: "/".to_string(),
            cookie_secure: true,
            cookie_http_only: true,
            cookie_same_site: SameSite::Strict,
        }
    }
}

/// Create a CSRF protection layer with the given configuration
pub fn create_csrf_layer(_config: CsrfProtectionConfig) -> CsrfLayer {
    // Create a CSRF configuration with default values to avoid lifetime issues
    // TODO: Fix this function to properly use the config values
    let csrf_config = CsrfConfig::new();

    // Create a CSRF layer
    CsrfLayer::new(csrf_config)
}

/// CSRF middleware for Axum
///
/// This middleware checks for a valid CSRF token in the request headers or form data.
/// It should be applied to all routes that modify data (POST, PUT, DELETE).
pub async fn csrf_middleware(
    State(csrf_token): State<CsrfToken>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip CSRF check for GET, HEAD, and OPTIONS requests
    let method = request.method().clone();
    if method == axum::http::Method::GET
        || method == axum::http::Method::HEAD
        || method == axum::http::Method::OPTIONS
    {
        return Ok(next.run(request).await);
    }

    // Check for CSRF token in headers
    let token_header = request.headers().get("X-CSRF-Token");
    if let Some(token) = token_header
        && let Ok(token_str) = token.to_str()
    {
        // The verify method returns Result<(), CsrfError>, so we need to check if it's Ok
        if csrf_token.verify(token_str).is_ok() {
            debug!("Valid CSRF token found in headers");
            return Ok(next.run(request).await);
        }
    }

    // If we get here, the CSRF token is invalid or missing
    warn!("Invalid or missing CSRF token");
    Err(StatusCode::FORBIDDEN)
}

/// Add CSRF token to a response
///
/// This function adds a CSRF token to the response headers.
pub fn add_csrf_token_to_response(response: Response, _csrf_token: &CsrfToken) -> Response {
    // TODO: Fix this function to properly get a string representation of the token
    // The CsrfToken doesn't implement Display or ToString
    // let token = csrf_token.to_string();

    // For now, just return the response without adding the token
    response
}
