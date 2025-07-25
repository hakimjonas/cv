#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_header "Starting local development environment"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker before proceeding."
    exit 1
fi

# Check for Docker Compose
# First check for docker compose (v2)
if docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
    print_info "Using Docker Compose v2"
# Fall back to docker-compose (v1)
elif command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
    print_warning "Using legacy Docker Compose v1. Consider upgrading to Docker Compose v2."
else
    print_error "Docker Compose is not installed. Please install Docker Compose before proceeding."
    exit 1
fi

# Function to display usage information
show_usage() {
    echo -e "${BLUE}Usage:${NC} $0 [OPTION]"
    echo -e "${BLUE}Options:${NC}"
    echo "  start    Start the local development environment"
    echo "  stop     Stop the local development environment"
    echo "  restart  Restart the local development environment"
    echo "  logs     Show logs from the containers"
    echo "  status   Show the status of the containers"
    echo "  rebuild  Rebuild the application (preserves data)"
    echo "  prune    Remove unused Docker resources"
    echo "  help     Show this help message"
}

# Get the absolute path to the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCKER_COMPOSE_FILE="${PROJECT_ROOT}/docker/docker-compose.local.yml"

# Default port for the application
DEFAULT_PORT=3002

# Function to check if a port is in use
check_port_in_use() {
    local port=$1
    if command -v nc &> /dev/null; then
        nc -z localhost "$port" &> /dev/null
        # nc returns 0 if port is in use, so return 0 (true) if port is in use
        return $?
    elif command -v lsof &> /dev/null; then
        lsof -i:"$port" &> /dev/null
        # lsof returns 0 if port is in use, so return 0 (true) if port is in use
        return $?
    else
        # Fallback to netstat if nc and lsof are not available
        netstat -tuln | grep -q ":$port " &> /dev/null
        # netstat|grep returns 0 if port is in use, so return 0 (true) if port is in use
        return $?
    fi
}

# Function to find an available port
find_available_port() {
    local port=$1
    local max_port=$((port + 100))  # Try up to 100 ports higher
    
    while [ "$port" -le "$max_port" ]; do
        if ! check_port_in_use "$port"; then
            # Double-check with a different method if possible
            if command -v lsof &> /dev/null && command -v nc &> /dev/null; then
                # If we have both tools, use the other one to double-check
                if [ "$(command -v nc)" != "" ] && nc -z localhost "$port" &> /dev/null; then
                    # Port is actually in use according to nc, try next port
                    port=$((port + 1))
                    continue
                elif [ "$(command -v lsof)" != "" ] && lsof -i:"$port" &> /dev/null; then
                    # Port is actually in use according to lsof, try next port
                    port=$((port + 1))
                    continue
                fi
            fi
            
            echo "$port"
            return 0
        fi
        port=$((port + 1))
    done
    
    # If no port is found, return the default port and let Docker handle the error
    echo "$DEFAULT_PORT"
    return 1
}

# Function to start the local development environment
start_local_env() {
    print_header "Building and starting local development environment"
    
    # Check if HOST_PORT is already set
    if [ -z "${HOST_PORT}" ]; then
        # Check if default port is in use
        if check_port_in_use "$DEFAULT_PORT"; then
            print_warning "Port $DEFAULT_PORT is already in use."
            # Find an available port
            HOST_PORT=$(find_available_port $((DEFAULT_PORT + 1)))
            if [ "$HOST_PORT" != "$DEFAULT_PORT" ]; then
                print_info "Using alternative port: $HOST_PORT"
                export HOST_PORT
            else
                print_error "Could not find an available port in the range $((DEFAULT_PORT + 1))-$((DEFAULT_PORT + 100))."
                print_info "You can manually specify a port with: HOST_PORT=<port> $0 $1"
                print_info "Or try stopping any services that might be using port $DEFAULT_PORT with: sudo lsof -i:$DEFAULT_PORT"
                exit 1
            fi
        else
            # Default port is available, use it
            print_info "Default port $DEFAULT_PORT is available."
            export HOST_PORT=$DEFAULT_PORT
        fi
    else
        # HOST_PORT is set, check if it's available
        if check_port_in_use "$HOST_PORT"; then
            print_error "Specified port $HOST_PORT is already in use."
            print_info "You can try a different port with: HOST_PORT=<port> $0 $1"
            print_info "Or try stopping any services that might be using port $HOST_PORT with: sudo lsof -i:$HOST_PORT"
            exit 1
        else
            print_info "Specified port $HOST_PORT is available."
        fi
    fi
    
    print_info "Using port: $HOST_PORT"
    
    # Run docker-compose and capture output to check for port binding errors
    DOCKER_OUTPUT=$($DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" up -d --build --remove-orphans 2>&1)
    DOCKER_EXIT_CODE=$?
    
    # Check if there was a port binding error
    if [ $DOCKER_EXIT_CODE -ne 0 ] && echo "$DOCKER_OUTPUT" | grep -q "Bind for.*:$HOST_PORT.*port is already allocated"; then
        print_error "Docker failed to bind to port $HOST_PORT even though it appeared to be available."
        print_info "This could be due to:"
        print_info "1. A recently stopped container that hasn't fully released the port"
        print_info "2. Another Docker service using the port"
        print_info "3. A system-level restriction on the port"
        print_info ""
        print_info "Try one of the following solutions:"
        print_info "1. Wait a few seconds and try again"
        print_info "2. Try a different port: HOST_PORT=<port> $0 $1"
        print_info "3. Restart Docker: sudo systemctl restart docker"
        print_info "4. Check for stale Docker resources: $0 prune"
        exit 1
    elif [ $DOCKER_EXIT_CODE -ne 0 ]; then
        print_error "Docker Compose failed with an unexpected error:"
        echo "$DOCKER_OUTPUT"
        exit 1
    fi
    
    print_info "Waiting for services to start..."

    # Wait for the container to be up
    print_info "Checking if container is running..."
    local max_attempts=10
    local wait_time=3
    for i in $(seq 1 $max_attempts); do
        if $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" ps | grep -q "blog-api.*Up"; then
            print_info "Container is up and running."
            break
        fi
        if [ $i -eq $max_attempts ]; then
            print_error "Failed to start container after $max_attempts attempts."
            print_info "Checking logs for potential issues:"
            $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" logs
            exit 1
        fi
        print_info "Waiting for container to start... (attempt $i/$max_attempts)"
        sleep $wait_time
    done

    # Wait for the application to be healthy
    print_info "Waiting for application to be ready..."
    local health_max_attempts=30  # Reduced from 40 to 30
    local health_wait_time=10
    for i in $(seq 1 $health_max_attempts); do
        HEALTH_STATUS=$(docker inspect --format='{{.State.Health.Status}}' $($DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" ps -q blog-api) 2>/dev/null)
        if [ "$HEALTH_STATUS" = "healthy" ]; then
            print_header "Application is ready!"
            print_info "Local development environment is running!"
            echo ""
            echo -e "${GREEN}🌐 Main Website:${NC}"
            echo "  Homepage: http://localhost:${HOST_PORT}"
            echo "  Blog: http://localhost:${HOST_PORT}/blog.html"
            echo "  CV: http://localhost:${HOST_PORT}/cv.html"
            echo "  Projects: http://localhost:${HOST_PORT}/projects.html"
            echo ""
            echo -e "${GREEN}🔧 Development Tools:${NC}"
            echo "  Blog API: http://localhost:${HOST_PORT}/api/blog"
            echo "  API Admin: http://localhost:${HOST_PORT}/admin"
            echo "  Blog Client: http://localhost:${HOST_PORT}/static/blog-client.html"
            echo "  Debug Tool: http://localhost:${HOST_PORT}/static/blog-debug.html"
            return 0
        fi
        print_info "Waiting for application to be ready... (attempt $i/$health_max_attempts, status: ${HEALTH_STATUS:-unknown})"
        sleep $health_wait_time
    done

    print_warning "Application failed to become ready in the expected time."
    print_info "This is normal for the first run as Rust needs to compile the application."
    print_info "Check the logs to see the compilation progress:"
    echo "  ./scripts/deploy-local.sh logs"
    print_info "You can also check the status with:"
    echo "  ./scripts/deploy-local.sh status"
    echo
    print_info "The application should be available at the following URLs once compilation is complete:"
    echo ""
    echo -e "${GREEN}🌐 Main Website:${NC}"
    echo "  Homepage: http://localhost:${HOST_PORT}"
    echo "  Blog: http://localhost:${HOST_PORT}/blog.html"
    echo "  CV: http://localhost:${HOST_PORT}/cv.html"
    echo "  Projects: http://localhost:${HOST_PORT}/projects.html"
    echo ""
    echo -e "${GREEN}🔧 Development Tools:${NC}"
    echo "  Blog API: http://localhost:${HOST_PORT}/api/blog"
    echo "  API Admin: http://localhost:${HOST_PORT}/admin"
    echo "  Blog Client: http://localhost:${HOST_PORT}/static/blog-client.html"
    echo "  Debug Tool: http://localhost:${HOST_PORT}/static/blog-debug.html"
}

# Function to stop the local development environment
stop_local_env() {
    print_header "Stopping local development environment"
    $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" down --remove-orphans
    print_info "Local development environment stopped."
}

# Function to show logs
show_logs() {
    print_header "Showing logs from local development environment"
    $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" logs -f
}

# Function to show status
show_status() {
    print_header "Status of local development environment"
    $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" ps
}

# Function to prune unused Docker resources
prune_resources() {
    print_header "Pruning unused Docker resources"

    print_info "This will remove all unused containers, networks, images, and volumes."
    read -p "Are you sure you want to continue? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Stopping any running containers first..."
        $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" down --remove-orphans

        print_info "Pruning containers..."
        docker container prune -f

        print_info "Pruning networks..."
        docker network prune -f

        print_info "Pruning images..."
        docker image prune -f

        print_info "Pruning volumes (unused)..."
        docker volume prune -f

        print_info "Docker resources pruned successfully."
    else
        print_info "Pruning cancelled."
    fi
}

# Function to rebuild the application
rebuild_app() {
    print_header "Rebuilding the application"

    print_info "This will stop the current containers, rebuild the images, and restart the application."
    print_info "Any data in volumes will be preserved."

    print_info "Stopping containers..."
    $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" down

    print_info "Rebuilding images..."
    $DOCKER_COMPOSE -f "${DOCKER_COMPOSE_FILE}" build --no-cache

    print_info "Starting containers..."
    start_local_env
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
    rebuild)
        rebuild_app
        ;;
    prune)
        prune_resources
        ;;
    help)
        show_usage
        ;;
    *)
        print_error "Unknown option: $1"
        show_usage
        exit 1
        ;;
esac

exit 0