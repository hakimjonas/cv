#!/bin/bash

# Kill any existing blog server processes
echo "Stopping any running blog servers..."
bash ./kill-blog-servers.sh

# Make the script executable
chmod +x "$0"

# Build the blog API server
echo "Building blog API server..."
cargo build --bin blog_api_server

# Run the server
echo "Starting blog API server..."
cargo run --bin blog_api_server
