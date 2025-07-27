#!/bin/bash
set -e

# Ensure data directory exists
mkdir -p data
mkdir -p test_data

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

echo "Generating website files..."
# Run CV generator but don't fail if PDF generation fails (due to missing Typst CLI)
cargo run --bin cv || {
    echo "Warning: CV generator failed, but continuing with deployment"
    echo "This is likely due to missing Typst CLI for PDF generation"
    echo "The website should still be functional without the PDF"
}

echo "Starting blog API server..."
# Start the blog API server in the background
cargo run --bin blog_api_server &
SERVER_PID=$!

# Wait a moment for the server to start
echo "Waiting for server to start..."
sleep 10

echo "Creating default users..."
cargo run --bin create_default_users -- --db-path="./test_data/blog_test.db"

echo "Resetting admin password directly in the database..."
# Wait a bit more to ensure database is fully initialized with tables
sleep 5
# Use the direct reset script to reset the admin password
echo "Running reset_admin_password_direct script..."
cargo run --bin reset_admin_password_direct -- --db-path="./test_data/blog_test.db"
echo "Admin password reset complete!"

# Wait for the server process to complete
echo "Server is running. Press Ctrl+C to stop."
wait $SERVER_PID
