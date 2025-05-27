//! Git config module for the CV generator
//!
//! This module provides functions for reading and writing git config values,
//! specifically for storing and retrieving the GitHub API token.
//!
//! For production deployment, GitHub Secrets can be used to store the token securely.
//! GitHub Secrets are encrypted environment variables that can be used in GitHub Actions workflows.
//! The token can be accessed in the workflow using the `${{ secrets.GITHUB_TOKEN }}` syntax.

use anyhow::{Context, Result};
use git2::Config;

// Extension trait to enable method chaining with pipe
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

// Implement Pipe for Config to enable method chaining
impl Pipe for Config {}

/// Git config key for the GitHub API token
pub const GITHUB_TOKEN_KEY: &str = "cv.github.token";

/// Reads the GitHub API token from git config
///
/// This function tries to read the GitHub API token from the git config.
/// It first tries to read from the repository-specific config, then falls back to the global config.
///
/// # Returns
///
/// A Result containing the token as a String if found, or an error if not found or if there was an error reading the config.
pub fn read_github_token() -> Result<Option<String>> {
    // Try to open the git config
    let config = Config::open_default().context("Failed to open git config")?;

    // Try to read the token from the config
    match config.get_string(GITHUB_TOKEN_KEY) {
        Ok(token) => Ok(Some(token)),
        Err(_) => Ok(None),
    }
}

/// Writes the GitHub API token to git config
///
/// This function writes the GitHub API token to the git config.
/// It writes to the global config to ensure the token is available for all repositories.
///
/// # Arguments
///
/// * `token` - The GitHub API token to write
///
/// # Returns
///
/// A Result indicating success or failure
pub fn write_github_token(token: &str) -> Result<()> {
    // Open the global git config and write the token
    Config::open_default()
        .context("Failed to open git config")?
        .pipe(|mut config| {
            config
                .set_str(GITHUB_TOKEN_KEY, token)
                .context("Failed to write token to git config")
        })
}

/// Removes the GitHub API token from git config
///
/// This function removes the GitHub API token from the git config.
///
/// # Returns
///
/// A Result indicating success or failure
pub fn remove_github_token() -> Result<()> {
    // Open the git config and remove the token
    Config::open_default()
        .context("Failed to open git config")?
        .pipe(|mut config| {
            // Remove the token from the config
            match config.remove(GITHUB_TOKEN_KEY) {
                Ok(_) => Ok(()),
                Err(_) => Ok(()), // Ignore errors if the key doesn't exist
            }
        })
}
