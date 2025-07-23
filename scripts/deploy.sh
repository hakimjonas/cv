#!/bin/bash

set -e

echo "Starting blog application deployment"

# Generate CV and website files
echo "Generating CV and website files..."
if ! command -v cargo &> /dev/null; then
    echo "Cargo is not installed. Please install Rust and Cargo before proceeding."
    exit 1
fi

cargo run --bin cv
echo "CV and website files generated successfully in dist/ directory"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Docker is not installed. Please install Docker before proceeding."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "Docker Compose is not installed. Please install Docker Compose before proceeding."
    exit 1
fi

# Function to check health status
check_health() {
    local retries=10
    local wait_time=3
    local container_name="blog-api"

    echo "Checking health status of $container_name..."

    for i in $(seq 1 $retries); do
        if docker inspect --format='{{.State.Health.Status}}' $(docker-compose ps -q $container_name) | grep -q "healthy"; then
            echo "$container_name is healthy!"
            return 0
        fi

        echo "Waiting for $container_name to become healthy (attempt $i/$retries)..."
        sleep $wait_time
    done

    echo "$container_name failed to become healthy after $retries attempts."
    return 1
}

# Build the Docker image
echo "Building Docker image..."
docker-compose build

# Check if the service is already running
if docker-compose ps | grep -q "blog-api.*Up"; then
    echo "Service is already running. Performing rolling update..."

    # Pull the latest image (if using a registry)
    # docker-compose pull

    # Update the service with zero downtime
    docker-compose up -d --no-deps --build blog-api

    # Check health status
    if ! check_health; then
        echo "Rolling update failed. Rolling back..."
        docker-compose logs blog-api
        # Here you would implement rollback logic if needed
        exit 1
    fi
else
    # Start the service for the first time
    echo "Starting the service for the first time..."
    docker-compose up -d

    # Check health status
    if ! check_health; then
        echo "Initial deployment failed. Check the logs:"
        docker-compose logs blog-api
        exit 1
    fi
fi

echo "Deployment successful! The blog API is running."
echo "You can access it at http://localhost:3000"

# Perform database migrations if needed
# echo "Running database migrations..."
# docker-compose exec blog-api /app/run_migrations.sh

echo "Deployment completed!"
