use cv::db::Database;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    
    // Initialize the database
    let db = Database::new(&db_path)?;
    let user_repo = db.user_repository();

    println!("Checking admin user password...");

    // Get the admin user by username
    match user_repo.get_user_by_username("admin").await? {
        Some(user) => {
            println!("Admin user found with ID: {:?}", user.id);
            println!("Username: {}", user.username);
            println!("Display name: {}", user.display_name);
            println!("Email: {}", user.email);
            println!("Password hash: {}", user.password_hash);
            println!("Role: {:?}", user.role);
            
            // Test password verification
            let test_password = "admin123";
            let is_valid = user_repo.verify_password(&user, test_password).await?;
            println!("Password verification for '{}': {}", test_password, if is_valid { "SUCCESS" } else { "FAILED" });
        },
        None => {
            println!("Admin user not found!");
        }
    }

    Ok(())
}