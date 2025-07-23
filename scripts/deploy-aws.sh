#!/bin/bash

set -e

echo "Starting AWS deployment for CV Blog Application"

# Generate CV and website files
echo "Generating CV and website files..."
if ! command -v cargo &> /dev/null; then
    echo "Cargo is not installed. Please install Rust and Cargo before proceeding."
    exit 1
fi

cargo run --bin cv
echo "CV and website files generated successfully in dist/ directory"

# Configuration
AWS_REGION="${AWS_REGION:-us-east-1}"
ECR_REPOSITORY="${ECR_REPOSITORY:-cv-blog-api}"
ECS_CLUSTER="${ECS_CLUSTER:-cv-blog-cluster}"
ECS_SERVICE="${ECS_SERVICE:-cv-blog-service}"
TASK_DEFINITION="${TASK_DEFINITION:-cv-blog-task}"
IMAGE_TAG="${IMAGE_TAG:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if AWS CLI is installed
if ! command -v aws &> /dev/null; then
    print_error "AWS CLI is not installed. Please install it before proceeding."
    exit 1
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker before proceeding."
    exit 1
fi

# Function to check AWS credentials
check_aws_credentials() {
    print_status "Checking AWS credentials..."
    if ! aws sts get-caller-identity &> /dev/null; then
        print_error "AWS credentials not configured. Please run 'aws configure' first."
        exit 1
    fi
    print_status "AWS credentials verified."
}

# Function to create ECR repository if it doesn't exist
create_ecr_repository() {
    print_status "Checking ECR repository..."
    if ! aws ecr describe-repositories --repository-names $ECR_REPOSITORY --region $AWS_REGION &> /dev/null; then
        print_status "Creating ECR repository: $ECR_REPOSITORY"
        aws ecr create-repository --repository-name $ECR_REPOSITORY --region $AWS_REGION
    else
        print_status "ECR repository $ECR_REPOSITORY already exists."
    fi
}

# Function to build and push Docker image
build_and_push_image() {
    print_status "Building Docker image..."

    # Get ECR login token
    aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $(aws ecr describe-repositories --repository-names $ECR_REPOSITORY --region $AWS_REGION --query 'repositories[0].repositoryUri' --output text | cut -d'/' -f1)

    # Get repository URI
    REPOSITORY_URI=$(aws ecr describe-repositories --repository-names $ECR_REPOSITORY --region $AWS_REGION --query 'repositories[0].repositoryUri' --output text)

    # Build the image
    docker build -t $ECR_REPOSITORY:$IMAGE_TAG .

    # Tag the image
    docker tag $ECR_REPOSITORY:$IMAGE_TAG $REPOSITORY_URI:$IMAGE_TAG

    # Push the image
    print_status "Pushing image to ECR..."
    docker push $REPOSITORY_URI:$IMAGE_TAG

    echo "REPOSITORY_URI=$REPOSITORY_URI:$IMAGE_TAG"
}

# Function to create ECS task definition
create_task_definition() {
    local repository_uri=$1

    print_status "Creating ECS task definition..."

    cat > task-definition.json << EOF
{
    "family": "$TASK_DEFINITION",
    "networkMode": "awsvpc",
    "requiresCompatibilities": ["FARGATE"],
    "cpu": "512",
    "memory": "1024",
    "executionRoleArn": "arn:aws:iam::$(aws sts get-caller-identity --query Account --output text):role/ecsTaskExecutionRole",
    "containerDefinitions": [
        {
            "name": "cv-blog-api",
            "image": "$repository_uri",
            "portMappings": [
                {
                    "containerPort": 3000,
                    "protocol": "tcp"
                }
            ],
            "environment": [
                {
                    "name": "RUST_LOG",
                    "value": "info"
                },
                {
                    "name": "RUST_BACKTRACE",
                    "value": "0"
                }
            ],
            "healthCheck": {
                "command": ["CMD-SHELL", "curl -f http://localhost:3000/health || exit 1"],
                "interval": 30,
                "timeout": 5,
                "retries": 3,
                "startPeriod": 60
            },
            "logConfiguration": {
                "logDriver": "awslogs",
                "options": {
                    "awslogs-group": "/ecs/$TASK_DEFINITION",
                    "awslogs-region": "$AWS_REGION",
                    "awslogs-stream-prefix": "ecs"
                }
            },
            "essential": true
        }
    ]
}
EOF

    # Create CloudWatch log group
    aws logs create-log-group --log-group-name "/ecs/$TASK_DEFINITION" --region $AWS_REGION 2>/dev/null || true

    # Register task definition
    aws ecs register-task-definition --cli-input-json file://task-definition.json --region $AWS_REGION

    # Clean up
    rm task-definition.json
}

# Function to create or update ECS service
deploy_service() {
    print_status "Deploying ECS service..."

    # Check if cluster exists
    if ! aws ecs describe-clusters --clusters $ECS_CLUSTER --region $AWS_REGION &> /dev/null; then
        print_status "Creating ECS cluster: $ECS_CLUSTER"
        aws ecs create-cluster --cluster-name $ECS_CLUSTER --region $AWS_REGION
    fi

    # Check if service exists
    if aws ecs describe-services --cluster $ECS_CLUSTER --services $ECS_SERVICE --region $AWS_REGION &> /dev/null; then
        print_status "Updating existing service..."
        aws ecs update-service --cluster $ECS_CLUSTER --service $ECS_SERVICE --task-definition $TASK_DEFINITION --region $AWS_REGION
    else
        print_status "Creating new service..."
        print_warning "Note: You'll need to configure VPC, subnets, and security groups manually or use AWS Console/CloudFormation"
        print_warning "This script creates the basic service. For production, consider using AWS CDK or CloudFormation."

        # This is a basic service creation - in production you'd want to specify VPC configuration
        aws ecs create-service \
            --cluster $ECS_CLUSTER \
            --service-name $ECS_SERVICE \
            --task-definition $TASK_DEFINITION \
            --desired-count 1 \
            --launch-type FARGATE \
            --region $AWS_REGION
    fi
}

# Main deployment flow
main() {
    print_status "Starting deployment process..."

    check_aws_credentials
    create_ecr_repository

    # Build and push image
    REPOSITORY_URI=$(build_and_push_image | grep "REPOSITORY_URI=" | cut -d'=' -f2)

    create_task_definition $REPOSITORY_URI
    deploy_service

    print_status "Deployment completed successfully!"
    print_status "Your application should be available on AWS ECS."
    print_warning "Don't forget to configure:"
    print_warning "  - VPC and subnet configuration"
    print_warning "  - Security groups (allow port 3000)"
    print_warning "  - Load balancer (for production traffic)"
    print_warning "  - Domain name and SSL certificate"
}

# Parse command line arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "build-only")
        check_aws_credentials
        create_ecr_repository
        build_and_push_image
        ;;
    "help")
        echo "Usage: $0 [deploy|build-only|help]"
        echo "  deploy     - Full deployment (default)"
        echo "  build-only - Only build and push Docker image"
        echo "  help       - Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for usage information."
        exit 1
        ;;
esac
