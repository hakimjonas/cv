#!/bin/bash

set -e

echo "Starting blog application deployment"

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

# Build the Docker image
echo "Building Docker image..."
docker-compose build

# Start the service
echo "Starting the service..."
docker-compose up -d

# Wait for the service to start
echo "Waiting for the service to start..."
sleep 5

# Check if the service is running
if docker-compose ps | grep -q "blog-api.*Up"; then
    echo "Deployment successful! The blog API is running."
    echo "You can access it at http://localhost:3000"
else
    echo "Deployment failed. Please check the logs with 'docker-compose logs'."
    exit 1
fi

echo "Deployment completed!"
