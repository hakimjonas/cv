use anyhow::{Context, Result};
use keyring::Entry;
use tracing::{debug, error, info};

const SERVICE_NAME: &str = "cv_github_token";
const USERNAME: &str = "github";

/// Gets the GitHub token from secure storage
///
/// # Returns
///
/// A Result containing an Option with the token if found
pub fn get_github_token() -> Result<Option<String>> {
    debug!("Retrieving GitHub token from secure storage");
    let keyring = Entry::new(SERVICE_NAME, USERNAME)
        .context("Failed to create keyring entry for GitHub token")?;

    match keyring.get_password() {
        Ok(token) => {
            debug!("Successfully retrieved GitHub token from secure storage");
            Ok(Some(token))
        }
        Err(keyring::Error::NoEntry) => {
            debug!("No GitHub token found in secure storage");
            Ok(None)
        }
        Err(e) => {
            error!("Failed to retrieve GitHub token from secure storage: {}", e);
            Err(e.into())
        }
    }
}

/// Stores a GitHub token in secure storage
///
/// # Arguments
///
/// * `token` - The GitHub token to store
///
/// # Returns
///
/// A Result indicating success or failure
pub fn store_github_token(token: &str) -> Result<()> {
    debug!("Storing GitHub token in secure storage");
    let keyring = Entry::new(SERVICE_NAME, USERNAME)
        .context("Failed to create keyring entry for GitHub token")?;

    keyring
        .set_password(token)
        .context("Failed to store GitHub token in secure storage")?;

    info!("Successfully stored GitHub token in secure storage");
    Ok(())
}

/// Removes the GitHub token from secure storage
///
/// # Returns
///
/// A Result indicating success or failure
pub fn remove_github_token() -> Result<()> {
    debug!("Removing GitHub token from secure storage");
    let keyring = Entry::new(SERVICE_NAME, USERNAME)
        .context("Failed to create keyring entry for GitHub token")?;

    // Check if the token exists
    match keyring.get_password() {
        Ok(_) => {
            // Token exists, set it to an empty string to effectively remove it
            keyring
                .set_password("")
                .context("Failed to remove GitHub token from secure storage")?;
            info!("Successfully removed GitHub token from secure storage");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            // Token doesn't exist, nothing to remove
            info!("No GitHub token found in secure storage to remove");
            Ok(())
        }
        Err(e) => {
            // Other error
            error!("Failed to check for GitHub token in secure storage: {}", e);
            Err(e.into())
        }
    }
}
