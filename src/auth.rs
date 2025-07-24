/*!
 * Authentication module
 * This module provides JWT-based authentication for the API
 */

use crate::blog_error::BlogError;
use crate::db::{Database, UserRepository};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// User role
    pub role: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
}

/// Login request structure
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Login response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// JWT token
    pub token: String,
    /// User ID
    pub user_id: i64,
    /// Username
    pub username: String,
    /// Display name
    pub display_name: String,
    /// User role
    pub role: String,
}

/// Register request structure
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Username
    pub username: String,
    /// Display name
    pub display_name: String,
    /// Email
    pub email: String,
    /// Password
    pub password: String,
}

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
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
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
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => AuthError::InvalidToken,
            _ => AuthError::Internal(format!("JWT error: {error}")),
        }
    }
}

impl From<anyhow::Error> for AuthError {
    fn from(error: anyhow::Error) -> Self {
        AuthError::Internal(format!("Internal error: {}", error))
    }
}

/// Authentication service
pub struct AuthService {
    /// JWT secret key
    jwt_secret: String,
    /// JWT token expiration in seconds
    token_expiration: i64,
    /// User repository
    user_repo: UserRepository,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(db: &Database, jwt_secret: String, token_expiration: i64) -> Self {
        Self {
            jwt_secret,
            token_expiration,
            user_repo: db.user_repository(),
        }
    }

    /// Login a user with username and password
    #[instrument(skip(self, password), err)]
    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResponse, AuthError> {
        // Authenticate the user
        let user = match self.user_repo.authenticate(username, password).await? {
            Some(user) => user,
            None => {
                warn!("Authentication failed for user: {}", username);
                return Err(AuthError::InvalidCredentials);
            }
        };

        // Get the user ID
        let user_id = match user.id {
            Some(id) => id,
            None => {
                error!("User has no ID: {}", username);
                return Err(AuthError::Internal("User has no ID".to_string()));
            }
        };

        // Generate a JWT token
        let token = self.generate_token(user_id, &user.username, &user.role).await?;

        // Convert role to string
        let role_str = match user.role {
            crate::db::repository::UserRole::Admin => "Admin",
            crate::db::repository::UserRole::Author => "Author",
            crate::db::repository::UserRole::Editor => "Editor",
            crate::db::repository::UserRole::Viewer => "Viewer",
        };

        info!("User logged in: {}", username);
        Ok(LoginResponse {
            token,
            user_id,
            username: user.username,
            display_name: user.display_name,
            role: role_str.to_string(),
        })
    }

    /// Register a new user
    #[instrument(skip(self, password), err)]
    pub async fn register(
        &self,
        username: &str,
        display_name: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, AuthError> {
        // Check if the username already exists
        if self.user_repo.get_user_by_username(username).await?.is_some() {
            warn!("Username already exists: {}", username);
            return Err(AuthError::Internal("Username already exists".to_string()));
        }

        // Create the user
        let user_id = self
            .user_repo
            .create_user(
                username,
                display_name,
                email,
                password,
                crate::db::repository::UserRole::Author, // Default role is Author
            )
            .await?;

        // Generate a JWT token
        let token = self
            .generate_token(user_id, username, &crate::db::repository::UserRole::Author)
            .await?;

        info!("User registered: {}", username);
        Ok(LoginResponse {
            token,
            user_id,
            username: username.to_string(),
            display_name: display_name.to_string(),
            role: "Author".to_string(),
        })
    }

    /// Generate a JWT token
    #[instrument(skip(self), err)]
    async fn generate_token(
        &self,
        user_id: i64,
        username: &str,
        role: &crate::db::repository::UserRole,
    ) -> Result<String, AuthError> {
        // Get the current time
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = (now + Duration::seconds(self.token_expiration)).timestamp();

        // Convert role to string
        let role_str = match role {
            crate::db::repository::UserRole::Admin => "Admin",
            crate::db::repository::UserRole::Author => "Author",
            crate::db::repository::UserRole::Editor => "Editor",
            crate::db::repository::UserRole::Viewer => "Viewer",
        };

        // Create the claims
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            role: role_str.to_string(),
            iat,
            exp,
        };

        // Encode the token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::Internal(format!("Failed to generate token: {}", e)))?;

        debug!("Generated JWT token for user: {}", username);
        Ok(token)
    }

    /// Validate a JWT token
    #[instrument(skip(self, token), err)]
    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        // Decode the token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;

        debug!("Validated JWT token for user: {}", token_data.claims.username);
        Ok(token_data.claims)
    }
}

/// Authenticated user extractor
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// User ID
    pub user_id: i64,
    /// Username
    pub username: String,
    /// User role
    pub role: String,
}

/// Helper function to extract and validate a JWT token from an Authorization header
pub async fn extract_and_validate_token(
    auth_service: &AuthService,
    auth_header: Option<&axum::http::HeaderValue>,
) -> Result<AuthUser, AuthError> {
    // Get the authorization header
    let auth_header = auth_header.ok_or(AuthError::MissingToken)?;

    // Extract the token
    let auth_header_str = auth_header
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?;

    // Check if the header starts with "Bearer "
    if !auth_header_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidToken);
    }

    // Extract the token
    let token = &auth_header_str[7..];

    // Validate the token
    let claims = auth_service.validate_token(token).await?;

    // Convert user_id from string to i64
    let user_id = claims.sub.parse::<i64>()
        .map_err(|_| AuthError::InvalidToken)?;

    // Create the authenticated user
    Ok(AuthUser {
        user_id,
        username: claims.username,
        role: claims.role,
    })
}

/// Login handler
#[instrument(skip(auth_service, login_request), err)]
pub async fn login_handler(
    State(auth_service): State<Arc<AuthService>>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let response = auth_service
        .login(&login_request.username, &login_request.password)
        .await?;
    Ok(Json(response))
}

/// Register handler
#[instrument(skip(auth_service, register_request), err)]
pub async fn register_handler(
    State(auth_service): State<Arc<AuthService>>,
    Json(register_request): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let response = auth_service
        .register(
            &register_request.username,
            &register_request.display_name,
            &register_request.email,
            &register_request.password,
        )
        .await?;
    Ok(Json(response))
}

/// Get current user handler
#[instrument(skip(auth_user), err)]
pub async fn get_current_user_handler(auth_user: AuthUser) -> Result<Json<AuthUser>, AuthError> {
    Ok(Json(auth_user))
}