/*!
 * Authentication middleware for the blog API
 * This module provides middleware for protecting API routes with authentication
 */

use crate::auth::{AuthError, AuthService, AuthUser, extract_and_validate_token};
use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Request, StatusCode, request::Parts},
    response::Response,
};
use std::sync::Arc;
use tracing::debug;

/// Middleware to require authentication for a route
pub async fn require_auth(
    auth_service: Arc<AuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    debug!("Authenticating request");

    // Extract the authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED);

    // If there's no authorization header, return 401
    let auth_header = match auth_header {
        Ok(header) => header,
        Err(_) => {
            debug!("No authorization header found");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Extract and validate the token
    let auth_user = match extract_and_validate_token(&auth_service, Some(auth_header)).await {
        Ok(user) => user,
        Err(e) => {
            debug!("Token validation failed: {:?}", e);
            return Err(match e {
                AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            });
        }
    };

    // Create a new request with the authenticated user in the extensions
    let mut req = req;
    req.extensions_mut().insert(auth_user);

    // Continue with the request
    Ok(next.run(req).await)
}

/// Middleware to require admin role for a route
pub async fn require_admin(
    auth_service: Arc<AuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    debug!("Checking admin role");

    // Extract the authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED);

    // If there's no authorization header, return 401
    let auth_header = match auth_header {
        Ok(header) => header,
        Err(_) => {
            debug!("No authorization header found");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Extract and validate the token
    let auth_user = match extract_and_validate_token(&auth_service, Some(auth_header)).await {
        Ok(user) => user,
        Err(e) => {
            debug!("Token validation failed: {:?}", e);
            return Err(match e {
                AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            });
        }
    };

    // Check if the user is an admin
    if auth_user.role != "Admin" {
        debug!("User is not an admin: {}", auth_user.username);
        return Err(StatusCode::FORBIDDEN);
    }

    // Create a new request with the authenticated user in the extensions
    let mut req = req;
    req.extensions_mut().insert(auth_user);

    // Continue with the request
    Ok(next.run(req).await)
}

/// Middleware to require author role for a route
pub async fn require_author(
    auth_service: Arc<AuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    debug!("Checking author role");

    // Extract the authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED);

    // If there's no authorization header, return 401
    let auth_header = match auth_header {
        Ok(header) => header,
        Err(_) => {
            debug!("No authorization header found");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Extract and validate the token
    let auth_user = match extract_and_validate_token(&auth_service, Some(auth_header)).await {
        Ok(user) => user,
        Err(e) => {
            debug!("Token validation failed: {:?}", e);
            return Err(match e {
                AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            });
        }
    };

    // Check if the user is an author or admin
    if auth_user.role != "Author" && auth_user.role != "Admin" {
        debug!("User is not an author or admin: {}", auth_user.username);
        return Err(StatusCode::FORBIDDEN);
    }

    // Create a new request with the authenticated user in the extensions
    let mut req = req;
    req.extensions_mut().insert(auth_user);

    // Continue with the request
    Ok(next.run(req).await)
}

/// Middleware to require editor role for a route
pub async fn require_editor(
    auth_service: Arc<AuthService>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    debug!("Checking editor role");

    // Extract the authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED);

    // If there's no authorization header, return 401
    let auth_header = match auth_header {
        Ok(header) => header,
        Err(_) => {
            debug!("No authorization header found");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Extract and validate the token
    let auth_user = match extract_and_validate_token(&auth_service, Some(auth_header)).await {
        Ok(user) => user,
        Err(e) => {
            debug!("Token validation failed: {:?}", e);
            return Err(match e {
                AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            });
        }
    };

    // Check if the user is an editor, author, or admin
    if auth_user.role != "Editor" && auth_user.role != "Author" && auth_user.role != "Admin" {
        debug!(
            "User is not an editor, author, or admin: {}",
            auth_user.username
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Create a new request with the authenticated user in the extensions
    let mut req = req;
    req.extensions_mut().insert(auth_user);

    // Continue with the request
    Ok(next.run(req).await)
}

/// Extractor for getting the authenticated user from request extensions
pub struct AuthenticatedUser(pub AuthUser);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .map(|user| AuthenticatedUser(user.clone()))
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
