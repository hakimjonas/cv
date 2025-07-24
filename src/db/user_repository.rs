/*!
 * User repository implementation
 * This module provides a clean interface for user-related database operations
 * following functional programming principles
 */

use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use im::Vector;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, params};
use std::sync::Arc;
use tokio::task;
use tracing::{debug, error, info, instrument};

use super::repository::UserRole;

/// User in the system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: String,
    pub updated_at: String,
}

/// Repository for user operations
#[allow(dead_code)]
pub struct UserRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

#[allow(dead_code)]
/// Implementation of the UserRepository
///
/// The UserRepository provides a high-level API for working with users
/// in the database. It handles user authentication, creation, and management.
///
/// ## Architecture
///
/// The repository follows a layered architecture:
///
/// 1. **Public API Methods**: Async methods that provide the main interface for CRUD operations
///    (e.g., `get_all_users`, `save_user`, `update_user`, `delete_user`).
///
/// 2. **Transaction Methods**: Helper methods that work within database transactions to ensure
///    atomicity of operations.
///
/// 3. **Authentication Methods**: Methods for user authentication and password management.
///
/// ## Async Implementation
///
/// The repository uses `task::spawn_blocking` to execute SQLite operations asynchronously,
/// preventing blocking of the async runtime. This is necessary because SQLite operations
/// are inherently blocking.
impl UserRepository {
    /// Create a new repository with the given connection pool
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// Get all users
    #[instrument(skip(self), err)]
    pub async fn get_all_users(&self) -> Result<Vector<User>> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut stmt = conn.prepare(
                "SELECT id, username, display_name, email, password_hash, role, created_at, updated_at 
                 FROM users ORDER BY username"
            )?;

            let user_iter = stmt.query_map([], |row| {
                let id = row.get(0)?;
                let username = row.get(1)?;
                let display_name = row.get(2)?;
                let email = row.get(3)?;
                let password_hash = row.get(4)?;
                let role_str: String = row.get(5)?;
                let created_at = row.get(6)?;
                let updated_at = row.get(7)?;

                // Convert role string to UserRole enum
                let role = match role_str.as_str() {
                    "Admin" => UserRole::Admin,
                    "Author" => UserRole::Author,
                    "Editor" => UserRole::Editor,
                    "Viewer" => UserRole::Viewer,
                    _ => UserRole::Author, // Default to Author if unknown
                };

                Ok(User {
                    id: Some(id),
                    username,
                    display_name,
                    email,
                    password_hash,
                    role,
                    created_at,
                    updated_at,
                })
            })?;

            // Use functional approach to collect users
            let users = user_iter
                .map(|user_result| user_result.map_err(anyhow::Error::from))
                .collect::<Result<Vector<_>>>()?;

            debug!("Loaded {} users", users.len());
            Ok(users)
        })
        .await?
    }

    /// Get a user by ID
    #[instrument(skip(self), err)]
    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let user_result = conn
                .query_row(
                    "SELECT id, username, display_name, email, password_hash, role, created_at, updated_at 
                     FROM users WHERE id = ?1",
                    [id],
                    |row| {
                        let id = row.get(0)?;
                        let username = row.get(1)?;
                        let display_name = row.get(2)?;
                        let email = row.get(3)?;
                        let password_hash = row.get(4)?;
                        let role_str: String = row.get(5)?;
                        let created_at = row.get(6)?;
                        let updated_at = row.get(7)?;

                        // Convert role string to UserRole enum
                        let role = match role_str.as_str() {
                            "Admin" => UserRole::Admin,
                            "Author" => UserRole::Author,
                            "Editor" => UserRole::Editor,
                            "Viewer" => UserRole::Viewer,
                            _ => UserRole::Author, // Default to Author if unknown
                        };

                        Ok(User {
                            id: Some(id),
                            username,
                            display_name,
                            email,
                            password_hash,
                            role,
                            created_at,
                            updated_at,
                        })
                    },
                )
                .optional()?;

            match user_result {
                Some(user) => {
                    debug!("Loaded user with ID: {}", id);
                    Ok(Some(user))
                }
                None => {
                    debug!("No user found with ID: {}", id);
                    Ok(None)
                }
            }
        })
        .await?
    }

    /// Get a user by username
    #[instrument(skip(self), err)]
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let username = username.to_string();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let user_result = conn
                .query_row(
                    "SELECT id, username, display_name, email, password_hash, role, created_at, updated_at 
                     FROM users WHERE username = ?1",
                    [&username],
                    |row| {
                        let id = row.get(0)?;
                        let username = row.get(1)?;
                        let display_name = row.get(2)?;
                        let email = row.get(3)?;
                        let password_hash = row.get(4)?;
                        let role_str: String = row.get(5)?;
                        let created_at = row.get(6)?;
                        let updated_at = row.get(7)?;

                        // Convert role string to UserRole enum
                        let role = match role_str.as_str() {
                            "Admin" => UserRole::Admin,
                            "Author" => UserRole::Author,
                            "Editor" => UserRole::Editor,
                            "Viewer" => UserRole::Viewer,
                            _ => UserRole::Author, // Default to Author if unknown
                        };

                        Ok(User {
                            id: Some(id),
                            username,
                            display_name,
                            email,
                            password_hash,
                            role,
                            created_at,
                            updated_at,
                        })
                    },
                )
                .optional()?;

            match user_result {
                Some(user) => {
                    debug!("Loaded user with username: {}", username);
                    Ok(Some(user))
                }
                None => {
                    debug!("No user found with username: {}", username);
                    Ok(None)
                }
            }
        })
        .await?
    }

    /// Save a new user
    #[instrument(skip(self, user), err)]
    pub async fn save_user(&self, user: &User) -> Result<i64> {
        let user = user.clone();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            // Convert role enum to string
            let role_str = match user.role {
                UserRole::Admin => "Admin",
                UserRole::Author => "Author",
                UserRole::Editor => "Editor",
                UserRole::Viewer => "Viewer",
            };

            tx.execute(
                "INSERT INTO users (username, display_name, email, password_hash, role, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    &user.username,
                    &user.display_name,
                    &user.email,
                    &user.password_hash,
                    role_str,
                    &user.created_at,
                    &user.updated_at
                ],
            )?;

            let user_id = tx.last_insert_rowid();

            // Commit the transaction
            match tx.commit() {
                Ok(_) => {
                    info!("Created user with ID: {}", user_id);
                    Ok(user_id)
                }
                Err(e) => {
                    error!("Failed to commit transaction: {}", e);
                    Err(anyhow!("Failed to commit transaction: {}", e))
                }
            }
        })
        .await?
    }

    /// Update an existing user
    #[instrument(skip(self, user), err)]
    pub async fn update_user(&self, user: &User) -> Result<()> {
        let user = user.clone();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            let user_id = match user.id {
                Some(id) => id,
                None => return Err(anyhow!("Cannot update user without ID")),
            };

            // Convert role enum to string
            let role_str = match user.role {
                UserRole::Admin => "Admin",
                UserRole::Author => "Author",
                UserRole::Editor => "Editor",
                UserRole::Viewer => "Viewer",
            };

            tx.execute(
                "UPDATE users SET 
                 username = ?1, display_name = ?2, email = ?3, password_hash = ?4, 
                 role = ?5, updated_at = ?6
                 WHERE id = ?7",
                params![
                    &user.username,
                    &user.display_name,
                    &user.email,
                    &user.password_hash,
                    role_str,
                    &user.updated_at,
                    user_id
                ],
            )?;

            // Commit the transaction
            match tx.commit() {
                Ok(_) => {
                    info!("Updated user with ID: {}", user_id);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to commit transaction: {}", e);
                    Err(anyhow!("Failed to commit transaction: {}", e))
                }
            }
        })
        .await?
    }

    /// Delete a user
    #[instrument(skip(self), err)]
    pub async fn delete_user(&self, user_id: i64) -> Result<()> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            conn.execute("DELETE FROM users WHERE id = ?1", [user_id])?;
            info!("Deleted user with ID: {}", user_id);
            Ok(())
        })
        .await?
    }

    /// Authenticate a user with username and password
    #[instrument(skip(self, password), err)]
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<Option<User>> {
        // Get the user by username
        let user = self.get_user_by_username(username).await?;

        // If user not found, return None
        let user = match user {
            Some(user) => user,
            None => return Ok(None),
        };

        // Verify the password
        let is_valid = self.verify_password(&user, password).await?;

        if is_valid {
            debug!("Authentication successful for user: {}", username);
            Ok(Some(user))
        } else {
            debug!("Authentication failed for user: {}", username);
            Ok(None)
        }
    }

    /// Hash a password using Argon2
    #[instrument(skip(self, password), err)]
    pub async fn hash_password(&self, password: &str) -> Result<String> {
        let password = password.to_string();

        task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();

            argon2
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
                .map_err(|e| anyhow!("Password hashing error: {}", e))
        })
        .await?
    }

    /// Verify a password against the stored hash
    #[instrument(skip(self, user, password), err)]
    pub async fn verify_password(&self, user: &User, password: &str) -> Result<bool> {
        let password = password.to_string();
        let password_hash = user.password_hash.clone();

        task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&password_hash)
                .map_err(|e| anyhow!("Password hash parsing error: {}", e))?;

            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await?
    }

    /// Create a new user with the given credentials
    #[instrument(skip(self, password), err)]
    pub async fn create_user(
        &self,
        username: &str,
        display_name: &str,
        email: &str,
        password: &str,
        role: UserRole,
    ) -> Result<i64> {
        // Hash the password
        let password_hash = self.hash_password(password).await?;

        // Create the user
        let now = chrono::Local::now().to_rfc3339();
        let user = User {
            id: None,
            username: username.to_string(),
            display_name: display_name.to_string(),
            email: email.to_string(),
            password_hash,
            role,
            created_at: now.clone(),
            updated_at: now,
        };

        // Save the user
        self.save_user(&user).await
    }

    /// Change a user's password
    #[instrument(skip(self, user_id, new_password), err)]
    pub async fn change_password(&self, user_id: i64, new_password: &str) -> Result<()> {
        // Get the user
        let user = match self.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(anyhow!("User not found")),
        };

        // Hash the new password
        let password_hash = self.hash_password(new_password).await?;

        // Update the user
        let updated_user = User {
            password_hash,
            updated_at: chrono::Local::now().to_rfc3339(),
            ..user
        };

        // Save the updated user
        self.update_user(&updated_user).await
    }
}
