#!/bin/bash
# Network connectivity diagnostics script for Docker builds
# This script tests various network aspects that might affect Docker builds,
# particularly for Rust projects pulling dependencies from crates.io

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

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to test DNS resolution
test_dns_resolution() {
    print_header "Testing DNS Resolution"
    
    # List of domains to test
    domains=(
        "crates.io"
        "static.crates.io"
        "github.com"
        "api.github.com"
        "index.docker.io"
        "registry.hub.docker.com"
        "production.cloudflare.docker.com"
    )
    
    # Test each domain
    for domain in "${domains[@]}"; do
        echo -n "Resolving $domain... "
        if host "$domain" > /dev/null 2>&1; then
            echo -e "${GREEN}OK${NC}"
            # Show the IP address
            ip=$(host "$domain" | grep "has address" | head -1 | awk '{print $4}')
            if [ -n "$ip" ]; then
                echo "  IP address: $ip"
            fi
        else
            echo -e "${RED}FAILED${NC}"
            print_error "Could not resolve $domain"
        fi
    done
    
    # Test DNS servers
    print_info "Testing DNS servers..."
    
    # Check if systemd-resolve is available
    if command_exists systemd-resolve; then
        echo "DNS servers from systemd-resolve:"
        systemd-resolve --status | grep "DNS Servers" -A2
    # Check if resolvectl is available
    elif command_exists resolvectl; then
        echo "DNS servers from resolvectl:"
        resolvectl status | grep "DNS Servers" -A2
    # Check /etc/resolv.conf
    elif [ -f /etc/resolv.conf ]; then
        echo "DNS servers from /etc/resolv.conf:"
        grep "nameserver" /etc/resolv.conf
    else
        print_warning "Could not determine DNS servers"
    fi
}

# Function to test HTTP connectivity
test_http_connectivity() {
    print_header "Testing HTTP Connectivity"
    
    # List of URLs to test
    urls=(
        "https://crates.io"
        "https://static.crates.io"
        "https://github.com"
        "https://api.github.com"
        "https://registry.hub.docker.com"
    )
    
    # Test each URL
    for url in "${urls[@]}"; do
        echo -n "Connecting to $url... "
        if curl -s -o /dev/null -w "%{http_code}" "$url" | grep -q "^[23]"; then
            echo -e "${GREEN}OK${NC}"
            # Measure response time
            time=$(curl -s -o /dev/null -w "%{time_total}" "$url")
            echo "  Response time: ${time}s"
        else
            echo -e "${RED}FAILED${NC}"
            print_error "Could not connect to $url"
        fi
    done
}

# Function to test Docker registry connectivity
test_docker_registry() {
    print_header "Testing Docker Registry Connectivity"
    
    if ! command_exists docker; then
        print_error "Docker is not installed"
        return 1
    fi
    
    echo -n "Pulling hello-world image... "
    if docker pull hello-world:latest > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "Could not pull hello-world image"
    fi
    
    echo -n "Docker registry login status... "
    if docker info 2>/dev/null | grep -q "Username"; then
        echo -e "${GREEN}LOGGED IN${NC}"
    else
        echo -e "${YELLOW}NOT LOGGED IN${NC}"
        print_warning "Not logged in to Docker registry (this may be normal)"
    fi
}

# Function to test Cargo registry connectivity
test_cargo_registry() {
    print_header "Testing Cargo Registry Connectivity"
    
    if ! command_exists cargo; then
        print_error "Cargo is not installed"
        return 1
    fi
    
    # Create a temporary directory
    temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Create a minimal Cargo project
    print_info "Creating a minimal Cargo project..."
    cargo init --bin test-project > /dev/null 2>&1
    cd test-project
    
    # Add a dependency
    echo 'serde = "1.0"' >> Cargo.toml
    
    # Try to fetch the dependency
    echo -n "Fetching dependency... "
    if cargo fetch > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "Could not fetch dependency"
    fi
    
    # Clean up
    cd ../..
    rm -rf "$temp_dir"
}

# Function to test network latency
test_network_latency() {
    print_header "Testing Network Latency"
    
    # List of hosts to test
    hosts=(
        "crates.io"
        "github.com"
        "1.1.1.1"
        "8.8.8.8"
    )
    
    # Test each host
    for host in "${hosts[@]}"; do
        echo -n "Pinging $host... "
        if ping -c 3 "$host" > /dev/null 2>&1; then
            echo -e "${GREEN}OK${NC}"
            # Show the average latency
            avg=$(ping -c 3 "$host" | tail -1 | awk '{print $4}' | cut -d '/' -f 2)
            echo "  Average latency: ${avg}ms"
        else
            echo -e "${RED}FAILED${NC}"
            print_error "Could not ping $host"
        fi
    done
}

# Function to test IPv6 connectivity
test_ipv6_connectivity() {
    print_header "Testing IPv6 Connectivity"
    
    # Check if IPv6 is enabled
    echo -n "IPv6 support... "
    if [ -f /proc/net/if_inet6 ] && [ -s /proc/net/if_inet6 ]; then
        echo -e "${GREEN}ENABLED${NC}"
    else
        echo -e "${YELLOW}DISABLED${NC}"
        print_warning "IPv6 is disabled on this system"
        return
    fi
    
    # Test IPv6 connectivity
    echo -n "IPv6 connectivity... "
    if ping6 -c 3 ipv6.google.com > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${YELLOW}LIMITED${NC}"
        print_warning "Limited IPv6 connectivity (this may be normal)"
    fi
}

# Function to test proxy settings
test_proxy_settings() {
    print_header "Testing Proxy Settings"
    
    # Check environment variables
    echo "HTTP_PROXY: ${HTTP_PROXY:-not set}"
    echo "HTTPS_PROXY: ${HTTPS_PROXY:-not set}"
    echo "NO_PROXY: ${NO_PROXY:-not set}"
    echo "http_proxy: ${http_proxy:-not set}"
    echo "https_proxy: ${https_proxy:-not set}"
    echo "no_proxy: ${no_proxy:-not set}"
    
    # Check if a proxy is in use
    if [ -n "$HTTP_PROXY" ] || [ -n "$HTTPS_PROXY" ] || [ -n "$http_proxy" ] || [ -n "$https_proxy" ]; then
        print_info "Proxy is configured"
        
        # Test proxy connectivity
        echo -n "Testing proxy connectivity... "
        if curl -s -o /dev/null -w "%{http_code}" https://www.google.com | grep -q "^[23]"; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}FAILED${NC}"
            print_error "Could not connect through proxy"
        fi
    else
        print_info "No proxy is configured"
    fi
}

# Function to test Docker network configuration
test_docker_network() {
    print_header "Testing Docker Network Configuration"
    
    if ! command_exists docker; then
        print_error "Docker is not installed"
        return 1
    fi
    
    # Check Docker network driver
    echo "Docker network drivers:"
    docker network ls --format "{{.Name}}: {{.Driver}}"
    
    # Check Docker DNS configuration
    echo -n "Docker DNS configuration... "
    if docker info 2>/dev/null | grep -q "DNS"; then
        echo -e "${GREEN}CONFIGURED${NC}"
        docker info 2>/dev/null | grep "DNS" -A3
    else
        echo -e "${YELLOW}DEFAULT${NC}"
        print_warning "Using default Docker DNS configuration"
    fi
    
    # Test network from inside a container
    echo -n "Testing network from inside a container... "
    if docker run --rm alpine ping -c 3 8.8.8.8 > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "Could not connect to the internet from inside a container"
    fi
}

# Function to test Cargo configuration
test_cargo_config() {
    print_header "Testing Cargo Configuration"
    
    if ! command_exists cargo; then
        print_error "Cargo is not installed"
        return 1
    fi
    
    # Check if .cargo/config.toml exists
    if [ -f ~/.cargo/config.toml ]; then
        echo "Cargo config file found: ~/.cargo/config.toml"
        echo "Contents:"
        cat ~/.cargo/config.toml
    elif [ -f ~/.cargo/config ]; then
        echo "Cargo config file found: ~/.cargo/config"
        echo "Contents:"
        cat ~/.cargo/config
    else
        print_warning "No Cargo config file found"
    fi
    
    # Check registry source
    echo -n "Cargo registry source... "
    if cargo --version > /dev/null 2>&1; then
        echo -e "${GREEN}DEFAULT${NC}"
    else
        echo -e "${RED}ERROR${NC}"
        print_error "Could not determine Cargo registry source"
    fi
}

# Function to generate a summary report
generate_summary() {
    print_header "Network Diagnostics Summary"
    
    # Check overall connectivity
    echo -n "Overall internet connectivity: "
    if ping -c 3 8.8.8.8 > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "No internet connectivity"
    fi
    
    # Check DNS resolution
    echo -n "DNS resolution: "
    if host google.com > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "DNS resolution is not working"
    fi
    
    # Check HTTP connectivity
    echo -n "HTTP connectivity: "
    if curl -s -o /dev/null -w "%{http_code}" https://www.google.com | grep -q "^[23]"; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "HTTP connectivity is not working"
    fi
    
    # Check Docker
    echo -n "Docker functionality: "
    if command_exists docker && docker info > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "Docker is not working properly"
    fi
    
    # Check Cargo
    echo -n "Cargo functionality: "
    if command_exists cargo && cargo --version > /dev/null 2>&1; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        print_error "Cargo is not working properly"
    fi
    
    print_info "For detailed information, see the test results above."
    print_info "If you're experiencing network issues, consider the following:"
    print_info "1. Check your DNS configuration"
    print_info "2. Check your proxy settings"
    print_info "3. Check your firewall settings"
    print_info "4. Try using alternative DNS servers (1.1.1.1, 8.8.8.8)"
    print_info "5. Try using a VPN"
    print_info "6. Check if your ISP is blocking certain domains"
}

# Main function
main() {
    print_header "Starting Network Diagnostics"
    
    # Check required commands
    for cmd in curl host ping; do
        if ! command_exists "$cmd"; then
            print_error "$cmd is required but not installed"
            exit 1
        fi
    done
    
    # Run tests
    test_dns_resolution
    test_http_connectivity
    test_network_latency
    test_ipv6_connectivity
    test_proxy_settings
    
    # Run Docker-specific tests if Docker is installed
    if command_exists docker; then
        test_docker_registry
        test_docker_network
    else
        print_warning "Docker is not installed, skipping Docker-specific tests"
    fi
    
    # Run Cargo-specific tests if Cargo is installed
    if command_exists cargo; then
        test_cargo_registry
        test_cargo_config
    else
        print_warning "Cargo is not installed, skipping Cargo-specific tests"
    fi
    
    # Generate summary
    generate_summary
}

# Run the main function
main