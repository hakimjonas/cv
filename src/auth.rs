/*!
 * Authentication module
 * This module provides common authentication types for the API
 */

use crate::blog_error::BlogError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::errors::ErrorKind;
use serde::Serialize;

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing token")]
    MissingToken,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing token".to_string()),
            AuthError::Unauthorized => (StatusCode::FORBIDDEN, "Unauthorized".to_string()),
            AuthError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, error_message).into_response()
    }
}

impl From<BlogError> for AuthError {
    fn from(error: BlogError) -> Self {
        AuthError::Internal(format!("Blog error: {error}"))
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            ErrorKind::InvalidToken => AuthError::InvalidToken,
            _ => AuthError::Internal(format!("JWT error: {error}")),
        }
    }
}

impl From<anyhow::Error> for AuthError {
    fn from(error: anyhow::Error) -> Self {
        AuthError::Internal(format!("Internal error: {error}"))
    }
}

/// Authenticated user extractor
#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    /// User ID
    pub user_id: i64,
    /// Username
    pub username: String,
    /// User role
    pub role: String,
    /// Display name (optional)
    #[serde(default)]
    pub display_name: Option<String>,
}