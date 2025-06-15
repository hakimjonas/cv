#!/bin/bash

set -e

echo "Starting local development environment"

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

# Function to display usage information
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo "Options:"
    echo "  start    Start the local development environment"
    echo "  stop     Stop the local development environment"
    echo "  restart  Restart the local development environment"
    echo "  logs     Show logs from the containers"
    echo "  status   Show the status of the containers"
    echo "  help     Show this help message"
}

# Function to start the local development environment
start_local_env() {
    echo "Building and starting local development environment..."
    docker-compose -f docker-compose.local.yml up -d --build

    echo "Waiting for services to start..."
    sleep 5

    # Check if the service is running
    if docker-compose -f docker-compose.local.yml ps | grep -q "blog-api.*Up"; then
        echo "Local development environment is running!"
        echo "You can access the blog API at http://localhost:3002"
        echo "You can access the blog client at http://localhost:3002/static/blog-client.html"
        echo "You can access the debug tool at http://localhost:3002/static/blog-debug.html"
    else
        echo "Failed to start local development environment. Check the logs:"
        docker-compose -f docker-compose.local.yml logs
        exit 1
    fi
}

# Function to stop the local development environment
stop_local_env() {
    echo "Stopping local development environment..."
    docker-compose -f docker-compose.local.yml down
    echo "Local development environment stopped."
}

# Function to show logs
show_logs() {
    echo "Showing logs from local development environment..."
    docker-compose -f docker-compose.local.yml logs -f
}

# Function to show status
show_status() {
    echo "Status of local development environment:"
    docker-compose -f docker-compose.local.yml ps
}

# Parse command line arguments
if [ $# -eq 0 ]; then
    show_usage
    exit 0
fi

case "$1" in
    start)
        start_local_env
        ;;
    stop)
        stop_local_env
        ;;
    restart)
        stop_local_env
        start_local_env
        ;;
    logs)
        show_logs
        ;;
    status)
        show_status
        ;;
    help)
        show_usage
        ;;
    *)
        echo "Unknown option: $1"
        show_usage
        exit 1
        ;;
esac

exit 0
