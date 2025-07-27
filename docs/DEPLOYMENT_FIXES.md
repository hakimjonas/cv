# Deployment Fixes

This document outlines the fixes applied to resolve issues with the local Docker deployment.

## Issues Fixed

1. **Authentication Failures**: Users were unable to log in with default credentials due to password hashing algorithm mismatches.
2. **CV Data Inconsistencies**: The CV data was not being properly preserved between container restarts.
3. **Database Persistence**: The database was not being properly persisted between container restarts.
4. **Admin Page Errors**: The admin page was showing "Error: testApiConnection is not defined" when clicking the Test API Connection button.
5. **Blog Debug Tool Errors**: The blog debug tool was showing "ReferenceError: response is not defined" when clicking the Create Post button.

## Changes Made

### 1. Fixed Authentication System

The authentication system was using Argon2 for password hashing and verification, but the SQL script for resetting the admin password was using bcrypt. This mismatch caused authentication failures.

**Solution**:
- Ensured that the startup.sh script uses the reset_admin_password_direct.rs script, which uses Argon2 for password hashing.
- Removed references to the SQL script that used bcrypt.

### 2. Fixed CV Data Persistence

The CV data was not being properly preserved between container restarts because the startup.sh script was creating a sample CV data file if one didn't exist, potentially overwriting the user's actual data.

**Solution**:
- Modified the startup.sh script to check for a backup file and restore from it if available.
- Added code to create a backup of the existing CV data file.
- Ensured that the data directory is properly mounted from the host.

```bash
# Check if cv_data.json exists, if not, check for a backup or create a sample one
if [ ! -f "data/cv_data.json" ]; then
    # Check if there's a backup file in the mounted volume
    if [ -f "/app/data/cv_data.json.bak" ]; then
        echo "CV data file not found, but backup exists. Restoring from backup..."
        cp /app/data/cv_data.json.bak /app/data/cv_data.json
    else
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
    fi
else
    echo "Using existing CV data file"
    # Create a backup of the existing file
    cp data/cv_data.json data/cv_data.json.bak
fi
```

### 3. Fixed Database Persistence

The database was not being properly persisted between container restarts because the test_data directory was not being mounted from the host.

**Solution**:
- Modified docker-compose.local.yml to mount the test_data directory from the host:

```yaml
volumes:
  # Mount data directory from host instead of using a volume
  - ../data:/app/data
  # Mount test_data directory for database persistence
  - ../test_data:/app/test_data
  # Mount source code for hot reloading
  - ../src:/app/src
```

- Modified the blog_utils.rs file to not remove the database if it's locked:

```rust
if e.to_string().contains("locked") {
    println!(
        "⚠️ Existing database appears to be locked, but we'll try to use it anyway"
    );
    // Don't remove the database, just wait a moment for it to be released
    std::thread::sleep(std::time::Duration::from_millis(500));
}
```

### 4. Fixed Admin Page Errors

The admin page was showing "Error: testApiConnection is not defined" when clicking the Test API Connection button.

**Solution**:
- Created a proper admin page at /static/admin/index.html that correctly uses the blog-debug.js functions.
- Modified blog_api.rs to serve the static admin/index.html file instead of hardcoded HTML.
- Added debugging to the admin page to check if the script is being loaded correctly:

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
<script src="/static/js/blog-debug.js" onerror="console.error('Failed to load blog-debug.js');"></script>
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

### 5. Fixed Blog Debug Tool Errors

The blog debug tool was showing "ReferenceError: response is not defined" when clicking the Create Post button.

**Solution**:
- Removed undefined response variable references from blog-debug.html:

```html
<!-- Remove or comment out these lines -->
<!--
console.log('Response status:', response.status);
console.log('Response headers:', Object.fromEntries([...response.headers]));
console.log('Response body:', responseJson || responseText);
-->
```

- Added proper error handling in the API client.

## Deployment Instructions

To deploy the application with these fixes:

1. **Ensure your CV data is in place**:
   ```bash
   # Copy your CV data to the data directory
   cp your_cv_data.json data/cv_data.json
   ```

2. **Restart the local deployment**:
   ```bash
   ./scripts/deploy-local.sh restart
   ```

3. **Test the deployment**:
   ```bash
   # Make the test script executable
   chmod +x scripts/test-deployment.sh
   
   # Run the test script
   ./scripts/test-deployment.sh
   ```

4. **Access the application**:
   - Main Website: http://localhost:3002
   - Admin Page: http://localhost:3002/admin
   - Blog Client: http://localhost:3002/static/blog-client.html
   - Debug Tool: http://localhost:3002/static/blog-debug.html

5. **Log in with default credentials**:
   - Username: admin
   - Password: admin123

## Troubleshooting

If you encounter issues:

1. **Check the logs**:
   ```bash
   ./scripts/deploy-local.sh logs
   ```

2. **Check the container status**:
   ```bash
   ./scripts/deploy-local.sh status
   ```

3. **Rebuild the application**:
   ```bash
   ./scripts/deploy-local.sh rebuild
   ```

4. **Check the database**:
   ```bash
   # Connect to the database
   sqlite3 test_data/blog_test.db
   
   # Check if the admin user exists
   SELECT * FROM users WHERE username = 'admin';
   
   # Exit SQLite
   .exit
   ```

5. **Reset the admin password manually**:
   ```bash
   # Run the reset_admin_password_direct script
   cargo run --bin reset_admin_password_direct -- --db-path="./test_data/blog_test.db"
   ```

## Future Improvements

For a more robust solution, consider the following improvements:

1. **Use environment variables** for database paths to ensure consistency across all components.
2. **Add more comprehensive error handling** in the JavaScript code.
3. **Implement a more robust database connection pool** that can handle locked databases.
4. **Add automated tests** for the deployment process to catch these issues earlier.
5. **Install Typst CLI** in the Docker image to enable PDF generation.
6. **Implement a more robust backup system** for CV data and database files.
7. **Add a health check endpoint** that verifies all components are working correctly.
8. **Implement a proper CI/CD pipeline** for testing and deployment.