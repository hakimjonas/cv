use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info};

use crate::cv_data::Cv;
use crate::db::Database;

/// Migrates CV data from JSON to SQLite (synchronous version)
///
/// This function is a synchronous wrapper around the async version.
/// For new code, prefer using `migrate_json_to_sqlite_async` instead.
///
/// # Arguments
///
/// * `json_path` - Path to the JSON file
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
///
/// A Result indicating success or failure
pub fn migrate_json_to_sqlite<P: AsRef<Path>, Q: AsRef<Path>>(
    json_path: P,
    db_path: Q,
) -> Result<()> {
    // Load CV data from JSON
    info!(
        "Loading CV data from JSON: {}",
        json_path.as_ref().display()
    );
    let cv = Cv::from_json(json_path.as_ref().to_str().unwrap())?;
    debug!(
        "Successfully loaded CV data with {} projects",
        cv.projects.len()
    );

    // Create database and schema
    info!("Creating database: {}", db_path.as_ref().display());
    let db = Database::new(db_path)?;
    db.create_schema()?;
    debug!("Database schema created successfully");

    // Insert CV data into database
    info!("Inserting CV data into database");
    db.insert_cv(&cv)?;
    debug!("CV data inserted successfully");

    info!("Migration completed successfully");
    Ok(())
}

/// Migrates CV data from JSON to SQLite asynchronously
///
/// # Arguments
///
/// * `json_path` - Path to the JSON file
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
///
/// A Result indicating success or failure
pub async fn migrate_json_to_sqlite_async<P: AsRef<Path>, Q: AsRef<Path>>(
    json_path: P,
    db_path: Q,
) -> Result<()> {
    // Load CV data from JSON
    info!(
        "Loading CV data from JSON asynchronously: {}",
        json_path.as_ref().display()
    );
    let cv = Cv::from_json(json_path.as_ref().to_str().unwrap())
        .context("Failed to load CV data from JSON")?;
    debug!(
        "Successfully loaded CV data with {} projects",
        cv.projects.len()
    );

    // Create database and schema asynchronously
    info!(
        "Creating database asynchronously: {}",
        db_path.as_ref().display()
    );
    let db = Database::new(&db_path).context("Failed to create database connection")?;
    db.create_schema_async()
        .await
        .context("Failed to create database schema")?;
    debug!("Database schema created successfully");

    // Insert CV data into database asynchronously
    info!("Inserting CV data into database asynchronously");
    db.insert_cv_async(&cv)
        .await
        .context("Failed to insert CV data into database")?;
    debug!("CV data inserted successfully");

    info!("Migration completed successfully");
    Ok(())
}

/// Loads CV data from SQLite (synchronous version)
///
/// This function is a synchronous wrapper around the async version.
/// For new code, prefer using `load_cv_from_sqlite_async` instead.
///
/// # Arguments
///
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
///
/// A Result containing the CV data or an error
pub fn load_cv_from_sqlite<P: AsRef<Path>>(db_path: P) -> Result<Cv> {
    // Open database
    info!(
        "Loading CV data from database: {}",
        db_path.as_ref().display()
    );
    let db = Database::new(db_path)?;
    debug!("Database connection established");

    // Load CV data
    let cv = db.load_cv()?;
    debug!("CV data loaded with {} projects", cv.projects.len());

    info!("CV data loaded successfully");
    Ok(cv)
}

/// Loads CV data from SQLite asynchronously
///
/// # Arguments
///
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
///
/// A Result containing the CV data or an error
pub async fn load_cv_from_sqlite_async<P: AsRef<Path>>(db_path: P) -> Result<Cv> {
    // Open database
    info!(
        "Loading CV data from database asynchronously: {}",
        db_path.as_ref().display()
    );
    let db = Database::new(&db_path).context("Failed to create database connection")?;
    debug!("Database connection established");

    // Load CV data asynchronously
    let cv = db
        .load_cv_async()
        .await
        .context("Failed to load CV data from database")?;
    debug!("CV data loaded with {} projects", cv.projects.len());

    info!("CV data loaded successfully");
    Ok(cv)
}
