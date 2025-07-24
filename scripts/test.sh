#!/bin/bash

set -e

# Colors for output
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
BLUE="\033[0;34m"
NC="\033[0m" # No Color

echo -e "${YELLOW}===== Starting Comprehensive Test Suite =====${NC}"

# Function to run a test and report status
run_test() {
  local test_name=$1
  local test_cmd=$2

  echo -e "\n${YELLOW}Running test: ${test_name}${NC}"
  if eval $test_cmd; then
    echo -e "${GREEN}✓ ${test_name} passed${NC}"
    return 0
  else
    echo -e "${RED}✗ ${test_name} failed${NC}"
    return 1
  fi
}

# Function to check if a binary exists
binary_exists() {
  cargo metadata --format-version=1 | grep -q "\"name\":\"$1\""
}

# Create test directory if it doesn't exist
mkdir -p test_data

# Print test environment information
echo -e "${BLUE}Test Environment:${NC}"
echo -e "Rust version: $(rustc --version)"
echo -e "Cargo version: $(cargo --version)"
echo -e "Working directory: $(pwd)"
echo -e "Date: $(date)"
echo -e ""

# Run unit tests
run_test "Unit Tests" "cargo test --lib"

# Run GitHub API tests
run_test "GitHub API Tests" "cargo test --test github_test"

# Run blog data tests
run_test "Blog Data Tests" "cargo test blog_data_test"

# Run blog property tests
run_test "Blog Property Tests" "cargo test --test blog_property_test"

# Run blog core functionality test
if binary_exists "test_blog_core"; then
  run_test "Blog Core Functionality" "cargo run --bin test_blog_core"
elif binary_exists "blog_tester"; then
  run_test "Blog Core Functionality" "cargo run --bin blog_tester"
else
  echo -e "${YELLOW}Skipping Blog Core Functionality test - binary not found${NC}"
fi

# Run security tests if available
if binary_exists "security_test"; then
  run_test "Security Tests" "cargo run --bin security_test"
else
  echo -e "${YELLOW}Skipping Security Tests - binary not found${NC}"
fi

# Determine port for API tests
# Use port 3002 for local development environment
API_PORT=3002

# Start the blog server in the background for API tests
echo -e "\n${YELLOW}Starting blog server for API tests on port ${API_PORT}...${NC}"

# Check if server is already running on the port
if netstat -tuln | grep -q ":${API_PORT} "; then
  echo -e "${YELLOW}Server already running on port ${API_PORT}${NC}"
  EXTERNAL_SERVER=true
else
  cargo run --bin blog_api_server -- --port ${API_PORT} &
  SERVER_PID=$!
  
  # Save the server PID to a file for later cleanup
  echo $SERVER_PID > .blog_server_pid
  EXTERNAL_SERVER=false
  
  # Wait for server to start
  echo -e "${YELLOW}Waiting for server to start...${NC}"
  for i in {1..10}; do
    if curl -s http://localhost:${API_PORT}/health > /dev/null 2>&1; then
      echo -e "${GREEN}Server started successfully${NC}"
      break
    fi
    
    if [ $i -eq 10 ]; then
      echo -e "${RED}Server failed to start after 10 attempts${NC}"
      if [ "$EXTERNAL_SERVER" = false ] && [ -f ".blog_server_pid" ]; then
        kill $(cat .blog_server_pid)
        rm .blog_server_pid
      fi
      exit 1
    fi
    
    echo -e "${YELLOW}Waiting for server to start (attempt $i/10)...${NC}"
    sleep 2
  done
fi

# Run API tests
run_test "Blog API Health Check" "curl -s http://localhost:${API_PORT}/health > /dev/null"
run_test "Blog API Basic Test" "curl -s http://localhost:${API_PORT}/api/blog > /dev/null"

# Run additional API tests if needed
if [ -f "blog-api-client.sh" ]; then
  run_test "Blog API Client Tests" "./blog-api-client.sh"
fi

# Kill the server if we started it
if [ "$EXTERNAL_SERVER" = false ] && [ -f ".blog_server_pid" ]; then
  echo -e "\n${YELLOW}Stopping blog server...${NC}"
  kill $(cat .blog_server_pid)
  rm .blog_server_pid
fi

echo -e "\n${GREEN}===== All tests completed =====${NC}"
