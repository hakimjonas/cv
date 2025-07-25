use cv::db::{Database, repository::UserRole};
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

    println!("Creating default user accounts...");

    // Check if admin user already exists
    if let Ok(Some(_)) = user_repo.get_user_by_username("admin").await {
        println!("Admin user already exists, skipping creation");
    } else {
        // Create admin user
        match user_repo
            .create_user(
                "admin",
                "Administrator",
                "admin@example.com",
                "admin123",
                UserRole::Admin,
            )
            .await
        {
            Ok(id) => println!("Created admin user with ID: {id}"),
            Err(e) => println!("Error creating admin user: {e}"),
        }
    }

    // Check if author user already exists
    if let Ok(Some(_)) = user_repo.get_user_by_username("author").await {
        println!("Author user already exists, skipping creation");
    } else {
        // Create author user
        match user_repo
            .create_user(
                "author",
                "Author User",
                "author@example.com",
                "author123",
                UserRole::Author,
            )
            .await
        {
            Ok(id) => println!("Created author user with ID: {id}"),
            Err(e) => println!("Error creating author user: {e}"),
        }
    }

    // Check if editor user already exists
    if let Ok(Some(_)) = user_repo.get_user_by_username("editor").await {
        println!("Editor user already exists, skipping creation");
    } else {
        // Create editor user
        match user_repo
            .create_user(
                "editor",
                "Editor User",
                "editor@example.com",
                "editor123",
                UserRole::Editor,
            )
            .await
        {
            Ok(id) => println!("Created editor user with ID: {id}"),
            Err(e) => println!("Error creating editor user: {e}"),
        }
    }

    println!("Default user accounts created successfully!");
    println!("You can now log in with the following credentials:");
    println!("Admin: username=admin, password=admin123");
    println!("Author: username=author, password=author123");
    println!("Editor: username=editor, password=editor123");

    Ok(())
}
