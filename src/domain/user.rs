/// User domain models
///
/// This module contains user-related domain entities and authentication logic.
use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};

use super::blog::UserId;

/// User role in the system
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UserRole {
    /// Administrator with full access
    Admin,
    /// Author who can create and edit their own posts
    Author,
    /// Editor who can edit but not create posts
    Editor,
    /// Viewer who can only view content
    Viewer,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Author
    }
}

impl UserRole {
    /// Check if this role can create posts
    pub fn can_create_posts(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Author)
    }

    /// Check if this role can edit posts
    pub fn can_edit_posts(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Author | UserRole::Editor)
    }

    /// Check if this role can delete posts
    pub fn can_delete_posts(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if this role can manage users
    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

/// User account in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    /// Unique identifier for the user (None for new users)
    pub id: Option<UserId>,

    /// Username for login
    pub username: String,

    /// Display name of the user
    pub display_name: String,

    /// Email address
    pub email: String,

    /// Password hash (never returned in API responses)
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// User role for authorization
    pub role: UserRole,

    /// When the user was created (ISO 8601 format)
    pub created_at: String,

    /// When the user was last updated (ISO 8601 format)
    pub updated_at: String,
}

impl User {
    /// Creates a new user with default values
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: None,
            username: String::new(),
            display_name: String::new(),
            email: String::new(),
            password_hash: String::new(),
            role: UserRole::default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Creates a new user with credentials
    pub fn with_credentials(
        username: &str,
        display_name: &str,
        email: &str,
        password: &str,
    ) -> Result<Self> {
        let password_hash = Self::hash_password(password)?;

        Ok(Self {
            username: username.to_string(),
            display_name: display_name.to_string(),
            email: email.to_string(),
            password_hash,
            ..Self::new()
        })
    }

    /// Builder pattern methods for immutable updates
    pub fn with_username(self, username: &str) -> Self {
        Self {
            username: username.to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..self
        }
    }

    pub fn with_display_name(self, display_name: &str) -> Self {
        Self {
            display_name: display_name.to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..self
        }
    }

    pub fn with_email(self, email: &str) -> Self {
        Self {
            email: email.to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..self
        }
    }

    pub fn with_password(self, password: &str) -> Result<Self> {
        let password_hash = Self::hash_password(password)?;
        Ok(Self {
            password_hash,
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..self
        })
    }

    pub fn with_role(self, role: UserRole) -> Self {
        Self {
            role,
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..self
        }
    }

    /// Hashes a password using Argon2
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
    }

    /// Verifies a password against the stored hash
    pub fn verify_password(&self, password: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(&self.password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse stored password hash: {}", e))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Validates the user data
    pub fn validate(&self) -> Result<()> {
        if self.username.trim().is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }

        if self.username.len() < 3 {
            return Err(anyhow::anyhow!("Username must be at least 3 characters"));
        }

        if self.display_name.trim().is_empty() {
            return Err(anyhow::anyhow!("Display name cannot be empty"));
        }

        if self.email.trim().is_empty() {
            return Err(anyhow::anyhow!("Email cannot be empty"));
        }

        // Basic email validation
        if !self.email.contains('@') || !self.email.contains('.') {
            return Err(anyhow::anyhow!("Invalid email format"));
        }

        if self.password_hash.trim().is_empty() {
            return Err(anyhow::anyhow!("Password hash cannot be empty"));
        }

        Ok(())
    }

    /// Returns a sanitized version of the user for public API responses
    pub fn public_view(self) -> Self {
        Self {
            password_hash: String::new(), // Clear the password hash
            ..self
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user =
            User::with_credentials("testuser", "Test User", "test@example.com", "password123")
                .expect("Failed to create user");

        assert_eq!(user.username, "testuser");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.password_hash.is_empty());
        assert_eq!(user.role, UserRole::Author);
    }

    #[test]
    fn test_password_verification() {
        let user =
            User::with_credentials("testuser", "Test User", "test@example.com", "password123")
                .expect("Failed to create user");

        assert!(user.verify_password("password123").unwrap());
        assert!(!user.verify_password("wrongpassword").unwrap());
    }

    #[test]
    fn test_user_roles() {
        assert!(UserRole::Admin.can_create_posts());
        assert!(UserRole::Admin.can_edit_posts());
        assert!(UserRole::Admin.can_delete_posts());
        assert!(UserRole::Admin.can_manage_users());

        assert!(UserRole::Author.can_create_posts());
        assert!(UserRole::Author.can_edit_posts());
        assert!(!UserRole::Author.can_delete_posts());
        assert!(!UserRole::Author.can_manage_users());

        assert!(!UserRole::Editor.can_create_posts());
        assert!(UserRole::Editor.can_edit_posts());
        assert!(!UserRole::Editor.can_delete_posts());
        assert!(!UserRole::Editor.can_manage_users());

        assert!(!UserRole::Viewer.can_create_posts());
        assert!(!UserRole::Viewer.can_edit_posts());
        assert!(!UserRole::Viewer.can_delete_posts());
        assert!(!UserRole::Viewer.can_manage_users());
    }

    #[test]
    fn test_user_validation() {
        let valid_user =
            User::with_credentials("testuser", "Test User", "test@example.com", "password123")
                .expect("Failed to create user");
        assert!(valid_user.validate().is_ok());

        let invalid_user = User::new();
        assert!(invalid_user.validate().is_err());

        let user_with_short_username = User::new().with_username("ab");
        assert!(user_with_short_username.validate().is_err());

        let user_with_invalid_email = User::new()
            .with_username("testuser")
            .with_display_name("Test User")
            .with_email("invalid-email");
        assert!(user_with_invalid_email.validate().is_err());
    }

    #[test]
    fn test_public_view() {
        let user =
            User::with_credentials("testuser", "Test User", "test@example.com", "password123")
                .expect("Failed to create user");

        let public_user = user.public_view();
        assert!(public_user.password_hash.is_empty());
        assert_eq!(public_user.username, "testuser");
    }
}
