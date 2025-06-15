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

3. Start the local development environment:
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
   - Blog Client: http://localhost:3002/static/blog-client.html
   - Debug Tool: http://localhost:3002/static/blog-debug.html

### Managing the Local Environment

The `deploy-local.sh` script provides several commands for managing the local environment:

- **Start the environment**:
  ```bash
  ./deploy-local.sh start
  ```

- **Stop the environment**:
  ```bash
  ./deploy-local.sh stop
  ```

- **Restart the environment**:
  ```bash
  ./deploy-local.sh restart
  ```

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

### Getting Help

If you encounter issues not covered in this guide, please:

1. Check the logs with `docker-compose logs blog-api`
2. Open an issue on the GitHub repository with detailed information about the problem
3. Include the relevant logs and error messages in your issue
