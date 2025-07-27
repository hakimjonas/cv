# Docker Network Connectivity Fix

This document explains the changes made to fix network connectivity issues during the Docker build process.

## Issue Description

When running `./scripts/deploy-local.sh rebuild`, the Docker build was failing with the following error:

```
Err:1 http://deb.debian.org/debian bullseye InRelease
Temporary failure resolving 'deb.debian.org'
```

This indicates a DNS resolution problem where the container couldn't reach the Debian package repositories during the build process.

The specific error was:

```
/bin/sh: 1: cannot create /etc/resolv.conf: Read-only file system
```

This occurred because the Dockerfile was trying to modify `/etc/resolv.conf` directly, which is mounted as read-only in Docker containers.

## Changes Made

### 1. Modified Dockerfile.local

Removed the direct modification of `/etc/resolv.conf` while keeping the retry logic for apt-get commands:

```dockerfile
# Before:
RUN echo "nameserver 8.8.8.8\nnameserver 8.8.4.4" > /etc/resolv.conf && \
    # Add retry logic for apt-get
    for i in $(seq 1 3); do \
        echo "Attempt $i: Running apt-get update..." && \
        (apt-get update && break || sleep 15) \
    done && \
    ...

# After:
RUN for i in $(seq 1 3); do \
        echo "Attempt $i: Running apt-get update..." && \
        (apt-get update && break || sleep 15) \
    done && \
    ...
```

### 2. Verified docker-compose.local.yml Configuration

Confirmed that the docker-compose.local.yml file already had proper DNS configuration:

```yaml
# Add DNS configuration for better network resilience
dns:
  - 8.8.8.8
  - 8.8.4.4
# Add extra hosts for better hostname resolution
extra_hosts:
  - "deb.debian.org:151.101.0.204"
  - "security.debian.org:151.101.0.204"
```

### 3. Created rebuild-docker.sh Script

Created a script to rebuild the Docker container with improved network resilience:

```bash
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
```

## How to Use

To rebuild the Docker container with the improved network resilience:

```bash
./scripts/rebuild-docker.sh
```

This will:
1. Stop any running containers
2. Clean up existing containers and images
3. Rebuild the Docker images with the network resilience improvements
4. Start the containers
5. Check the status of the containers

## Explanation

The issue was caused by trying to modify `/etc/resolv.conf` directly in the Dockerfile, which is not possible because this file is mounted from the host and is read-only in Docker containers.

The proper way to configure DNS in Docker is through the `docker-compose.yml` file using the `dns` and `extra_hosts` options, which was already correctly set up in the project.

By removing the direct modification of `/etc/resolv.conf` from the Dockerfile and relying on the DNS configuration in docker-compose.local.yml, we've fixed the network connectivity issues during the Docker build process.

## Troubleshooting

If you still encounter network connectivity issues:

1. Check your internet connection and DNS settings
2. Try using a different network (e.g., switch from Wi-Fi to wired connection)
3. Consider using a VPN to bypass any network restrictions
4. Update the IP addresses in the `extra_hosts` configuration if the Debian mirror IPs have changed

## Future Improvements

For even better network resilience, consider:

1. Adding more Debian mirror IPs to the `extra_hosts` configuration
2. Implementing a more sophisticated retry mechanism with exponential backoff
3. Using a local Debian mirror or apt-cacher-ng to cache packages
4. Pre-downloading required packages and including them in the Docker image