# Deployment Guide

This document provides detailed instructions for deploying the application, both locally for testing and to production environments, either manually or using the CI/CD pipeline.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development and Testing](#local-development-and-testing)
- [CI/CD Pipeline](#cicd-pipeline)
- [Manual Deployment](#manual-deployment)
- [Deployment Configuration](#deployment-configuration)
- [Monitoring and Health Checks](#monitoring-and-health-checks)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before deploying the application, ensure you have the following:

- For local development and testing:
  - Docker and Docker Compose installed
  - Git repository cloned locally

- For CI/CD deployment:
  - GitHub repository with the code
  - Server with SSH access
  - Docker and Docker Compose installed on the server
  - GitHub repository secrets configured (see [CI/CD Pipeline](#cicd-pipeline))

- For manual deployment:
  - Docker and Docker Compose installed
  - Git repository cloned locally

## Local Development and Testing

Before deploying to production, it's recommended to test your changes in a local development environment. This project includes a dedicated setup for local testing that provides:

- Hot reloading for faster development
- Debug-level logging for better troubleshooting
- Easy-to-use commands for managing the environment

### Setting Up Local Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/personal-website.git
   cd personal-website
   ```

2. Make the local deployment script executable (if it's not already):
   ```bash
   chmod +x deploy-local.sh
   ```

3. Ensure the dist directory exists for generated files:
   ```bash
   mkdir -p dist
   ```

   This ensures that the application has a place to store generated files.

4. Start the local development environment:
   ```bash
   ./deploy-local.sh start
   ```

   This will:
   - Build the Docker image for local development
   - Start the containers
   - Check if the service is running properly
   - Display URLs for accessing the application

4. Access the application:
   - Blog API: http://localhost:3002
   - Main Frontend: http://localhost:3002/
   - Blog Client: http://localhost:3002/static/blog-client.html
   - Debug Tool: http://localhost:3002/static/blog-debug.html

   > **Note**: The first time you start the local environment, it may take several minutes for the blog-api service to compile and start up. The script will wait for the service to be ready and provide appropriate feedback. If the service takes longer than expected to start, the script will provide instructions on how to check the status and view the logs.
   >
   > The compilation process can take up to 10 minutes on the first run, depending on your system's performance and internet connection. Subsequent starts will be faster as the compiled artifacts are cached.

### Managing the Local Environment

The `deploy-local.sh` script provides several commands for managing the local environment:

- **Start the environment**:
  ```bash
  ./deploy-local.sh start
  ```
  This command builds and starts the containers, automatically removing any orphaned containers.

- **Stop the environment**:
  ```bash
  ./deploy-local.sh stop
  ```
  This command stops and removes the containers, including any orphaned containers.

- **Restart the environment**:
  ```bash
  ./deploy-local.sh restart
  ```
  This command stops and then starts the environment, cleaning up orphaned containers.

- **View logs**:
  ```bash
  ./deploy-local.sh logs
  ```

- **Check status**:
  ```bash
  ./deploy-local.sh status
  ```

- **Show help**:
  ```bash
  ./deploy-local.sh help
  ```

> **Note**: The script automatically handles orphaned containers (containers that were part of the project but are no longer defined in the docker-compose file) by removing them during start and stop operations.

### Development Workflow

1. Start the local environment with `./deploy-local.sh start`
2. Make changes to the code
3. The application will automatically reload with your changes (thanks to cargo-watch)
4. Test your changes in the browser or using API tools
5. View logs with `./deploy-local.sh logs` if you encounter issues
6. When finished, stop the environment with `./deploy-local.sh stop`

### Testing Before Production Deployment

Before deploying to production, it's recommended to:

1. Test all features in the local environment
2. Check for any errors in the logs
3. Verify that the application works as expected with different browsers and devices
4. Run the automated tests:
   ```bash
   cargo test
   ```

### How Local Deployment Validates Production Readiness

The local development environment is designed to closely simulate the production environment, making it an effective way to validate that your changes will work in production:

1. **Similar Docker-based Architecture**: Both environments use Docker and Docker Compose, ensuring that container-related issues are caught early.
2. **Same Application Code**: The local environment runs the same application code that will be deployed to production.
3. **Identical API Endpoints**: All API endpoints available in production are also available in the local environment.
4. **Similar Frontend Integration**: The frontend integration works the same way in both environments, just on different ports (3002 for local, 3000 for production).
5. **Health Checks**: The local environment includes health checks similar to those used in production, helping validate that your service will pass production health checks.

If your application works correctly in the local environment, it's a strong indication that it will work in production. Any differences between the environments are primarily related to development convenience (e.g., hot reloading, debug logging) and don't affect the core functionality.

## CI/CD Pipeline

The project uses GitHub Actions for CI/CD. There are three workflows:

1. **Rust CI** (`ci.yml`): Runs tests, linting, and formatting checks on every push and pull request.
2. **Deploy CV** (`deploy.yml`): Deploys the CV site to GitHub Pages on every push to the main branch.
3. **Deploy Blog API** (`deploy-blog-api.yml`): Deploys the blog API to a server on every push to the main branch.

### Setting Up GitHub Secrets

For the blog API deployment to work, you need to configure the following secrets in your GitHub repository:

- `SSH_PRIVATE_KEY`: The private SSH key for accessing the server
- `SERVER_IP`: The IP address of the server
- `SERVER_USER`: The username to use when connecting to the server

### Deployment Process

The CI/CD pipeline automatically:

1. Runs tests and checks on every push and pull request
2. Deploys the CV site to GitHub Pages on every push to the main branch
3. Deploys the blog API to the server on every push to the main branch

## Manual Deployment

### CV Site

To manually deploy the CV site:

1. Build the site:
   ```bash
   cargo run --release
   ```

2. The generated files will be in the `dist/` directory. You can deploy these files to any static web hosting service.

### Blog API

To manually deploy the blog API:

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/personal-website.git
   cd personal-website
   ```

2. Run the deployment script:
   ```bash
   ./deploy.sh
   ```

The deployment script will:
- Build the Docker image
- Start the service or perform a rolling update if it's already running
- Check the health status of the container
- Provide detailed error messages if deployment fails

## Deployment Configuration

### Docker Configuration

The Docker configuration includes:

- **Resource Limits**: CPU and memory limits to prevent resource exhaustion
- **Health Checks**: Regular checks to ensure the service is healthy
- **Logging**: Configured with rotation to prevent disk space issues
- **Networking**: Isolated network for better security

### Zero-Downtime Deployment

The deployment script implements zero-downtime deployment using Docker's rolling update feature. When the service is already running, it:

1. Builds a new image
2. Updates the service without stopping the existing one
3. Checks the health status of the new container
4. Rolls back if the health check fails

## Monitoring and Health Checks

The application includes a health check endpoint at `/health` that returns a 200 status code when the service is healthy. This endpoint is used by:

- Docker's health check feature
- The deployment script to verify successful deployment
- The CI/CD pipeline to verify successful deployment

## Troubleshooting

### Common Issues

1. **Deployment fails with "Service is not healthy"**:
   - Check the logs with `docker-compose logs blog-api`
   - Ensure the health check endpoint is working correctly
   - Check if the service has enough resources

2. **CI/CD pipeline fails with SSH error**:
   - Ensure the SSH_PRIVATE_KEY secret is correctly configured
   - Check if the server is accessible from GitHub Actions
   - Verify that the SERVER_IP and SERVER_USER secrets are correct

3. **Docker Compose fails to start the service**:
   - Check if Docker and Docker Compose are installed
   - Ensure the ports are not already in use
   - Check if there are permission issues with the volumes

## Frontend Access URLs

After deployment, you can access the frontend of the application at the following URLs:

- **GitHub Pages Deployment**:
  - Main Website: https://yourusername.github.io/personal-website/
  - CV Page: https://yourusername.github.io/personal-website/cv.html
  - Projects Page: https://yourusername.github.io/personal-website/projects.html

- **Server Deployment (Blog API)**:
  - API Root: http://your-server-ip:3000/
  - API Endpoints: http://your-server-ip:3000/api/blog
  - Blog Client: http://your-server-ip:3000/static/blog-client.html
  - Debug Tool: http://your-server-ip:3000/static/blog-debug.html

> **Note**: Replace `yourusername` with your actual GitHub username and `your-server-ip` with the actual IP address or domain name of your server.

### Getting Help

If you encounter issues not covered in this guide, please:

1. Check the logs with `docker-compose logs blog-api`
2. Open an issue on the GitHub repository with detailed information about the problem
3. Include the relevant logs and error messages in your issue
