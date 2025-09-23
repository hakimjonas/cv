#!/bin/bash
# Unified Docker build script with support for different configurations
# This script provides a single interface for building Docker images with
# different configurations (production, development, minimal)

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

# Default values
BUILD_TYPE="prod"
BUILD_VERSION="0.1.0"
PUSH=false
CACHE=true
PLATFORM="linux/amd64"
NETWORK_TEST=false
OFFLINE=false
REGISTRY=""
BUILDKIT=true
SCAN=false

# Get the absolute path to the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Function to display usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -t, --type TYPE       Build type: prod, dev, minimal (default: prod)"
    echo "  -v, --version VERSION Build version (default: 0.1.0)"
    echo "  -p, --push            Push the image to the registry"
    echo "  --no-cache            Disable Docker build cache"
    echo "  --platform PLATFORM   Build for specific platform (default: linux/amd64)"
    echo "  --network-test        Run network test before building"
    echo "  --offline             Use vendored dependencies (offline build)"
    echo "  --registry REGISTRY   Docker registry to push to"
    echo "  --no-buildkit         Disable BuildKit"
    echo "  --scan                Run security scan after build"
    echo "  -h, --help            Show this help message"
    echo
    echo "Examples:"
    echo "  $0 --type prod --version 1.0.0"
    echo "  $0 --type minimal --push --registry ghcr.io/username"
    echo "  $0 --type dev --no-cache"
    echo "  $0 --platform linux/arm64"
    echo "  $0 --network-test --offline"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            BUILD_TYPE="$2"
            shift 2
            ;;
        -v|--version)
            BUILD_VERSION="$2"
            shift 2
            ;;
        -p|--push)
            PUSH=true
            shift
            ;;
        --no-cache)
            CACHE=false
            shift
            ;;
        --platform)
            PLATFORM="$2"
            shift 2
            ;;
        --network-test)
            NETWORK_TEST=true
            shift
            ;;
        --offline)
            OFFLINE=true
            shift
            ;;
        --registry)
            REGISTRY="$2"
            shift 2
            ;;
        --no-buildkit)
            BUILDKIT=false
            shift
            ;;
        --scan)
            SCAN=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Validate build type
if [[ "$BUILD_TYPE" != "prod" && "$BUILD_TYPE" != "dev" && "$BUILD_TYPE" != "minimal" ]]; then
    print_error "Invalid build type: $BUILD_TYPE"
    print_info "Valid build types are: prod, dev, minimal"
    exit 1
fi

# Set image name based on build type
if [[ "$BUILD_TYPE" == "prod" ]]; then
    IMAGE_NAME="blog-api"
    DOCKERFILE="docker/Dockerfile.prod"
    COMPOSE_FILE="docker/docker-compose.prod.yml"
elif [[ "$BUILD_TYPE" == "dev" ]]; then
    IMAGE_NAME="blog-api-dev"
    DOCKERFILE="docker/Dockerfile.dev"
    COMPOSE_FILE="docker/docker-compose.dev.yml"
else
    IMAGE_NAME="blog-api-minimal"
    DOCKERFILE="docker/Dockerfile.minimal"
    COMPOSE_FILE="docker/docker-compose.prod.yml"
fi

# Set full image name with registry if provided
if [[ -n "$REGISTRY" ]]; then
    FULL_IMAGE_NAME="$REGISTRY/$IMAGE_NAME:$BUILD_VERSION"
else
    FULL_IMAGE_NAME="$IMAGE_NAME:$BUILD_VERSION"
fi

# Run network test if requested
if [[ "$NETWORK_TEST" == true ]]; then
    print_header "Running network test"
    "${PROJECT_ROOT}/scripts/network-test.sh"
    
    # Ask user if they want to continue after seeing the network test results
    read -p "Continue with the build? [Y/n] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        print_info "Build cancelled by user"
        exit 0
    fi
fi

# Set BuildKit environment variable
if [[ "$BUILDKIT" == true ]]; then
    export DOCKER_BUILDKIT=1
    print_info "BuildKit enabled"
else
    export DOCKER_BUILDKIT=0
    print_info "BuildKit disabled"
fi

# Set offline mode environment variable
if [[ "$OFFLINE" == true ]]; then
    export CARGO_NET_OFFLINE=true
    print_info "Offline mode enabled (using vendored dependencies)"
else
    unset CARGO_NET_OFFLINE
fi

# Print build configuration
print_header "Build Configuration"
echo "Build Type:    $BUILD_TYPE"
echo "Version:       $BUILD_VERSION"
echo "Image Name:    $FULL_IMAGE_NAME"
echo "Dockerfile:    $DOCKERFILE"
echo "Compose File:  $COMPOSE_FILE"
echo "Platform:      $PLATFORM"
echo "Cache:         $CACHE"
echo "Push:          $PUSH"
echo "Offline:       $OFFLINE"
echo "Registry:      ${REGISTRY:-none}"
echo "Scan:          $SCAN"

# Confirm build
read -p "Proceed with build? [Y/n] " -n 1 -r
echo
if [[ $REPLY =~ ^[Nn]$ ]]; then
    print_info "Build cancelled by user"
    exit 0
fi

# Start build timer
BUILD_START_TIME=$(date +%s)

# Build the image
print_header "Building Docker Image: $FULL_IMAGE_NAME"

# Prepare build arguments
BUILD_ARGS=(
    "--build-arg" "BUILD_VERSION=$BUILD_VERSION"
)

# Add platform argument if specified
if [[ "$PLATFORM" != "linux/amd64" ]]; then
    BUILD_ARGS+=("--platform" "$PLATFORM")
fi

# Add no-cache argument if specified
if [[ "$CACHE" == false ]]; then
    BUILD_ARGS+=("--no-cache")
fi

# Build the image using docker-compose
print_info "Building image using docker-compose..."
export BUILD_VERSION="$BUILD_VERSION"
export HOST_PORT=3000

# Use docker-compose to build the image
if ! docker-compose -f "$COMPOSE_FILE" build "${BUILD_ARGS[@]}"; then
    print_error "Build failed"
    exit 1
fi

# Calculate build time
BUILD_END_TIME=$(date +%s)
BUILD_DURATION=$((BUILD_END_TIME - BUILD_START_TIME))
BUILD_MINUTES=$((BUILD_DURATION / 60))
BUILD_SECONDS=$((BUILD_DURATION % 60))

print_info "Build completed in ${BUILD_MINUTES}m ${BUILD_SECONDS}s"

# Get image size
IMAGE_SIZE=$(docker images "$IMAGE_NAME:$BUILD_VERSION" --format "{{.Size}}")
print_info "Image size: $IMAGE_SIZE"

# Run security scan if requested
if [[ "$SCAN" == true ]]; then
    print_header "Running Security Scan"
    
    # Check if Trivy is installed
    if command -v trivy &> /dev/null; then
        print_info "Scanning image with Trivy..."
        trivy image "$FULL_IMAGE_NAME"
    else
        print_warning "Trivy is not installed. Skipping security scan."
        print_info "To install Trivy, follow the instructions at: https://aquasecurity.github.io/trivy/latest/getting-started/installation/"
    fi
fi

# Push the image if requested
if [[ "$PUSH" == true ]]; then
    print_header "Pushing Image to Registry"
    
    if [[ -z "$REGISTRY" ]]; then
        print_warning "No registry specified. Using default Docker Hub registry."
    fi
    
    print_info "Pushing image: $FULL_IMAGE_NAME"
    if ! docker push "$FULL_IMAGE_NAME"; then
        print_error "Failed to push image"
        print_info "Make sure you are logged in to the registry:"
        print_info "  docker login ${REGISTRY}"
        exit 1
    fi
    
    print_info "Image pushed successfully"
fi

# Print summary
print_header "Build Summary"
echo "Image:         $FULL_IMAGE_NAME"
echo "Size:          $IMAGE_SIZE"
echo "Build Time:    ${BUILD_MINUTES}m ${BUILD_SECONDS}s"
echo "Build Type:    $BUILD_TYPE"
echo "Platform:      $PLATFORM"

# Print next steps
print_header "Next Steps"
if [[ "$BUILD_TYPE" == "prod" ]]; then
    echo "To run the production image:"
    echo "  docker-compose -f docker/docker-compose.prod.yml up -d"
elif [[ "$BUILD_TYPE" == "dev" ]]; then
    echo "To run the development image with hot-reloading:"
    echo "  docker-compose -f docker/docker-compose.dev.yml up -d"
else
    echo "To run the minimal image:"
    echo "  docker-compose -f docker/docker-compose.prod.yml up -d"
fi

if [[ "$PUSH" == true ]]; then
    echo "To pull the image on another machine:"
    echo "  docker pull $FULL_IMAGE_NAME"
fi

print_info "Build script completed successfully"