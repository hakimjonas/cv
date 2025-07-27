#!/bin/bash
# Test script to verify the deployment

# Set colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to print success message
success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error message
error() {
    echo -e "${RED}✗ $1${NC}"
}

# Function to print warning message
warning() {
    echo -e "${YELLOW}! $1${NC}"
}

# Function to print section header
section() {
    echo -e "\n${YELLOW}=== $1 ===${NC}"
}

# Function to check if a URL returns a 200 status code
check_url() {
    local url=$1
    local expected_status=${2:-200}
    local status=$(curl -s -o /dev/null -w "%{http_code}" $url)
    
    if [ "$status" -eq "$expected_status" ]; then
        success "URL $url returned status $status (expected $expected_status)"
        return 0
    else
        error "URL $url returned status $status (expected $expected_status)"
        return 1
    fi
}

# Function to check if a file exists
check_file() {
    local file=$1
    
    if [ -f "$file" ]; then
        success "File $file exists"
        return 0
    else
        error "File $file does not exist"
        return 1
    fi
}

# Function to check if a directory exists
check_dir() {
    local dir=$1
    
    if [ -d "$dir" ]; then
        success "Directory $dir exists"
        return 0
    else
        error "Directory $dir does not exist"
        return 1
    fi
}

# Function to check if a string is in a file
check_string_in_file() {
    local file=$1
    local string=$2
    
    if grep -q "$string" "$file"; then
        success "String '$string' found in $file"
        return 0
    else
        error "String '$string' not found in $file"
        return 1
    fi
}

# Function to test authentication
test_auth() {
    section "Testing Authentication"
    
    # Test login with admin credentials
    local login_response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"username":"admin","password":"admin123"}' http://localhost:3002/api/auth/login)
    
    if echo "$login_response" | grep -q "token"; then
        success "Login successful with admin credentials"
        local token=$(echo "$login_response" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
        echo "Token: $token"
        
        # Test authenticated API call
        local auth_response=$(curl -s -H "Authorization: Bearer $token" http://localhost:3002/api/blog)
        if [ $? -eq 0 ]; then
            success "Authenticated API call successful"
        else
            error "Authenticated API call failed"
        fi
    else
        error "Login failed with admin credentials: $login_response"
    fi
}

# Function to test CV data
test_cv_data() {
    section "Testing CV Data"
    
    # Check if CV data file exists
    check_file "data/cv_data.json"
    
    # Check if CV data backup file exists
    check_file "data/cv_data.json.bak"
    
    # Check if CV HTML file exists
    check_file "dist/cv.html"
    
    # Check if CV PDF file exists
    check_file "dist/cv.pdf"
    
    # Check if the CV HTML contains the correct name
    if [ -f "dist/cv.html" ]; then
        local name=$(grep -o '<h1>[^<]*</h1>' dist/cv.html | head -1 | sed 's/<h1>\(.*\)<\/h1>/\1/')
        echo "Name in CV HTML: $name"
        
        # Check if the name in the HTML matches the name in the JSON
        local json_name=$(grep -o '"name":"[^"]*"' data/cv_data.json | head -1 | cut -d'"' -f4)
        echo "Name in CV JSON: $json_name"
        
        if [ "$name" = "$json_name" ]; then
            success "Name in CV HTML matches name in CV JSON"
        else
            error "Name in CV HTML ($name) does not match name in CV JSON ($json_name)"
        fi
    fi
}

# Function to test blog functionality
test_blog() {
    section "Testing Blog Functionality"
    
    # Check if blog API is accessible
    check_url "http://localhost:3002/api/blog"
    
    # Check if blog client is accessible
    check_url "http://localhost:3002/static/blog-client.html"
    
    # Check if blog debug tool is accessible
    check_url "http://localhost:3002/static/blog-debug.html"
    
    # Check if admin page is accessible
    check_url "http://localhost:3002/admin"
}

# Function to test database persistence
test_database() {
    section "Testing Database Persistence"
    
    # Check if test_data directory exists
    check_dir "test_data"
    
    # Check if database file exists
    check_file "test_data/blog_test.db"
    
    # Check if database file is writable
    if [ -f "test_data/blog_test.db" ]; then
        if [ -w "test_data/blog_test.db" ]; then
            success "Database file is writable"
        else
            error "Database file is not writable"
        fi
    fi
}

# Main function
main() {
    section "Starting Deployment Tests"
    
    # Check if the deployment is running
    if ! check_url "http://localhost:3002"; then
        error "Deployment is not running. Please start it with './scripts/deploy-local.sh start'"
        exit 1
    fi
    
    # Run tests
    test_database
    test_auth
    test_cv_data
    test_blog
    
    section "Tests Completed"
}

# Run the main function
main