use crate::auth::{AuthError, AuthUser};
use crate::simple_auth::{SimpleAuthService, extract_and_validate_token};
use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Request, StatusCode, request::Parts},
    response::Response,
};
use std::sync::Arc;

/// Middleware to require authentication
pub async fn require_auth(
    auth_service: Arc<SimpleAuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();
    
    // Extract remote address for dev mode auto-authentication
    let remote_addr = parts.extensions.get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| connect_info.0.ip());
    
    // Extract and validate the token
    let auth_result = extract_and_validate_token(
        &auth_service,
        parts.headers.get("Authorization"),
        remote_addr,
    ).await;
    
    match auth_result {
        Ok(user) => {
            // Add the authenticated user to the request extensions
            parts.extensions.insert(AuthenticatedUser(user));
            let req = Request::from_parts(parts, body);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware to require admin role
pub async fn require_admin(
    auth_service: Arc<SimpleAuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();
    
    // Extract remote address for dev mode auto-authentication
    let remote_addr = parts.extensions.get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| connect_info.0.ip());
    
    // Extract and validate the token
    let auth_result = extract_and_validate_token(
        &auth_service,
        parts.headers.get("Authorization"),
        remote_addr,
    ).await;
    
    match auth_result {
        Ok(user) => {
            // Check if the user has admin role
            if user.role == "Admin" {
                // Add the authenticated user to the request extensions
                parts.extensions.insert(AuthenticatedUser(user));
                let req = Request::from_parts(parts, body);
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware to require author role
pub async fn require_author(
    auth_service: Arc<SimpleAuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();
    
    // Extract remote address for dev mode auto-authentication
    let remote_addr = parts.extensions.get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| connect_info.0.ip());
    
    // Extract and validate the token
    let auth_result = extract_and_validate_token(
        &auth_service,
        parts.headers.get("Authorization"),
        remote_addr,
    ).await;
    
    match auth_result {
        Ok(user) => {
            // Check if the user has author or admin role
            if user.role == "Author" || user.role == "Admin" {
                // Add the authenticated user to the request extensions
                parts.extensions.insert(AuthenticatedUser(user));
                let req = Request::from_parts(parts, body);
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware to require editor role
pub async fn require_editor(
    auth_service: Arc<SimpleAuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();
    
    // Extract remote address for dev mode auto-authentication
    let remote_addr = parts.extensions.get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| connect_info.0.ip());
    
    // Extract and validate the token
    let auth_result = extract_and_validate_token(
        &auth_service,
        parts.headers.get("Authorization"),
        remote_addr,
    ).await;
    
    match auth_result {
        Ok(user) => {
            // Check if the user has editor, author, or admin role
            if user.role == "Editor" || user.role == "Author" || user.role == "Admin" {
                // Add the authenticated user to the request extensions
                parts.extensions.insert(AuthenticatedUser(user));
                let req = Request::from_parts(parts, body);
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Wrapper for authenticated user
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub AuthUser);

/// Implementation to extract authenticated user from request
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or(AuthError::Unauthorized)
    }
}