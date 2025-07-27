#!/bin/bash
set -e

# Script to rebuild the Docker container with improved network resilience
echo "=== Rebuilding Docker container with improved network resilience ==="

# Change to the project root directory
cd "$(dirname "$0")/.."

# Stop any running containers
echo "Stopping any running containers..."
./scripts/deploy-local.sh stop || true

# Clean up any existing containers and images
echo "Cleaning up Docker resources..."
docker-compose -f docker/docker-compose.local.yml down --remove-orphans || true

# Rebuild the images with no cache to ensure all changes are applied
echo "Rebuilding Docker images with no cache..."
docker-compose -f docker/docker-compose.local.yml build --no-cache

# Start the containers
echo "Starting containers..."
./scripts/deploy-local.sh start

# Check the status
echo "Checking container status..."
./scripts/deploy-local.sh status

echo "=== Rebuild complete ==="
echo "If the container is running successfully, you can access the application at:"
echo "  - Main Website: http://localhost:3002"
echo "  - Blog API: http://localhost:3002/api/blog"
echo "  - Admin: http://localhost:3002/admin"
echo ""
echo "To view logs, run: ./scripts/deploy-local.sh logs"