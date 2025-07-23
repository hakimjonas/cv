//! Utility to check and secure database permissions
//!
//! This module provides functions to verify and secure database file and directory permissions

use anyhow::Result;
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Checks if a database file is accessible and writable
pub fn check_db_permissions(db_path: &Path) -> Result<()> {
    println!("Checking database permissions for: {db_path:?}");

    // Check if the parent directory exists and is writable
    if let Some(parent_dir) = db_path.parent() {
        check_directory_permissions(parent_dir)?;
    }

    // If the database file already exists, check if it's writable
    if db_path.exists() {
        check_file_permissions(db_path)?;
    } else {
        // If it doesn't exist, try to create it
        println!("Database file doesn't exist yet, will be created by SQLite");
    }

    // Try to open the database in read-write mode
    match rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
    ) {
        Ok(_) => {
            println!("✅ Successfully opened database connection in read-write mode");
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to open database in read-write mode: {e}");
            Err(anyhow::anyhow!(
                "Cannot open database in read-write mode: {}",
                e
            ))
        }
    }
}

/// Checks if a directory exists and is writable
pub fn check_directory_permissions(dir_path: &Path) -> Result<()> {
    println!("Checking directory permissions for: {dir_path:?}");

    if !dir_path.exists() {
        println!("❌ Directory doesn't exist: {dir_path:?}");
        return Err(anyhow::anyhow!("Directory doesn't exist: {:?}", dir_path));
    }

    if !dir_path.is_dir() {
        println!("❌ Path is not a directory: {dir_path:?}");
        return Err(anyhow::anyhow!("Path is not a directory: {:?}", dir_path));
    }

    // Check if the directory is writable by creating a temporary file
    let test_file_path = dir_path.join(format!("write_test_{}.tmp", std::process::id()));
    match std::fs::File::create(&test_file_path) {
        Ok(_) => {
            println!("✅ Directory is writable: {dir_path:?}");
            // Clean up test file
            if let Err(e) = std::fs::remove_file(&test_file_path) {
                println!("⚠️ Warning: Could not remove test file: {e}");
            }
            Ok(())
        }
        Err(e) => {
            println!("❌ Directory is not writable: {e}");
            Err(anyhow::anyhow!("Directory is not writable: {}", e))
        }
    }
}

/// Checks if a file exists and is writable
pub fn check_file_permissions(file_path: &Path) -> Result<()> {
    println!("Checking file permissions for: {file_path:?}");

    if !file_path.exists() {
        println!("❌ File doesn't exist: {file_path:?}");
        return Err(anyhow::anyhow!("File doesn't exist: {:?}", file_path));
    }

    if !file_path.is_file() {
        println!("❌ Path is not a file: {file_path:?}");
        return Err(anyhow::anyhow!("Path is not a file: {:?}", file_path));
    }

    // Check if the file is writable
    match std::fs::OpenOptions::new().write(true).open(file_path) {
        Ok(_) => {
            println!("✅ File is writable: {file_path:?}");
            Ok(())
        }
        Err(e) => {
            println!("❌ File is not writable: {e}");
            Err(anyhow::anyhow!("File is not writable: {}", e))
        }
    }
}

/// Sets secure permissions for a database file and its parent directory
///
/// This function sets the database file permissions to 0600 (read/write for owner only)
/// and the parent directory permissions to 0700 (read/write/execute for owner only).
///
/// # Arguments
///
/// * `db_path` - The path to the database file
///
/// # Returns
///
/// A Result containing () if the permissions were set successfully, or an error if they weren't
pub fn secure_db_permissions(db_path: &Path) -> Result<()> {
    info!("Setting secure permissions for database: {db_path:?}");

    // Create parent directory if it doesn't exist
    if let Some(parent_dir) = db_path.parent() {
        if !parent_dir.exists() {
            debug!("Creating parent directory: {parent_dir:?}");
            fs::create_dir_all(parent_dir)?;
        }

        // Set secure permissions on parent directory (0700 - rwx------)
        debug!("Setting secure permissions on parent directory: {parent_dir:?}");
        let dir_perms = Permissions::from_mode(0o700);
        match fs::set_permissions(parent_dir, dir_perms) {
            Ok(_) => info!("✅ Set directory permissions to 0700 (owner read/write/execute only)"),
            Err(e) => {
                warn!("⚠️ Could not set directory permissions: {e}");
                // Continue even if we couldn't set directory permissions
            }
        }
    }

    // If the database file exists, set secure permissions on it
    if db_path.exists() {
        debug!("Setting secure permissions on database file: {db_path:?}");
        let file_perms = Permissions::from_mode(0o600);
        match fs::set_permissions(db_path, file_perms) {
            Ok(_) => info!("✅ Set file permissions to 0600 (owner read/write only)"),
            Err(e) => {
                error!("❌ Could not set file permissions: {e}");
                return Err(anyhow::anyhow!("Could not set file permissions: {}", e));
            }
        }
    } else {
        debug!("Database file doesn't exist yet, will be created by SQLite");
    }

    Ok(())
}
