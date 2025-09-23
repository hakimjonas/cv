use crate::auth::{AuthError, AuthUser};
use crate::git_identity::{GitIdentity, GitIdentityService};
use crate::unified_config::{AppConfig, OwnerConfig};
use axum::http::HeaderValue;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tracing::{debug, error, info, instrument, warn};

/// Claims for JWT token
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject (user ID)
    sub: String,
    /// Username
    username: String,
    /// User role
    role: String,
    /// Expiration time (Unix timestamp)
    exp: i64,
    /// Issued at (Unix timestamp)
    iat: i64,
}

/// Response for successful authentication
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// JWT token
    pub token: String,
    /// User information
    pub user: AuthUser,
}

/// Simple authentication service that uses Git identity
pub struct SimpleAuthService {
    /// Git identity service
    git_identity_service: GitIdentityService,
    /// JWT secret for token signing
    jwt_secret: String,
    /// Token expiration in seconds
    token_expiration: i64,
    /// Owner configuration
    owner: Option<OwnerConfig>,
    /// Development mode flag
    dev_mode: bool,
}

impl SimpleAuthService {
    /// Creates a new SimpleAuthService
    pub fn new(config: &AppConfig, jwt_secret: String, token_expiration: i64) -> Self {
        Self {
            git_identity_service: GitIdentityService::new(),
            jwt_secret,
            token_expiration,
            owner: config.owner.clone(),
            dev_mode: config.dev_mode,
        }
    }

    /// Creates a session for the Git user
    #[instrument(skip(self))]
    pub async fn create_session(&self) -> Result<AuthResponse, AuthError> {
        // If owner is already configured, use that
        if let Some(owner) = &self.owner {
            return self.create_session_from_owner(owner);
        }

        // Otherwise, try to get identity from Git
        match self.git_identity_service.get_identity() {
            Ok(identity) => self.create_session_from_git_identity(&identity),
            Err(e) => {
                if self.dev_mode {
                    // In dev mode, create a default session if Git identity fails
                    info!(
                        "Dev mode enabled, creating default session despite Git identity error: {}",
                        e
                    );
                    self.create_default_dev_session()
                } else {
                    error!("Failed to get Git identity: {}", e);
                    Err(AuthError::Unauthorized)
                }
            }
        }
    }

    /// Creates a session from owner configuration
    fn create_session_from_owner(&self, owner: &OwnerConfig) -> Result<AuthResponse, AuthError> {
        let display_name = owner.display_name.as_ref().unwrap_or(&owner.name);
        let role = if owner.role.is_empty() {
            "Author".to_string()
        } else {
            owner.role.clone()
        };

        let user = AuthUser {
            user_id: 1, // Single user always has ID 1
            username: owner
                .github_username
                .clone()
                .unwrap_or_else(|| "owner".to_string()),
            display_name: Some(display_name.clone()),
            role,
        };

        let token = self.generate_token(&user)?;

        Ok(AuthResponse { token, user })
    }

    /// Creates a session from Git identity
    fn create_session_from_git_identity(
        &self,
        identity: &GitIdentity,
    ) -> Result<AuthResponse, AuthError> {
        let username = identity.github_username.clone().unwrap_or_else(|| {
            // Use email username part if GitHub username is not available
            identity
                .email
                .split('@')
                .next()
                .unwrap_or("user")
                .to_string()
        });

        let user = AuthUser {
            user_id: 1, // Single user always has ID 1
            username,
            display_name: Some(identity.name.clone()),
            role: "Author".to_string(), // Default role is Author
        };

        let token = self.generate_token(&user)?;

        Ok(AuthResponse { token, user })
    }

    /// Creates a default session for development mode
    fn create_default_dev_session(&self) -> Result<AuthResponse, AuthError> {
        let user = AuthUser {
            user_id: 1,
            username: "dev_user".to_string(),
            display_name: Some("Development User".to_string()),
            role: "Author".to_string(),
        };

        let token = self.generate_token(&user)?;

        Ok(AuthResponse { token, user })
    }

    /// Generates a JWT token for the user
    fn generate_token(&self, user: &AuthUser) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiration = now + Duration::seconds(self.token_expiration);

        let claims = Claims {
            sub: user.user_id.to_string(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        };

        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_bytes());
        encode(&Header::default(), &claims, &encoding_key).map_err(|e| {
            error!("Failed to generate token: {}", e);
            AuthError::Internal(format!("Token generation error: {e}"))
        })
    }

    /// Validates a JWT token
    pub fn validate_token(&self, token: &str) -> Result<AuthUser, AuthError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let validation = Validation::default();

        let token_data =
            decode::<Claims>(token, &decoding_key, &validation).map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => {
                    warn!("Invalid token: {}", e);
                    AuthError::InvalidToken
                }
            })?;

        let claims = token_data.claims;

        let username = claims.username;
        let user = AuthUser {
            user_id: claims.sub.parse().unwrap_or(1),
            username: username.clone(),
            display_name: Some(username),
            role: claims.role,
        };

        Ok(user)
    }

    /// Automatically authenticates for localhost/development mode
    pub fn auto_authenticate_for_dev(&self, remote_addr: Option<IpAddr>) -> Option<AuthUser> {
        if !self.dev_mode {
            return None;
        }

        // Check if the request is from localhost
        let is_localhost = remote_addr.map(|addr| addr.is_loopback()).unwrap_or(false);

        if is_localhost {
            debug!("Auto-authenticating for localhost in dev mode");

            // Create a default user for development
            Some(AuthUser {
                user_id: 1,
                username: "dev_user".to_string(),
                display_name: Some("Development User".to_string()),
                role: "Author".to_string(),
            })
        } else {
            None
        }
    }
}

/// Extracts and validates the authentication token from the request headers
///
/// Note: This function is marked as async for API consistency, even though it doesn't
/// perform any asynchronous operations internally. It's awaited in middleware functions.
pub async fn extract_and_validate_token(
    auth_service: &SimpleAuthService,
    auth_header: Option<&HeaderValue>,
    remote_addr: Option<IpAddr>,
) -> Result<AuthUser, AuthError> {
    // Try auto-authentication for development mode first
    if let Some(user) = auth_service.auto_authenticate_for_dev(remote_addr) {
        return Ok(user);
    }

    // Otherwise, extract and validate the token
    let auth_header = auth_header.ok_or(AuthError::MissingToken)?;
    let auth_str = auth_header.to_str().map_err(|_| AuthError::InvalidToken)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidToken);
    }

    let token = &auth_str[7..]; // Remove "Bearer " prefix
    auth_service.validate_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_auto_authenticate_for_dev() {
        let config = AppConfig {
            dev_mode: true,
            ..Default::default()
        };

        let service = SimpleAuthService::new(&config, "test_secret".to_string(), 3600);

        // Test with localhost
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let user = service.auto_authenticate_for_dev(Some(localhost));
        assert!(user.is_some());

        // Test with non-localhost
        let non_localhost = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let user = service.auto_authenticate_for_dev(Some(non_localhost));
        assert!(user.is_none());
    }

    #[test]
    fn test_generate_and_validate_token() {
        let config = AppConfig::default();
        let service = SimpleAuthService::new(&config, "test_secret".to_string(), 3600);

        let user = AuthUser {
            user_id: 1,
            username: "test_user".to_string(),
            display_name: Some("Test User".to_string()),
            role: "Author".to_string(),
        };

        let token = service.generate_token(&user).unwrap();
        let validated_user = service.validate_token(&token).unwrap();

        assert_eq!(validated_user.user_id, user.user_id);
        assert_eq!(validated_user.username, user.username);
        assert_eq!(validated_user.role, user.role);
    }
}
