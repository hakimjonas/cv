# Deployment Guide

This document provides detailed instructions for deploying the application, both manually and using the CI/CD pipeline.

## Table of Contents

- [Prerequisites](#prerequisites)
- [CI/CD Pipeline](#cicd-pipeline)
- [Manual Deployment](#manual-deployment)
- [Deployment Configuration](#deployment-configuration)
- [Monitoring and Health Checks](#monitoring-and-health-checks)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before deploying the application, ensure you have the following:

- For CI/CD deployment:
  - GitHub repository with the code
  - Server with SSH access
  - Docker and Docker Compose installed on the server
  - GitHub repository secrets configured (see [CI/CD Pipeline](#cicd-pipeline))

- For manual deployment:
  - Docker and Docker Compose installed
  - Git repository cloned locally

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