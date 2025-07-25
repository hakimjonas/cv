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

    println!("Resetting admin user password...");

    // Get the admin user by username
    match user_repo.get_user_by_username("admin").await? {
        Some(user) => {
            // Admin user exists, reset the password
            if let Some(user_id) = user.id {
                match user_repo.change_password(user_id, "admin123").await {
                    Ok(_) => println!("Successfully reset password for admin user (ID: {user_id})"),
                    Err(e) => println!("Error resetting admin password: {e}"),
                }
            } else {
                println!("Error: Admin user has no ID");
            }
        },
        None => {
            println!("Admin user not found, creating it...");
            // Create admin user
            match user_repo
                .create_user(
                    "admin",
                    "Administrator",
                    "admin@example.com",
                    "admin123",
                    cv::db::repository::UserRole::Admin,
                )
                .await
            {
                Ok(id) => println!("Created admin user with ID: {id}"),
                Err(e) => println!("Error creating admin user: {e}"),
            }
        }
    }

    println!("Password reset complete!");
    println!("You can now log in with the following credentials:");
    println!("Username: admin");
    println!("Password: admin123");

    Ok(())
}