use cv::auth::AuthService;
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
    
    // Create the auth service with the same JWT secret as in blog_api.rs
    let jwt_secret = "secure_jwt_secret_for_development_only".to_string();
    let token_expiration = 86400; // 24 hours
    let auth_service = AuthService::new(&db, jwt_secret, token_expiration);

    println!("Testing authentication service...");

    // Test login with admin credentials
    let username = "admin";
    let password = "admin123";
    
    println!("Attempting to log in with username: {username}, password: {password}");
    
    match auth_service.login(username, password).await {
        Ok(response) => {
            println!("Login successful!");
            println!("User ID: {}", response.user_id);
            println!("Username: {}", response.username);
            println!("Display name: {}", response.display_name);
            println!("Role: {}", response.role);
            println!("Token: {}", response.token);
        },
        Err(e) => {
            println!("Login failed: {}", e);
            
            // Get the user to check if it exists
            let user_repo = db.user_repository();
            match user_repo.get_user_by_username(username).await? {
                Some(user) => {
                    println!("User exists in the database:");
                    println!("  ID: {:?}", user.id);
                    println!("  Username: {}", user.username);
                    println!("  Display name: {}", user.display_name);
                    println!("  Password hash: {}", user.password_hash);
                    
                    // Test password verification directly
                    let is_valid = user_repo.verify_password(&user, password).await?;
                    println!("Direct password verification: {}", if is_valid { "SUCCESS" } else { "FAILED" });
                    
                    // Test authentication directly
                    let auth_result = user_repo.authenticate(username, password).await?;
                    println!("Direct authentication: {}", if auth_result.is_some() { "SUCCESS" } else { "FAILED" });
                },
                None => {
                    println!("User does not exist in the database!");
                }
            }
        }
    }

    Ok(())
}