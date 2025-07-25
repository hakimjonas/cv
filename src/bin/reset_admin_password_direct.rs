use argon2::{
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    Argon2,
};
use std::env;
use std::error::Error;
use std::path::Path;
use rusqlite::Connection;
use anyhow::{Result, anyhow};

fn main() -> Result<()> {
    // Parse command-line arguments for database path
    let args: Vec<String> = env::args().collect();
    let mut db_path = "data/blog.db".to_string();
    
    // Look for --db-path argument
    for i in 0..args.len() {
        if args[i] == "--db-path" && i + 1 < args.len() {
            db_path = args[i + 1].clone();
            println!("Using database path: {db_path}");
            break;
        }
    }
    
    // Check if the database file exists
    if !Path::new(&db_path).exists() {
        println!("Database file not found at: {db_path}");
        println!("Waiting for database to be created...");
        
        // Wait for the database to be created (max 30 seconds)
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if Path::new(&db_path).exists() {
                println!("Database file found!");
                break;
            }
        }
        
        if !Path::new(&db_path).exists() {
            return Err(anyhow!("Database file not found after waiting: {}", db_path));
        }
    }
    
    // Wait a bit more to ensure migrations are complete
    println!("Waiting for database migrations to complete...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // Generate a new password hash
    let password = "admin123";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Password hashing error: {}", e))?
        .to_string();
    
    println!("Generated password hash for 'admin123'");
    
    // Connect to the database
    let conn = Connection::open(&db_path)?;
    
    // Check if admin user exists
    let mut stmt = conn.prepare("SELECT id FROM users WHERE username = 'admin'")?;
    let admin_id: Option<i64> = stmt.query_row([], |row| row.get(0)).ok();
    
    if let Some(user_id) = admin_id {
        // Update the admin user's password
        conn.execute(
            "UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![
                password_hash,
                chrono::Local::now().to_rfc3339(),
                user_id
            ],
        )?;
        
        println!("Successfully reset password for admin user (ID: {user_id})");
    } else {
        // Create admin user if it doesn't exist
        conn.execute(
            "INSERT INTO users (username, display_name, email, password_hash, role, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "admin",
                "Administrator",
                "admin@example.com",
                password_hash,
                "Admin",
                chrono::Local::now().to_rfc3339(),
                chrono::Local::now().to_rfc3339()
            ],
        )?;
        
        let user_id = conn.last_insert_rowid();
        println!("Created admin user with ID: {user_id}");
    }
    
    println!("Password reset complete!");
    println!("You can now log in with the following credentials:");
    println!("Username: admin");
    println!("Password: admin123");
    
    Ok(())
}