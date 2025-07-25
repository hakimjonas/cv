#!/bin/bash
set -e

# Ensure data directory exists
mkdir -p data
mkdir -p test_data

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

echo "Generating website files..."
# Run CV generator but don't fail if PDF generation fails (due to missing Typst CLI)
cargo run --bin cv || {
    echo "Warning: CV generator failed, but continuing with deployment"
    echo "This is likely due to missing Typst CLI for PDF generation"
    echo "The website should still be functional without the PDF"
}

echo "Creating default users..."
cargo run --bin create_default_users -- --db-path="./test_data/blog_test.db"

echo "Starting blog API server..."
exec cargo run --bin blog_api_server