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
    docker-compose -f docker-compose.local.yml up -d --build --remove-orphans

    echo "Waiting for services to start..."

    # Wait for the container to be up
    echo "Checking if container is running..."
    for i in {1..10}; do
        if docker-compose -f docker-compose.local.yml ps | grep -q "blog-api.*Up"; then
            echo "Container is up and running."
            break
        fi
        if [ $i -eq 10 ]; then
            echo "Failed to start container. Check the logs:"
            docker-compose -f docker-compose.local.yml logs
            exit 1
        fi
        echo "Waiting for container to start... (attempt $i/10)"
        sleep 3
    done

    # Wait for the application to be healthy
    echo "Waiting for application to be ready..."
    for i in {1..40}; do
        HEALTH_STATUS=$(docker inspect --format='{{.State.Health.Status}}' $(docker-compose -f docker-compose.local.yml ps -q blog-api) 2>/dev/null)
        if [ "$HEALTH_STATUS" = "healthy" ]; then
            echo "Application is ready!"
            echo "Local development environment is running!"
            echo ""
            echo "🌐 Main Website:"
            echo "  Homepage: http://localhost:3002"
            echo "  Blog: http://localhost:3002/blog.html"
            echo "  CV: http://localhost:3002/cv.html"
            echo "  Projects: http://localhost:3002/projects.html"
            echo ""
            echo "🔧 Development Tools:"
            echo "  Blog API: http://localhost:3002/api/blog"
            echo "  API Admin: http://localhost:3002/admin"
            echo "  Blog Client: http://localhost:3002/static/blog-client.html"
            echo "  Debug Tool: http://localhost:3002/static/blog-debug.html"
            return 0
        fi
        echo "Waiting for application to be ready... (attempt $i/40, status: ${HEALTH_STATUS:-unknown})"
        sleep 10
    done

    echo "Application failed to become ready in the expected time."
    echo "This is normal for the first run as Rust needs to compile the application."
    echo "Check the logs to see the compilation progress:"
    echo "  ./deploy-local.sh logs"
    echo "You can also check the status with:"
    echo "  ./deploy-local.sh status"
    echo
    echo "The application should be available at the following URLs once compilation is complete:"
    echo ""
    echo "🌐 Main Website:"
    echo "  Homepage: http://localhost:3002"
    echo "  Blog: http://localhost:3002/blog.html"
    echo "  CV: http://localhost:3002/cv.html"
    echo "  Projects: http://localhost:3002/projects.html"
    echo ""
    echo "🔧 Development Tools:"
    echo "  Blog API: http://localhost:3002/api/blog"
    echo "  API Admin: http://localhost:3002/admin"
    echo "  Blog Client: http://localhost:3002/static/blog-client.html"
    echo "  Debug Tool: http://localhost:3002/static/blog-debug.html"
}

# Function to stop the local development environment
stop_local_env() {
    echo "Stopping local development environment..."
    docker-compose -f docker-compose.local.yml down --remove-orphans
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
