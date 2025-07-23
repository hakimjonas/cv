# Docker Files Cleanup

This document describes the cleanup and consolidation of Docker files in the project.

## Changes Made

1. **Moved Docker Files to `docker/` Directory**
   - All Docker-related files were moved from the root directory to the `docker/` directory for better organization.

2. **Consolidated Docker Files**
   - Removed unnecessary duplicate and "improved" files.
   - Kept only the minimal set of files needed for different environments.
   - Updated file references in docker-compose files.

3. **Improved Files**
   - Replaced original files with their improved versions where appropriate.
   - Enhanced `.dockerignore` with more comprehensive patterns and better organization.
   - Updated `Dockerfile.local` with better security practices.

## Current Docker Files

The project now has the following Docker files:

### Production Files
- `docker/Dockerfile`: Main Dockerfile for production builds
- `docker/docker-compose.yml`: Docker Compose configuration for simple production deployment
- `docker/docker-compose.prod.yml`: Docker Compose configuration for production with nginx reverse proxy

### Development Files
- `docker/Dockerfile.local`: Dockerfile for local development with hot reloading
- `docker/docker-compose.local.yml`: Docker Compose configuration for local development

### Support Files
- `docker/.dockerignore`: Comprehensive list of files to exclude from Docker builds

## Usage

### Local Development

```bash
docker-compose -f docker/docker-compose.local.yml up -d
```

### Simple Production Deployment

```bash
docker-compose -f docker/docker-compose.yml up -d
```

### Production Deployment with Nginx

```bash
docker-compose -f docker/docker-compose.prod.yml up -d
```

## File Descriptions

### Dockerfile
- Multi-stage build for production
- Uses Rust 1.88.0
- Includes security best practices (non-root user, minimal dependencies)
- Optimized for smaller image size and faster builds

### Dockerfile.local
- Development-focused Dockerfile
- Includes hot reloading support
- Mounts source code as volumes
- Includes debugging tools and verbose logging

### docker-compose.yml
- Basic production setup
- Exposes port 3000
- Includes resource limits and health checks
- Uses persistent volume for data

### docker-compose.prod.yml
- Enhanced production setup
- Includes nginx reverse proxy for HTTPS
- Maps port 80 to container port 3000
- Includes SSL configuration
- Higher resource limits for production workloads

### docker-compose.local.yml
- Development environment setup
- Maps port 3002 to container port 3000
- Mounts source code for hot reloading
- Includes debugging configuration
- Higher timeout values for development

### .dockerignore
- Comprehensive list of files to exclude from Docker builds
- Organized by category with comments
- Includes exceptions for critical files
- Optimized for faster builds and smaller context size