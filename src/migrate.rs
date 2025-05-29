use anyhow::Result;
use std::path::Path;

use crate::cv_data::Cv;
use crate::db::Database;

/// Migrates CV data from JSON to SQLite
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
    println!(
        "Loading CV data from JSON: {}",
        json_path.as_ref().display()
    );
    let cv = Cv::from_json(json_path.as_ref().to_str().unwrap())?;

    // Create database and schema
    println!("Creating database: {}", db_path.as_ref().display());
    let mut db = Database::new(db_path)?;
    db.create_schema()?;

    // Insert CV data into database
    println!("Inserting CV data into database");
    db.insert_cv(&cv)?;

    println!("Migration completed successfully");
    Ok(())
}

/// Loads CV data from SQLite
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
    println!(
        "Loading CV data from database: {}",
        db_path.as_ref().display()
    );
    let db = Database::new(db_path)?;

    // Load CV data
    let cv = db.load_cv()?;

    println!("CV data loaded successfully");
    Ok(cv)
}
