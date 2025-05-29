//! Credentials module for secure storage of sensitive information
//!
//! This module provides a more secure way to store and retrieve sensitive
//! information like API tokens, using the system's keyring when available
//! or falling back to an encrypted file.

use anyhow::{Context, Result};
use im::HashMap;
use once_cell::sync::Lazy;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// Constants for credential storage
const CREDENTIAL_SERVICE: &str = "cv-generator";
const GITHUB_TOKEN_KEY: &str = "github-token";
const FALLBACK_FILE: &str = ".cv-credentials";

// In-memory cache of credentials to avoid repeated keyring access
static CREDENTIALS_CACHE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Gets the path to the fallback credentials file
///
/// This function returns the path to the fallback credentials file,
/// which is located in the user's home directory.
///
/// # Returns
///
/// A Result containing the path to the fallback credentials file
fn get_fallback_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home_dir.join(FALLBACK_FILE))
}

/// Stores a credential in the system's keyring or fallback file
///
/// This function tries to store the credential in the system's keyring.
/// If that fails, it falls back to storing it in an encrypted file.
///
/// # Arguments
///
/// * `key` - The key to store the credential under
/// * `value` - The credential value to store
///
/// # Returns
///
/// A Result indicating success or failure
pub fn store_credential(key: &str, value: &str) -> Result<()> {
    // First, update the in-memory cache
    {
        let mut cache = CREDENTIALS_CACHE.lock().unwrap();
        cache.insert(key.to_string(), value.to_string());
    }

    // Try to store in the system keyring
    match keyring::Entry::new(CREDENTIAL_SERVICE, key) {
        Ok(entry) => match entry.set_password(value) {
            Ok(_) => return Ok(()),
            Err(e) => println!("Warning: Failed to store credential in keyring: {}", e),
        },
        Err(e) => println!("Warning: Failed to access keyring: {}", e),
    }

    // Fall back to encrypted file storage
    store_in_file(key, value)
}

/// Retrieves a credential from the system's keyring or fallback file
///
/// This function first checks the in-memory cache, then tries to retrieve
/// the credential from the system's keyring. If that fails, it falls back
/// to retrieving it from an encrypted file.
///
/// # Arguments
///
/// * `key` - The key to retrieve the credential for
///
/// # Returns
///
/// A Result containing the credential value if found
pub fn get_credential(key: &str) -> Result<Option<String>> {
    // First, check the in-memory cache
    {
        let cache = CREDENTIALS_CACHE.lock().unwrap();
        if let Some(value) = cache.get(key) {
            return Ok(Some(value.clone()));
        }
    }

    // Try to retrieve from the system keyring
    match keyring::Entry::new(CREDENTIAL_SERVICE, key) {
        Ok(entry) => {
            match entry.get_password() {
                Ok(value) => {
                    // Update the cache
                    {
                        let mut cache = CREDENTIALS_CACHE.lock().unwrap();
                        cache.insert(key.to_string(), value.clone());
                    }
                    return Ok(Some(value));
                }
                Err(keyring::Error::NoEntry) => (),
                Err(e) => println!("Warning: Failed to retrieve credential from keyring: {}", e),
            }
        }
        Err(e) => println!("Warning: Failed to access keyring: {}", e),
    }

    // Fall back to encrypted file storage
    get_from_file(key)
}

/// Removes a credential from the system's keyring and fallback file
///
/// This function tries to remove the credential from the system's keyring
/// and the fallback file.
///
/// # Arguments
///
/// * `key` - The key to remove the credential for
///
/// # Returns
///
/// A Result indicating success or failure
pub fn remove_credential(key: &str) -> Result<()> {
    // Remove from the in-memory cache
    {
        let mut cache = CREDENTIALS_CACHE.lock().unwrap();
        cache.remove(key);
    }

    // Try to remove from the system keyring
    match keyring::Entry::new(CREDENTIAL_SERVICE, key) {
        Ok(entry) => match entry.delete_credential() {
            Ok(_) => (),
            Err(keyring::Error::NoEntry) => (),
            Err(e) => println!("Warning: Failed to remove credential from keyring: {}", e),
        },
        Err(e) => println!("Warning: Failed to access keyring: {}", e),
    }

    // Remove from the fallback file
    remove_from_file(key)
}

/// Stores a credential in an encrypted file
///
/// This function stores the credential in an encrypted file in the user's home directory.
///
/// # Arguments
///
/// * `key` - The key to store the credential under
/// * `value` - The credential value to store
///
/// # Returns
///
/// A Result indicating success or failure
fn store_in_file(key: &str, value: &str) -> Result<()> {
    let file_path = get_fallback_path()?;

    // Read existing credentials or create a new map
    let mut credentials = read_credentials_file(&file_path).unwrap_or_else(|_| HashMap::new());

    // Update the credential
    credentials.insert(key.to_string(), value.to_string());

    // Write the updated credentials back to the file
    write_credentials_file(&file_path, &credentials)
}

/// Retrieves a credential from an encrypted file
///
/// This function retrieves the credential from an encrypted file in the user's home directory.
///
/// # Arguments
///
/// * `key` - The key to retrieve the credential for
///
/// # Returns
///
/// A Result containing the credential value if found
fn get_from_file(key: &str) -> Result<Option<String>> {
    let file_path = get_fallback_path()?;

    // If the file doesn't exist, return None
    if !file_path.exists() {
        return Ok(None);
    }

    // Read the credentials file
    let credentials = read_credentials_file(&file_path)?;

    // Get the credential
    Ok(credentials.get(key).cloned())
}

/// Removes a credential from an encrypted file
///
/// This function removes the credential from an encrypted file in the user's home directory.
///
/// # Arguments
///
/// * `key` - The key to remove the credential for
///
/// # Returns
///
/// A Result indicating success or failure
fn remove_from_file(key: &str) -> Result<()> {
    let file_path = get_fallback_path()?;

    // If the file doesn't exist, there's nothing to remove
    if !file_path.exists() {
        return Ok(());
    }

    // Read existing credentials
    let mut credentials = read_credentials_file(&file_path)?;

    // Remove the credential
    credentials.remove(key);

    // Write the updated credentials back to the file
    write_credentials_file(&file_path, &credentials)
}

/// Reads credentials from an encrypted file
///
/// This function reads and decrypts credentials from a file.
///
/// # Arguments
///
/// * `file_path` - Path to the credentials file
///
/// # Returns
///
/// A Result containing a HashMap of credentials
fn read_credentials_file(file_path: &Path) -> Result<HashMap<String, String>> {
    // If the file doesn't exist, return an error
    if !file_path.exists() {
        return Err(anyhow::anyhow!("Credentials file does not exist"));
    }

    // Read the encrypted file
    let encrypted_data = fs::read(file_path).context("Failed to read credentials file")?;

    // Decrypt the data using a machine-specific key
    let decrypted_data = decrypt_data(&encrypted_data)?;

    // Parse the JSON data
    let credentials: HashMap<String, String> =
        serde_json::from_slice(&decrypted_data).context("Failed to parse credentials file")?;

    Ok(credentials)
}

/// Writes credentials to an encrypted file
///
/// This function encrypts and writes credentials to a file.
///
/// # Arguments
///
/// * `file_path` - Path to the credentials file
/// * `credentials` - HashMap of credentials to write
///
/// # Returns
///
/// A Result indicating success or failure
fn write_credentials_file(file_path: &Path, credentials: &HashMap<String, String>) -> Result<()> {
    // Serialize the credentials to JSON
    let json_data = serde_json::to_vec(credentials).context("Failed to serialize credentials")?;

    // Encrypt the data using a machine-specific key
    let encrypted_data = encrypt_data(&json_data)?;

    // Write the encrypted data to the file
    fs::write(file_path, encrypted_data).context("Failed to write credentials file")?;

    // Set appropriate file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(file_path).context("Failed to get file metadata")?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600); // Read/write for owner only
        fs::set_permissions(file_path, permissions).context("Failed to set file permissions")?;
    }

    Ok(())
}

/// Encrypts data using a machine-specific key
///
/// This function encrypts data using a key derived from machine-specific information.
///
/// # Arguments
///
/// * `data` - Data to encrypt
///
/// # Returns
///
/// A Result containing the encrypted data
fn encrypt_data(data: &[u8]) -> Result<Vec<u8>> {
    // For simplicity, we're using a basic XOR encryption with a machine-specific key
    // In a real-world application, you would use a proper encryption library
    let key = get_machine_key()?;
    let mut encrypted = Vec::with_capacity(data.len());

    for (i, &byte) in data.iter().enumerate() {
        encrypted.push(byte ^ key[i % key.len()]);
    }

    Ok(encrypted)
}

/// Decrypts data using a machine-specific key
///
/// This function decrypts data using a key derived from machine-specific information.
///
/// # Arguments
///
/// * `data` - Data to decrypt
///
/// # Returns
///
/// A Result containing the decrypted data
fn decrypt_data(data: &[u8]) -> Result<Vec<u8>> {
    // The XOR operation is its own inverse, so encryption and decryption are the same
    encrypt_data(data)
}

/// Gets a machine-specific key for encryption/decryption
///
/// This function derives a key from machine-specific information.
///
/// # Returns
///
/// A Result containing the machine-specific key
fn get_machine_key() -> Result<Vec<u8>> {
    // In a real-world application, you would use a more secure method to derive a key
    // For simplicity, we're using a combination of the hostname and username
    let hostname = hostname::get()
        .context("Failed to get hostname")?
        .to_string_lossy()
        .to_string();

    let username = whoami::username().unwrap_or_else(|_| "unknown_user".to_string());

    let combined = format!("{}:{}", hostname, username);
    let mut key = Vec::with_capacity(32);

    // Create a 32-byte key from the combined string
    for i in 0..32 {
        let byte = combined
            .as_bytes()
            .get(i % combined.len())
            .copied()
            .unwrap_or(i as u8);
        key.push(byte);
    }

    Ok(key)
}

/// Convenience function to store the GitHub token
///
/// # Arguments
///
/// * `token` - The GitHub token to store
///
/// # Returns
///
/// A Result indicating success or failure
pub fn store_github_token(token: &str) -> Result<()> {
    store_credential(GITHUB_TOKEN_KEY, token)
}

/// Convenience function to retrieve the GitHub token
///
/// # Returns
///
/// A Result containing the GitHub token if found
pub fn get_github_token() -> Result<Option<String>> {
    get_credential(GITHUB_TOKEN_KEY)
}

/// Convenience function to remove the GitHub token
///
/// # Returns
///
/// A Result indicating success or failure
pub fn remove_github_token() -> Result<()> {
    remove_credential(GITHUB_TOKEN_KEY)
}
