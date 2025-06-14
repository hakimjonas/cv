#!/bin/bash

# Make this script executable
chmod +x "$0"

echo "Blog API Server Test"
echo "------------------"

# Detect the blog API server port
detect_port() {
    for port in $(seq 3000 3010); do
        if curl -s http://localhost:$port >/dev/null; then
            echo $port
            return 0
        fi
    done
    echo "Could not detect blog API server port. Make sure it's running."
    exit 1
}

# Set port (use detected port if not specified)
PORT=${1:-$(detect_port)}
BASE_URL="http://localhost:$PORT"

echo "Using API at: $BASE_URL"
echo 

echo "1. Testing connection to root endpoint"
curl -i "$BASE_URL/"
echo 

echo "2. Testing GET all posts"
curl -i "$BASE_URL/api/blog"
echo 

echo "3. Testing CORS with OPTIONS request"
curl -i -X OPTIONS \
  -H "Origin: http://localhost:8000" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type" \
  "$BASE_URL/api/blog"
echo 

echo "4. Testing POST a new post"
curl -i -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "title": "API Test Post",
    "slug": "api-test-post",
    "date": "2025-06-12",
    "author": "API Tester",
    "excerpt": "This is a test excerpt",
    "content": "This is the full content of the test post.",
    "published": true,
    "featured": false,
    "image": null,
    "tags": [],
    "metadata": {}
  }' \
  "$BASE_URL/api/blog"
echo 

echo "5. Testing GET specific post"
curl -i "$BASE_URL/api/blog/api-test-post"
echo 

echo "6. Testing DELETE post"
curl -i -X DELETE "$BASE_URL/api/blog/api-test-post"
echo 

echo "Test completed."
