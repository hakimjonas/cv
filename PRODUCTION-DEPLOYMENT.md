# Production Deployment Guide

This guide covers deploying the CV Blog application to production environments, with a focus on AWS deployment.

## Overview

The CV Blog application is a Rust-based web application that serves a personal website with blog functionality. It consists of:

- **Static Site Generator**: Generates HTML pages from templates
- **Blog API Server**: Provides REST API for blog management
- **Frontend**: Serves the generated website and blog interface

## Prerequisites

### Local Development
- Docker and Docker Compose
- Rust toolchain (for local development)

### AWS Deployment
- AWS CLI configured with appropriate credentials
- Docker
- AWS account with permissions for:
  - ECR (Elastic Container Registry)
  - ECS (Elastic Container Service)
  - CloudWatch Logs
  - IAM (for task execution roles)

## Deployment Options

### 1. Local Production Testing

Use the production Docker Compose configuration for local testing:

```bash
# Build and start with production configuration
docker-compose -f docker-compose.prod.yml up -d --build

# Check health
curl http://localhost/health

# View logs
docker-compose -f docker-compose.prod.yml logs -f
```

### 2. AWS ECS Deployment

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

### 3. Alternative Deployment Options

#### Docker Swarm
```bash
# Initialize swarm (if not already done)
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.prod.yml cv-blog
```

#### Kubernetes
For Kubernetes deployment, you'll need to create appropriate manifests. The Docker images built by this project can be used in any Kubernetes cluster.

## Configuration

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

### Monitoring and Logging

#### CloudWatch Logs
- Logs are automatically sent to CloudWatch when using ECS
- Log group: `/ecs/cv-blog-task`
- Retention can be configured in AWS Console

#### Metrics
Consider setting up:
- CloudWatch Container Insights for ECS metrics
- Application-level metrics (if implemented)
- Custom dashboards for monitoring

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

## Scaling

### Horizontal Scaling
```bash
# Scale ECS service
aws ecs update-service \
  --cluster cv-blog-cluster \
  --service cv-blog-service \
  --desired-count 3
```

### Vertical Scaling
Modify the task definition to allocate more CPU/memory:
- CPU: 512, 1024, 2048, 4096 (CPU units)
- Memory: 1GB, 2GB, 4GB, 8GB

## Troubleshooting

### Common Issues

1. **Container fails to start**
   - Check CloudWatch logs
   - Verify environment variables
   - Ensure health check endpoint is accessible

2. **Health check failures**
   - Verify `/health` endpoint responds correctly
   - Check container port configuration
   - Review security group rules

3. **Image build failures**
   - Ensure all required files are present
   - Check Dockerfile syntax
   - Verify base image availability

### Debugging Commands

```bash
# Check ECS service status
aws ecs describe-services --cluster cv-blog-cluster --services cv-blog-service

# View task logs
aws logs tail /ecs/cv-blog-task --follow

# Check container health
docker exec -it <container_id> curl http://localhost:3000/health
```

## Backup and Recovery

### Data Backup
- ECS persistent volumes are backed up automatically
- Consider additional backup strategies for critical data

### Disaster Recovery
- Multi-AZ deployment for high availability
- Cross-region replication for disaster recovery
- Infrastructure as Code for quick recovery

## Cost Optimization

### ECS Fargate Costs
- Use appropriate CPU/memory allocation
- Consider Spot instances for non-critical workloads
- Monitor usage with AWS Cost Explorer

### Storage Costs
- Regular cleanup of old logs
- Optimize image sizes
- Use lifecycle policies for ECR

## Maintenance

### Updates
```bash
# Update application
./deploy-aws.sh

# Update only the image
./deploy-aws.sh build-only
```

### Monitoring
- Set up CloudWatch alarms for key metrics
- Regular health checks
- Performance monitoring

## Support

For issues related to:
- **Application bugs**: Check application logs and GitHub issues
- **AWS deployment**: Review AWS documentation and CloudWatch logs
- **Infrastructure**: Consider using AWS Support or consulting services

---

**Note**: This deployment guide assumes familiarity with AWS services. For production deployments, consider working with a DevOps engineer or AWS solutions architect to ensure best practices are followed.