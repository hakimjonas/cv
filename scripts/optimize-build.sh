#!/bin/bash
# Build optimization utilities for Rust Docker builds
# This script provides utilities for optimizing Docker builds, including:
# - Dependency caching
# - Vendoring dependencies for offline builds
# - Cleaning up build artifacts
# - Benchmarking build performance

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

# Get the absolute path to the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Default values
ACTION=""
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
VENDOR_DIR="${PROJECT_ROOT}/vendor"
REGISTRY_DIR="${CARGO_HOME}/registry"
CACHE_DIR="${PROJECT_ROOT}/.cargo-cache"
PLATFORM="x86_64-unknown-linux-musl"
CLEAN_LEVEL="normal"
BENCHMARK_ITERATIONS=3

# Function to display usage information
show_usage() {
    echo "Usage: $0 [ACTION] [OPTIONS]"
    echo
    echo "Actions:"
    echo "  vendor       Vendor dependencies for offline builds"
    echo "  cache        Set up dependency caching"
    echo "  clean        Clean up build artifacts"
    echo "  benchmark    Benchmark build performance"
    echo
    echo "Options:"
    echo "  --vendor-dir DIR      Directory for vendored dependencies (default: ./vendor)"
    echo "  --cache-dir DIR       Directory for cached dependencies (default: ./.cargo-cache)"
    echo "  --platform PLATFORM   Target platform (default: x86_64-unknown-linux-musl)"
    echo "  --clean-level LEVEL   Clean level: normal, deep, all (default: normal)"
    echo "  --iterations N        Number of benchmark iterations (default: 3)"
    echo "  -h, --help            Show this help message"
    echo
    echo "Examples:"
    echo "  $0 vendor --vendor-dir ./deps"
    echo "  $0 cache --cache-dir /tmp/cargo-cache"
    echo "  $0 clean --clean-level deep"
    echo "  $0 benchmark --iterations 5"
}

# Parse command line arguments
if [ $# -eq 0 ]; then
    show_usage
    exit 1
fi

ACTION="$1"
shift

while [[ $# -gt 0 ]]; do
    case $1 in
        --vendor-dir)
            VENDOR_DIR="$2"
            shift 2
            ;;
        --cache-dir)
            CACHE_DIR="$2"
            shift 2
            ;;
        --platform)
            PLATFORM="$2"
            shift 2
            ;;
        --clean-level)
            CLEAN_LEVEL="$2"
            shift 2
            ;;
        --iterations)
            BENCHMARK_ITERATIONS="$2"
            shift 2
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

# Validate action
if [[ "$ACTION" != "vendor" && "$ACTION" != "cache" && "$ACTION" != "clean" && "$ACTION" != "benchmark" ]]; then
    print_error "Invalid action: $ACTION"
    print_info "Valid actions are: vendor, cache, clean, benchmark"
    exit 1
fi

# Function to check if cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed"
        print_info "Please install Rust and Cargo: https://www.rust-lang.org/tools/install"
        exit 1
    fi
}

# Function to vendor dependencies
vendor_dependencies() {
    print_header "Vendoring Dependencies for Offline Builds"
    
    check_cargo
    
    # Create vendor directory if it doesn't exist
    mkdir -p "$VENDOR_DIR"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    print_info "Vendoring dependencies to $VENDOR_DIR..."
    
    # Vendor dependencies
    if cargo vendor --versioned-dirs "$VENDOR_DIR"; then
        print_info "Dependencies vendored successfully"
        
        # Create .cargo/config.toml if it doesn't exist
        mkdir -p "${PROJECT_ROOT}/.cargo"
        
        # Check if .cargo/config.toml exists
        if [ -f "${PROJECT_ROOT}/.cargo/config.toml" ]; then
            print_info "Updating existing .cargo/config.toml"
            
            # Check if [source.crates-io] section exists
            if grep -q "\[source.crates-io\]" "${PROJECT_ROOT}/.cargo/config.toml"; then
                # Update existing section
                sed -i '/\[source.crates-io\]/,/^$/d' "${PROJECT_ROOT}/.cargo/config.toml"
            fi
            
            # Add vendor configuration
            cat >> "${PROJECT_ROOT}/.cargo/config.toml" << EOF

[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "${VENDOR_DIR}"
EOF
        else
            print_info "Creating new .cargo/config.toml"
            
            # Create new config file
            cat > "${PROJECT_ROOT}/.cargo/config.toml" << EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "${VENDOR_DIR}"

[build]
target = "${PLATFORM}"

[target.${PLATFORM}]
linker = "gcc"
EOF
        fi
        
        print_info "Cargo configuration updated for offline builds"
        print_info "To build in offline mode, use: CARGO_NET_OFFLINE=true cargo build"
        print_info "Or with Docker: ./scripts/build-docker.sh --offline"
        
        # Calculate size of vendored dependencies
        VENDOR_SIZE=$(du -sh "$VENDOR_DIR" | cut -f1)
        print_info "Vendored dependencies size: $VENDOR_SIZE"
    else
        print_error "Failed to vendor dependencies"
        exit 1
    fi
}

# Function to set up dependency caching
setup_caching() {
    print_header "Setting Up Dependency Caching"
    
    check_cargo
    
    # Create cache directory if it doesn't exist
    mkdir -p "$CACHE_DIR"
    mkdir -p "$CACHE_DIR/registry"
    mkdir -p "$CACHE_DIR/git"
    
    print_info "Setting up dependency caching in $CACHE_DIR..."
    
    # Create .cargo/config.toml if it doesn't exist
    mkdir -p "${PROJECT_ROOT}/.cargo"
    
    # Check if .cargo/config.toml exists
    if [ -f "${PROJECT_ROOT}/.cargo/config.toml" ]; then
        print_info "Updating existing .cargo/config.toml"
        
        # Check if [build] section exists
        if grep -q "\[build\]" "${PROJECT_ROOT}/.cargo/config.toml"; then
            # Update existing section
            sed -i '/\[build\]/,/^$/d' "${PROJECT_ROOT}/.cargo/config.toml"
        fi
    else
        print_info "Creating new .cargo/config.toml"
    fi
    
    # Add cache configuration
    cat >> "${PROJECT_ROOT}/.cargo/config.toml" << EOF
[build]
target-dir = "${CACHE_DIR}/target"

[registry]
index = "sparse+https://index.crates.io/"

[net]
retry = 3
git-fetch-with-cli = true
EOF
    
    print_info "Cargo configuration updated for dependency caching"
    
    # Create Docker volume for caching
    print_info "Creating Docker volume for caching..."
    docker volume create cargo-registry-cache
    
    # Update docker-compose files to use the cache volume
    for compose_file in "${PROJECT_ROOT}/docker/docker-compose"*.yml; do
        if [ -f "$compose_file" ]; then
            print_info "Updating $compose_file..."
            
            # Check if the file already has the volume
            if grep -q "cargo-registry-cache" "$compose_file"; then
                print_info "Cache volume already configured in $compose_file"
            else
                # Add the volume to the services section
                sed -i '/volumes:/a\      - cargo-registry-cache:/root/.cargo/registry' "$compose_file"
                
                # Add the volume to the volumes section
                if grep -q "volumes:" "$compose_file"; then
                    sed -i '/volumes:/a\  cargo-registry-cache:' "$compose_file"
                else
                    echo -e "\nvolumes:\n  cargo-registry-cache:" >> "$compose_file"
                fi
                
                print_info "Cache volume added to $compose_file"
            fi
        fi
    done
    
    print_info "Dependency caching set up successfully"
    print_info "Cached dependencies will be stored in Docker volume: cargo-registry-cache"
}

# Function to clean up build artifacts
clean_artifacts() {
    print_header "Cleaning Build Artifacts"
    
    check_cargo
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    if [ "$CLEAN_LEVEL" = "normal" ]; then
        print_info "Cleaning normal build artifacts..."
        cargo clean
        
        print_info "Removing temporary files..."
        find . -name "*.rs.bk" -delete
        find . -name "*.orig" -delete
        find . -name "*.rej" -delete
        
    elif [ "$CLEAN_LEVEL" = "deep" ]; then
        print_info "Performing deep clean..."
        cargo clean
        
        print_info "Removing temporary files..."
        find . -name "*.rs.bk" -delete
        find . -name "*.orig" -delete
        find . -name "*.rej" -delete
        
        print_info "Removing Cargo registry cache..."
        rm -rf "${REGISTRY_DIR}/cache"
        rm -rf "${REGISTRY_DIR}/index"
        
        print_info "Removing local cache..."
        rm -rf "${CACHE_DIR}"
        
    elif [ "$CLEAN_LEVEL" = "all" ]; then
        print_info "Performing complete clean..."
        cargo clean
        
        print_info "Removing temporary files..."
        find . -name "*.rs.bk" -delete
        find . -name "*.orig" -delete
        find . -name "*.rej" -delete
        
        print_info "Removing Cargo registry cache..."
        rm -rf "${REGISTRY_DIR}"
        
        print_info "Removing local cache..."
        rm -rf "${CACHE_DIR}"
        
        print_info "Removing vendored dependencies..."
        rm -rf "${VENDOR_DIR}"
        
        print_info "Removing Docker build cache..."
        docker builder prune -f
        
        print_info "Removing Docker volume cache..."
        docker volume rm cargo-registry-cache || true
    else
        print_error "Invalid clean level: $CLEAN_LEVEL"
        print_info "Valid clean levels are: normal, deep, all"
        exit 1
    fi
    
    print_info "Clean completed successfully"
}

# Function to benchmark build performance
benchmark_build() {
    print_header "Benchmarking Build Performance"
    
    check_cargo
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    print_info "Running $BENCHMARK_ITERATIONS build iterations..."
    
    # Clean first to ensure consistent results
    print_info "Cleaning before benchmark..."
    cargo clean
    
    # Run benchmark iterations
    total_time=0
    best_time=999999
    worst_time=0
    
    for i in $(seq 1 $BENCHMARK_ITERATIONS); do
        print_info "Build iteration $i/$BENCHMARK_ITERATIONS..."
        
        # Clean target directory but keep registry cache
        cargo clean
        
        # Measure build time
        start_time=$(date +%s)
        
        if cargo build --release --target "$PLATFORM"; then
            end_time=$(date +%s)
            build_time=$((end_time - start_time))
            
            print_info "Build time: ${build_time}s"
            
            # Update statistics
            total_time=$((total_time + build_time))
            
            if [ "$build_time" -lt "$best_time" ]; then
                best_time=$build_time
            fi
            
            if [ "$build_time" -gt "$worst_time" ]; then
                worst_time=$build_time
            fi
        else
            print_error "Build failed"
            exit 1
        fi
    done
    
    # Calculate average
    average_time=$((total_time / BENCHMARK_ITERATIONS))
    
    print_header "Benchmark Results"
    echo "Iterations:    $BENCHMARK_ITERATIONS"
    echo "Average time:  ${average_time}s"
    echo "Best time:     ${best_time}s"
    echo "Worst time:    ${worst_time}s"
    
    # Measure binary size
    binary_size=$(du -h "target/${PLATFORM}/release/blog_api_server" 2>/dev/null | cut -f1 || echo "N/A")
    echo "Binary size:   $binary_size"
    
    # Run Docker build benchmark if Docker is available
    if command -v docker &> /dev/null; then
        print_info "Running Docker build benchmark..."
        
        # Clean Docker build cache
        docker builder prune -f -q
        
        # Measure Docker build time
        start_time=$(date +%s)
        
        if docker build -f "${PROJECT_ROOT}/docker/Dockerfile.prod" -t benchmark-test:latest "${PROJECT_ROOT}"; then
            end_time=$(date +%s)
            docker_build_time=$((end_time - start_time))
            
            print_info "Docker build time: ${docker_build_time}s"
            
            # Get image size
            image_size=$(docker images benchmark-test:latest --format "{{.Size}}")
            
            echo "Docker build time: ${docker_build_time}s"
            echo "Docker image size: $image_size"
            
            # Clean up
            docker rmi benchmark-test:latest
        else
            print_error "Docker build failed"
        fi
    else
        print_warning "Docker not available, skipping Docker build benchmark"
    fi
    
    print_info "Benchmark completed successfully"
}

# Execute the requested action
case "$ACTION" in
    vendor)
        vendor_dependencies
        ;;
    cache)
        setup_caching
        ;;
    clean)
        clean_artifacts
        ;;
    benchmark)
        benchmark_build
        ;;
esac