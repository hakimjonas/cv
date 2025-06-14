#!/bin/bash

set -e

# Colors for output
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
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

# Create test directory if it doesn't exist
mkdir -p test_data

# Run unit tests
run_test "Unit Tests" "cargo test --lib"
#!/bin/bash

set -e

# Colors for output
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
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

# Create test directory if it doesn't exist
mkdir -p test_data

# Run unit tests
run_test "Unit Tests" "cargo test --lib"

# Run GitHub API tests
run_test "GitHub API Tests" "cargo test --test github_test"

# Run blog data tests
run_test "Blog Data Tests" "cargo test blog_data_test"

# Run blog core functionality test
run_test "Blog Core Functionality" "cargo run --bin blog_tester"

# Start the blog server in the background for API tests
echo -e "\n${YELLOW}Starting blog server for API tests...${NC}"
cargo run --bin blog_api_server &
SERVER_PID=$!

# Save the server PID to a file for later cleanup
echo $SERVER_PID > .blog_server_pid

# Wait for server to start
sleep 2

# Run API tests
run_test "Blog API Basic Test" "curl -s http://localhost:3000/api/blog > /dev/null"

# Run additional API tests if needed
if [ -f "blog-api-client.sh" ]; then
  run_test "Blog API Client Tests" "./blog-api-client.sh"
fi

# Kill the server
if [ -f ".blog_server_pid" ]; then
  echo -e "\n${YELLOW}Stopping blog server...${NC}"
  kill $(cat .blog_server_pid)
  rm .blog_server_pid
fi

echo -e "\n${GREEN}===== All tests completed =====${NC}"
# Run blog data tests
run_test "Blog Data Tests" "cargo test blog_data_test"

# Run blog core functionality test
run_test "Blog Core Functionality" "cargo run --bin blog_tester"

# Start the blog server in the background for API tests
echo -e "\n${YELLOW}Starting blog server for API tests...${NC}"
cargo run --bin blog_api_server &
SERVER_PID=$!

# Save the server PID to a file for later cleanup
echo $SERVER_PID > .blog_server_pid

# Wait for server to start
sleep 2

# Run API tests
run_test "Blog API Tests" "curl -s http://localhost:3000/api/blog > /dev/null"

# Run additional API tests if needed
if [ -f "blog-api-client.sh" ]; then
  run_test "Blog API Client Tests" "./blog-api-client.sh"
fi

# Kill the server
if [ -f ".blog_server_pid" ]; then
  echo -e "\n${YELLOW}Stopping blog server...${NC}"
  kill $(cat .blog_server_pid)
  rm .blog_server_pid
fi

echo -e "\n${GREEN}===== All tests completed =====${NC}"
