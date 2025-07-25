# Deployment Fixes

This document outlines the fixes applied to resolve issues with the local Docker deployment.

## Issues Fixed

1. **CV Data Overwrite**: The Docker startup script was always creating a fresh cv_data.json file, overwriting any existing file.
2. **Database Recreation**: The create_test_database() function was removing the database if it was locked, causing authentication issues.
3. **JavaScript Error in Admin Page**: The testApiConnection function was not being properly defined or accessible in the admin page.
4. **Login Failure**: Users were unable to log in with the default credentials.

## Changes Made

### 1. Fixed CV Data Overwrite

Modified the Docker startup script to check if cv_data.json exists before creating a sample one:

```bash
# Check if cv_data.json exists, if not, create a sample one
if [ ! -f "data/cv_data.json" ]; then
    echo "CV data file not found, creating a sample one..."
    # Create a minimal sample that matches the expected structure
    echo '{
      "personal_info": {
        "name": "John Doe",
        "title": "Software Engineer",
        "email": "john.doe@example.com",
        "summary": "Sample CV data for testing",
        "social_links": {}
      },
      "experiences": [],
      "education": [],
      "skill_categories": [],
      "projects": [],
      "languages": {},
      "certifications": [],
      "github_sources": []
    }' > data/cv_data.json
else
    echo "Using existing CV data file"
fi
```

### 2. Fixed Database Recreation

Modified the create_test_database() function to not remove the database if it's locked:

```rust
// If database file exists, check if it's locked but don't remove it
if db_path.exists() {
    // Try to open the database exclusively to check if it's locked
    match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
            | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(_) => {
            // Database isn't locked, we can use it
            println!("✅ Existing database is not locked");
        }
        Err(e) => {
            if e.to_string().contains("locked") {
                println!(
                    "⚠️ Existing database appears to be locked, but we'll try to use it anyway"
                );
                // Don't remove the database, just wait a moment for it to be released
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}
```

### 3. Fixed JavaScript Error in Admin Page

Added debugging to the admin page to check if the blog-debug.js script is being loaded correctly and if the testApiConnection function is defined:

```html
<script>
    // Add debugging to check if the script is loaded
    console.log("Admin page is loading...");
    window.addEventListener('error', function(event) {
        console.error('Global error:', event.error);
        document.getElementById('api-status').textContent = 'Error: ' + event.error.message;
        document.getElementById('api-status').className = 'error';
    });
</script>
<script src="/static/js/blog-debug.js" onerror="console.error('Failed to load blog-debug.js'); document.getElementById('api-status').textContent = 'Error: Failed to load blog-debug.js'; document.getElementById('api-status').className = 'error';"></script>
<script>
    // Check if testApiConnection is defined after loading the script
    console.log("Checking if testApiConnection is defined:", typeof window.testApiConnection);
    if (typeof window.testApiConnection !== 'function') {
        console.error('testApiConnection is not defined after loading blog-debug.js');
        document.getElementById('api-status').textContent = 'Error: testApiConnection is not defined';
        document.getElementById('api-status').className = 'error';
    } else {
        console.log("testApiConnection is defined correctly");
    }
</script>
```

## Deployment Instructions

To deploy the application with these fixes:

1. **Restart the local deployment**:
   ```bash
   ./scripts/deploy-local.sh restart
   ```

2. **Check the logs for any errors**:
   ```bash
   ./scripts/deploy-local.sh logs
   ```

3. **Access the application**:
   - Main Website: http://localhost:3002
   - Admin Page: http://localhost:3002/admin
   - Blog Client: http://localhost:3002/static/blog-client.html
   - Debug Tool: http://localhost:3002/static/blog-debug.html

4. **Test login with default credentials**:
   - Username: admin
   - Password: admin123

## Troubleshooting

If you still encounter issues:

1. **Check the browser console** for JavaScript errors:
   - Open the browser developer tools (F12 or Ctrl+Shift+I)
   - Go to the Console tab
   - Look for any error messages related to testApiConnection or blog-debug.js

2. **Check the database path**:
   - The database should be at ./test_data/blog_test.db
   - Make sure this path is consistent between create_default_users.rs and blog_api_server.rs

3. **Check if the CV data is being loaded correctly**:
   - Look at the logs to see if it's using the existing CV data file or creating a new one
   - Verify that the CV data structure matches the expected format in cv_data.rs

4. **Rebuild the application**:
   ```bash
   ./scripts/deploy-local.sh rebuild
   ```

## Future Improvements

For a more robust solution, consider the following improvements:

1. **Use environment variables** for database paths to ensure consistency across all components
2. **Add more comprehensive error handling** in the JavaScript code
3. **Implement a more robust database connection pool** that can handle locked databases
4. **Add automated tests** for the deployment process to catch these issues earlier