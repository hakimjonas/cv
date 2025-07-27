/*!
 * GitHub OAuth module
 * This module provides GitHub OAuth authentication for the API
 */

use crate::auth::{AuthError, LoginResponse};
use crate::db::repository::UserRole;
use crate::db::{Database, UserRepository};
use anyhow::Result;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use url::Url;

/// GitHub user information
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// GitHub OAuth configuration
#[derive(Debug, Clone)]
pub struct GitHubOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub auth_url: String,
    pub token_url: String,
}

impl GitHubOAuthConfig {
    /// Create a new GitHub OAuth configuration from the unified configuration
    pub fn from_config(config: &crate::unified_config::AppConfig) -> Result<Self> {
        let client_id = match config.github_oauth_client_id() {
            Some(id) => id.to_string(),
            None => {
                warn!("GitHub OAuth client ID not configured. Using placeholder value.");
                warn!("GitHub OAuth login will not work without a valid client ID.");
                info!(
                    "You can create a GitHub OAuth app at https://github.com/settings/developers"
                );
                "your-github-client-id".to_string()
            }
        };

        let client_secret = match config.github_oauth_client_secret() {
            Some(secret) => secret.to_string(),
            None => {
                warn!("GitHub OAuth client secret not configured. Using placeholder value.");
                warn!("GitHub OAuth login will not work without a valid client secret.");
                info!(
                    "You can create a GitHub OAuth app at https://github.com/settings/developers"
                );
                "your-github-client-secret".to_string()
            }
        };

        let redirect_url = config.github_oauth_redirect_url().to_string();

        Ok(Self {
            client_id,
            client_secret,
            redirect_url,
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
        })
    }

    /// Create a new GitHub OAuth configuration from environment variables (legacy method)
    pub fn from_env() -> Result<Self> {
        let client_id =
            env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| "your-github-client-id".to_string());
        let client_secret = env::var("GITHUB_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-github-client-secret".to_string());
        let redirect_url = env::var("GITHUB_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:3002/api/auth/github/callback".to_string());

        Ok(Self {
            client_id,
            client_secret,
            redirect_url,
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
        })
    }
}

/// GitHub OAuth service
pub struct GitHubOAuthService {
    config: GitHubOAuthConfig,
    oauth_client: BasicClient,
    user_repo: Arc<UserRepository>,
    jwt_secret: String,
    token_expiration: i64,
}

impl GitHubOAuthService {
    /// Create a new GitHub OAuth service using the unified configuration
    pub fn new_with_config(
        db: &Database,
        app_config: &crate::unified_config::AppConfig,
        jwt_secret: String,
        token_expiration: i64,
    ) -> Result<Self> {
        let config = GitHubOAuthConfig::from_config(app_config)?;

        // Check if GitHub OAuth is properly configured
        if !app_config.is_github_oauth_configured() {
            warn!("GitHub OAuth is not properly configured. Login with GitHub will not work.");
            info!("To enable GitHub OAuth login, please configure valid GitHub OAuth credentials.");
            debug!("You can create a GitHub OAuth app at https://github.com/settings/developers");
        }

        let oauth_client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?);

        Ok(Self {
            config,
            oauth_client,
            user_repo: db.user_repository().into(),
            jwt_secret,
            token_expiration,
        })
    }

    /// Create a new GitHub OAuth service (legacy method)
    pub fn new(db: &Database, jwt_secret: String, token_expiration: i64) -> Result<Self> {
        let config = GitHubOAuthConfig::from_env()?;

        let oauth_client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone())?);

        warn!("Using legacy GitHubOAuthService::new() method with environment variables.");
        warn!(
            "Consider using GitHubOAuthService::new_with_config() instead for better configuration."
        );

        Ok(Self {
            config,
            oauth_client,
            user_repo: db.user_repository().into(),
            jwt_secret,
            token_expiration,
        })
    }

    /// Generate the GitHub authorization URL
    #[instrument(skip(self))]
    pub fn authorize_url(&self) -> (Url, CsrfToken, PkceCodeVerifier) {
        // Create a PKCE code verifier and challenge
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the authorization URL
        let (auth_url, csrf_token) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read:user".to_string()))
            .add_scope(Scope::new("user:email".to_string()))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        (auth_url, csrf_token, pkce_code_verifier)
    }

    /// Exchange the authorization code for an access token
    #[instrument(skip(self, code, pkce_verifier), err)]
    pub async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<String, AuthError> {
        // Exchange the code for an access token
        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                error!("Failed to exchange code for token: {}", e);
                AuthError::Internal(format!("Failed to exchange code for token: {e}"))
            })?;

        Ok(token_result.access_token().secret().clone())
    }

    /// Get the GitHub user information
    #[instrument(skip(self, access_token), err)]
    pub async fn get_github_user(&self, access_token: &str) -> Result<GitHubUser, AuthError> {
        // Get the user information from GitHub
        let client = reqwest::Client::new();
        let user_response = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("token {access_token}"))
            .header("User-Agent", "cv-app")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get GitHub user: {}", e);
                AuthError::Internal(format!("Failed to get GitHub user: {e}"))
            })?;

        if !user_response.status().is_success() {
            let status = user_response.status();
            let text = user_response.text().await.unwrap_or_default();
            error!("GitHub API error: {} - {}", status, text);
            return Err(AuthError::Internal(format!(
                "GitHub API error: {status} - {text}"
            )));
        }

        let github_user = user_response.json::<GitHubUser>().await.map_err(|e| {
            error!("Failed to parse GitHub user: {}", e);
            AuthError::Internal(format!("Failed to parse GitHub user: {e}"))
        })?;

        // If email is not public, get the email from the email API
        let github_user = if github_user.email.is_none() {
            let email_response = client
                .get("https://api.github.com/user/emails")
                .header("Authorization", format!("token {access_token}"))
                .header("User-Agent", "cv-app")
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to get GitHub emails: {}", e);
                    AuthError::Internal(format!("Failed to get GitHub emails: {e}"))
                })?;

            if !email_response.status().is_success() {
                let status = email_response.status();
                let text = email_response.text().await.unwrap_or_default();
                error!("GitHub API error: {} - {}", status, text);
                return Err(AuthError::Internal(format!(
                    "GitHub API error: {status} - {text}"
                )));
            }

            #[derive(Debug, Deserialize)]
            struct GitHubEmail {
                email: String,
                primary: bool,
                verified: bool,
            }

            let emails = email_response
                .json::<Vec<GitHubEmail>>()
                .await
                .map_err(|e| {
                    error!("Failed to parse GitHub emails: {}", e);
                    AuthError::Internal(format!("Failed to parse GitHub emails: {e}"))
                })?;

            // Find the primary email
            let primary_email = emails
                .iter()
                .find(|e| e.primary && e.verified)
                .or_else(|| emails.iter().find(|e| e.verified))
                .map(|e| e.email.clone());

            GitHubUser {
                email: primary_email,
                ..github_user
            }
        } else {
            github_user
        };

        Ok(github_user)
    }

    /// Login or register a user with GitHub
    #[instrument(skip(self, github_user), err)]
    pub async fn login_with_github(
        &self,
        github_user: GitHubUser,
    ) -> Result<LoginResponse, AuthError> {
        // Check if the user exists
        let username = format!("github_{}", github_user.login);
        let user = self
            .user_repo
            .get_user_by_username(&username)
            .await
            .map_err(|e| {
                error!("Failed to get user: {}", e);
                AuthError::Internal(format!("Failed to get user: {e}"))
            })?;

        let user_id = if let Some(user) = user {
            // User exists, return the user ID
            user.id.ok_or_else(|| {
                error!("User has no ID: {}", username);
                AuthError::Internal("User has no ID".to_string())
            })?
        } else {
            // User doesn't exist, create a new user
            let display_name = github_user
                .name
                .unwrap_or_else(|| github_user.login.clone());
            let email = github_user
                .email
                .unwrap_or_else(|| format!("{}@github.com", github_user.login));

            // Generate a random password (not used for GitHub login)
            let password = uuid::Uuid::new_v4().to_string();

            self.user_repo
                .create_user(
                    &username,
                    &display_name,
                    &email,
                    &password,
                    UserRole::Author,
                )
                .await
                .map_err(|e| {
                    error!("Failed to create user: {}", e);
                    AuthError::Internal(format!("Failed to create user: {e}"))
                })?
        };

        // Generate a JWT token
        let user = self
            .user_repo
            .get_user_by_id(user_id)
            .await
            .map_err(|e| {
                error!("Failed to get user: {}", e);
                AuthError::Internal(format!("Failed to get user: {e}"))
            })?
            .ok_or_else(|| {
                error!("User not found: {}", user_id);
                AuthError::Internal("User not found".to_string())
            })?;

        // Generate a JWT token using the same logic as in AuthService
        let now = chrono::Utc::now();
        let iat = now.timestamp();
        let exp = (now + chrono::Duration::seconds(self.token_expiration)).timestamp();

        let role_str = match user.role {
            UserRole::Admin => "Admin",
            UserRole::Author => "Author",
            UserRole::Editor => "Editor",
            UserRole::Viewer => "Viewer",
        };

        let claims = crate::auth::Claims {
            sub: user_id.to_string(),
            username: user.username.clone(),
            role: role_str.to_string(),
            iat,
            exp,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| {
            error!("Failed to generate token: {}", e);
            AuthError::Internal(format!("Failed to generate token: {e}"))
        })?;

        info!("GitHub user logged in: {}", user.username);
        Ok(LoginResponse {
            token,
            user_id,
            username: user.username,
            display_name: user.display_name,
            role: role_str.to_string(),
        })
    }
}
