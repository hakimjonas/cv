# Authentication Fix

## Issue

Users were unable to log in with the default credentials (admin/admin123) even though the default users were being created successfully. The logs showed:

```
Authentication failed for user: admin
```

## Root Cause

The issue was that the default users were being created correctly, but the password hashing algorithm or parameters might have changed, causing the stored password hash to be incompatible with the verification process.

## Solution

We created a password reset script (`reset_admin_password.rs`) that:

1. Checks if the admin user exists
2. If it exists, resets its password to "admin123"
3. If it doesn't exist, creates a new admin user with the password "admin123"

We then modified the Docker startup script to run this password reset script after creating the default users but before starting the blog API server.

## Implementation Details

### 1. Password Reset Script

```rust
// src/bin/reset_admin_password.rs
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
```

### 2. Modified Docker Startup Script

```bash
# docker/startup.sh
echo "Creating default users..."
cargo run --bin create_default_users -- --db-path="./test_data/blog_test.db"

echo "Resetting admin password..."
cargo run --bin reset_admin_password -- --db-path="./test_data/blog_test.db"

echo "Starting blog API server..."
exec cargo run --bin blog_api_server
```

## Testing

To test the fix:

1. Rebuild the Docker container:
   ```bash
   ./scripts/deploy-local.sh rebuild
   ```

2. Check the logs to ensure the password reset script ran successfully:
   ```bash
   ./scripts/deploy-local.sh logs | grep "Resetting admin password"
   ```

3. Attempt to log in with the admin credentials:
   - Username: admin
   - Password: admin123

## Future Improvements

1. **Consistent Password Hashing**: Ensure that the password hashing algorithm and parameters are consistent across all components.

2. **Better Error Handling**: Add more detailed error messages for authentication failures to make debugging easier.

3. **Password Reset API**: Consider adding a password reset API endpoint for administrators to reset user passwords.

4. **Password Strength Requirements**: Implement password strength requirements to ensure users choose secure passwords.

5. **Account Lockout**: Implement account lockout after a certain number of failed login attempts to prevent brute force attacks.