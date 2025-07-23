# Comprehensive Deployment Guide

This document provides detailed instructions for deploying the CV Blog application in various environments, from local development to production.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Local Development and Testing](#local-development-and-testing)
3. [Production Deployment](#production-deployment)
   - [AWS Deployment](#aws-deployment)
   - [Docker Swarm Deployment](#docker-swarm-deployment)
   - [Kubernetes Deployment](#kubernetes-deployment)
4. [CI/CD Pipeline](#cicd-pipeline)
5. [Deployment Configuration](#deployment-configuration)
6. [Monitoring and Health Checks](#monitoring-and-health-checks)
7. [Backup and Recovery](#backup-and-recovery)
8. [Security Considerations](#security-considerations)
9. [Cost Optimization](#cost-optimization)
10. [Troubleshooting](#troubleshooting)
11. [Recent Updates](#recent-updates)

## Prerequisites

### Local Development
- Docker and Docker Compose
- Rust toolchain (for local development)
- Git repository cloned locally

### Production Deployment
- Docker and Docker Compose
- Server with SSH access (for manual deployment)
- AWS account with appropriate permissions (for AWS deployment)
  - ECR (Elastic Container Registry)
  - ECS (Elastic Container Service)
  - CloudWatch Logs
  - IAM (for task execution roles)

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

5. Access the application:
   - Main Website: http://localhost:3002
   - Blog: http://localhost:3002/blog.html
   - CV: http://localhost:3002/cv.html
   - Projects: http://localhost:3002/projects.html
   - Blog API: http://localhost:3002/api/blog
   - API Admin: http://localhost:3002/admin
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

- **Rebuild the application (preserves data)**:
  ```bash
  ./deploy-local.sh rebuild
  ```
  This command stops the current containers, rebuilds the images without using cache, and restarts the application. Data in volumes is preserved.

- **Remove unused Docker resources**:
  ```bash
  ./deploy-local.sh prune
  ```
  This command removes all unused containers, networks, images, and volumes. It will prompt for confirmation before proceeding.

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

### How Local Deployment Validates Production Readiness

The local development environment is designed to closely simulate the production environment, making it an effective way to validate that your changes will work in production:

1. **Similar Docker-based Architecture**: Both environments use Docker and Docker Compose, ensuring that container-related issues are caught early.
2. **Same Application Code**: The local environment runs the same application code that will be deployed to production.
3. **Identical API Endpoints**: All API endpoints available in production are also available in the local environment.
4. **Similar Frontend Integration**: The frontend integration works the same way in both environments, just on different ports (3002 for local, 3000 for production).
5. **Health Checks**: The local environment includes health checks similar to those used in production, helping validate that your service will pass production health checks.

If your application works correctly in the local environment, it's a strong indication that it will work in production. Any differences between the environments are primarily related to development convenience (e.g., hot reloading, debug logging) and don't affect the core functionality.

## Production Deployment

The CV Blog application can be deployed to production using several methods:

### AWS Deployment

#### Quick Start

```bash
# Set environment variables (optional)
export AWS_REGION=us-east-1
export ECR_REPOSITORY=cv-blog-api
export ECS_CLUSTER=cv-blog-cluster

# Deploy to AWS
./deploy-aws.sh
```

#### Manual Configuration Steps

After running the deployment script, you'll need to configure:

1. **VPC and Networking**
   - Create or use existing VPC
   - Configure subnets (public for load balancer, private for containers)
   - Set up security groups

2. **Load Balancer** (recommended for production)
   - Create Application Load Balancer
   - Configure target group pointing to ECS service
   - Set up health checks on `/health` endpoint

3. **Domain and SSL**
   - Configure Route 53 or your DNS provider
   - Set up SSL certificate via AWS Certificate Manager
   - Configure HTTPS redirect

#### Environment Variables

The following environment variables can be configured:

| Variable | Default | Description |
|----------|---------|-------------|
| `AWS_REGION` | `us-east-1` | AWS region for deployment |
| `ECR_REPOSITORY` | `cv-blog-api` | ECR repository name |
| `ECS_CLUSTER` | `cv-blog-cluster` | ECS cluster name |
| `ECS_SERVICE` | `cv-blog-service` | ECS service name |
| `TASK_DEFINITION` | `cv-blog-task` | ECS task definition name |
| `IMAGE_TAG` | `latest` | Docker image tag |

### Docker Swarm Deployment

```bash
# Initialize swarm (if not already done)
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.prod.yml cv-blog
```

### Kubernetes Deployment

For Kubernetes deployment, you'll need to create appropriate manifests. The Docker images built by this project can be used in any Kubernetes cluster.

### Manual Deployment

#### CV Site

To manually deploy the CV site:

1. Build the site:
   ```bash
   cargo run --release
   ```

2. The generated files will be in the `dist/` directory. You can deploy these files to any static web hosting service.

#### Blog API

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

## Deployment Configuration

### Production Environment Variables

Set these in your production environment:

```bash
# Logging
RUST_LOG=info
RUST_BACKTRACE=0

# Application-specific (if needed)
# Add any application-specific environment variables here
```

### Health Checks

The application provides a health check endpoint at `/health` that returns:

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Docker Configuration

The Docker configuration includes:

- Resource limits to prevent resource exhaustion
- Health checks to ensure the service is healthy
- Logging with rotation to prevent disk space issues
- Isolated network for better security

### Zero-Downtime Deployment

The deployment script implements zero-downtime deployment using Docker's rolling update feature:
1. Builds a new image
2. Updates the service without stopping the existing one
3. Checks the health status of the new container
4. Rolls back if the health check fails

## Monitoring and Logging

### CloudWatch Logs
- Logs are automatically sent to CloudWatch when using ECS
- Log group: `/ecs/cv-blog-task`
- Retention can be configured in AWS Console

### Metrics
Consider setting up:
- CloudWatch Container Insights for ECS metrics
- Application-level metrics (if implemented)
- Custom dashboards for monitoring

## Backup and Recovery

### Data Backup
- ECS persistent volumes are backed up automatically
- Consider additional backup strategies for critical data

### Disaster Recovery
- Multi-AZ deployment for high availability
- Cross-region replication for disaster recovery
- Infrastructure as Code for quick recovery

## Security Considerations

### Container Security
- Images are built from official Rust base images
- Non-root user execution (configured in Dockerfile)
- Minimal attack surface with multi-stage builds

### Network Security
- Use private subnets for containers
- Configure security groups to allow only necessary traffic
- Use Application Load Balancer for SSL termination

### Data Security
- Blog data is stored in persistent volumes
- Consider encryption at rest for sensitive data
- Regular backups of persistent volumes

## Cost Optimization

### ECS Fargate Costs
- Use appropriate CPU/memory allocation
- Consider Spot instances for non-critical workloads
- Monitor usage with AWS Cost Explorer

### Storage Costs
- Regular cleanup of old logs
- Optimize image sizes
- Use lifecycle policies for ECR

## Troubleshooting

### Common Issues

1. **Container fails to start**
   - Check the logs with `docker-compose logs blog-api` or `./deploy-local.sh logs`
   - Verify Docker is running: `docker info`
   - Check for port conflicts: `netstat -tuln | grep 3002`
   - Ensure the health check endpoint is accessible

2. **Application is not accessible**
   - Check container status: `./deploy-local.sh status`
   - Verify the container is healthy: `docker inspect --format='{{.State.Health.Status}}' $(docker compose -f docker-compose.local.yml ps -q blog-api)`
   - Check application logs: `./deploy-local.sh logs`

3. **CI/CD pipeline fails with SSH error**
   - Ensure the SSH_PRIVATE_KEY secret is correctly configured
   - Check if the server is accessible from GitHub Actions
   - Verify that the SERVER_IP and SERVER_USER secrets are correct

4. **Docker Compose fails to start the service**
   - Check if Docker and Docker Compose are installed
   - Ensure the ports are not already in use
   - Check if there are permission issues with the volumes

5. **Rebuild fails**
   - Try pruning unused resources: `./deploy-local.sh prune`
   - Check disk space: `df -h`
   - Verify Docker daemon is running correctly: `docker info`

### Debugging Commands

```bash
# Check ECS service status
aws ecs describe-services --cluster cv-blog-cluster --services cv-blog-service

# View task logs
aws logs tail /ecs/cv-blog-task --follow

# Check container health
docker exec -it <container_id> curl http://localhost:3000/health
```

## Recent Updates

### July 2025 Updates

The deployment scripts have been updated with the following improvements:

#### Modernization
- Added support for Docker Compose V2 (`docker compose`) with fallback to V1 (`docker-compose`)
- Updated command syntax to use the appropriate Docker Compose command based on the installed version

#### User Experience Improvements
- Added colored output for better readability
- Improved error messages and status information
- Enhanced progress reporting during startup
- Optimized waiting times for health checks

#### New Features
- Added `rebuild` command to rebuild the application from scratch while preserving data
- Added `prune` command to clean up unused Docker resources

#### Error Handling
- Improved error detection and reporting
- Added more descriptive error messages
- Enhanced cleanup on failure

#### Docker Configuration Updates
- Added version specification to docker-compose.local.yml
- Added resource limits to prevent container from using too much CPU/memory
- Added logging configuration to limit log file size
- Added volume driver specification for better compatibility
- Ensured consistency between Dockerfile and Dockerfile.local by using the same Rust version