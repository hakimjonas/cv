# Docker Deployment Fixes

This document outlines the fixes applied to resolve issues with the local Docker deployment.

## Issues Fixed

1. **CV Data Structure Mismatch**: The sample CV data in the startup script didn't match the expected structure in the code.
2. **Database Path Mismatch**: The default users script was using a different database path than the one used in the container.
3. **PDF Generation Failure**: The CV generator was failing because the Typst CLI wasn't installed in the container.

## Changes Made

### 1. Fixed CV Data Structure

The startup script now creates a properly formatted `cv_data.json` file that matches the expected structure in `cv_data.rs`:

```bash
# Always create a fresh cv_data.json file with the correct structure
echo "Creating CV data file with the correct structure..."
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
```

### 2. Fixed Database Path

Modified `create_default_users.rs` to accept a command-line argument for the database path:

```rust
// In src/bin/create_default_users.rs
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments for database path
    let args: Vec<String> = env::args().collect();
    let mut db_path = "data/blog.db".to_string();
    
    // Look for --db-path argument
    for i in 0..args.len() {
        if args[i] == "--db-path" && i + 1 < args.len() {
            db_path = args[i + 1].clone();
            println!("Using database path: {}", db_path);
            break;
        }
    }
    
    // Initialize the database
    let db = Database::new(&db_path)?;
    // Rest of the function...
}
```

### 3. Handled PDF Generation Failure

Modified the startup script to continue even if the CV generator fails due to missing Typst CLI:

```bash
# Run CV generator but don't fail if PDF generation fails (due to missing Typst CLI)
cargo run --bin cv || {
    echo "Warning: CV generator failed, but continuing with deployment"
    echo "This is likely due to missing Typst CLI for PDF generation"
    echo "The website should still be functional without the PDF"
}
```

## Future Improvements

For a complete solution, consider the following improvements:

1. **Install Typst CLI**: Add the Typst CLI to the Docker image to enable PDF generation. This requires resolving network connectivity issues during the Docker build.

   ```dockerfile
   # Install Typst CLI for PDF generation
   RUN wget -q https://github.com/typst/typst/releases/download/v0.10.0/typst-x86_64-unknown-linux-musl.tar.xz \
       && tar -xf typst-x86_64-unknown-linux-musl.tar.xz \
       && mv typst-x86_64-unknown-linux-musl/typst /usr/local/bin/ \
       && rm -rf typst-x86_64-unknown-linux-musl.tar.xz typst-x86_64-unknown-linux-musl
   ```

2. **Create Sample Language Icons**: Add a sample `language_icons.json` file to avoid the warning about missing language icons.

3. **Improve Error Handling**: Add more robust error handling in the startup script to provide clearer error messages.

## Deployment Instructions

To deploy the application locally:

1. Ensure Docker and Docker Compose are installed
2. Run the local deployment script:

   ```bash
   ./scripts/deploy-local.sh start
   ```

3. Access the application at:
   - Main Website: http://localhost:3002
   - Blog: http://localhost:3002/blog.html
   - CV: http://localhost:3002/cv.html
   - Projects: http://localhost:3002/projects.html
   - Blog API: http://localhost:3002/api/blog
   - API Admin: http://localhost:3002/admin
   - Blog Client: http://localhost:3002/static/blog-client.html
   - Debug Tool: http://localhost:3002/static/blog-debug.html

4. To stop the application:

   ```bash
   ./scripts/deploy-local.sh stop
   ```

5. To restart the application:

   ```bash
   ./scripts/deploy-local.sh restart
   ```

## Troubleshooting

If you encounter issues:

1. Check the logs:

   ```bash
   ./scripts/deploy-local.sh logs
   ```

2. Check the container status:

   ```bash
   ./scripts/deploy-local.sh status
   ```

3. Rebuild the application:

   ```bash
   ./scripts/deploy-local.sh rebuild
   ```

4. If the container is unhealthy, it might be due to:
   - Missing or incorrectly formatted CV data
   - Database path issues
   - PDF generation failures