# User Documentation

## Table of Contents
1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [API Documentation](#api-documentation)
4. [Common Workflows](#common-workflows)
5. [Troubleshooting](#troubleshooting)
6. [FAQ](#faq)

## Introduction

Welcome to the CV and Blog application! This documentation provides comprehensive instructions on how to use the system, including common workflows and troubleshooting tips.

The application consists of two main components:
1. **CV Generator** - Creates a CV in HTML and PDF formats
2. **Blog API** - Provides a RESTful API for managing blog posts and tags

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:
- Rust (1.70 or later)
- SQLite
- Docker (optional, for containerized deployment)

### Installation

#### Local Development

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/cv.git
   cd cv
   ```

2. Build the application:
   ```bash
   cargo build
   ```

3. Generate the CV:
   ```bash
   cargo run
   ```

4. Start the Blog API server:
   ```bash
   cargo run --bin blog_api_server
   ```

#### Docker Deployment

1. Build the Docker image:
   ```bash
   docker build -t cv-app -f docker/Dockerfile .
   ```

2. Run the container:
   ```bash
   docker run -p 3000:3000 cv-app
   ```

Alternatively, you can use Docker Compose for a more complete setup:

```bash
cd docker
docker-compose up -d
```

For local development with hot reloading, use:

```bash
./scripts/deploy-local.sh start
```

## API Documentation

The Blog API provides a RESTful interface for managing blog posts and tags. The API documentation is available through Swagger UI when the server is running.

### Accessing API Documentation

1. Start the Blog API server:
   ```bash
   cargo run --bin blog_api_server
   ```

2. Open your browser and navigate to:
   ```
   http://localhost:3000/api-docs  # For production deployment
   ```
   
   Or if you're using the local development environment:
   ```
   http://localhost:3002/api-docs  # For local development
   ```

3. The Swagger UI provides interactive documentation where you can:
   - View all available endpoints
   - See request and response schemas
   - Try out API calls directly from the browser

![Swagger UI Screenshot](docs/images/swagger-ui.png)

## Common Workflows

### Creating a Blog Post

1. **Using the API directly**:
   
   Send a POST request to `/api/posts` with the following JSON payload:
   ```json
   {
     "title": "My First Blog Post",
     "content": "This is the content of my first blog post.",
     "excerpt": "A short summary of the post.",
     "author": "Your Name",
     "published": true,
     "tags": [
       {
         "name": "Technology",
         "slug": "technology"
       }
     ]
   }
   ```

   Example using curl:
   ```bash
   # For production deployment
   curl -X POST http://localhost:3000/api/posts \
     -H "Content-Type: application/json" \
     -d '{"title":"My First Blog Post","content":"This is the content of my first blog post.","excerpt":"A short summary of the post.","author":"Your Name","published":true,"tags":[{"name":"Technology","slug":"technology"}]}'
   
   # For local development
   curl -X POST http://localhost:3002/api/posts \
     -H "Content-Type: application/json" \
     -d '{"title":"My First Blog Post","content":"This is the content of my first blog post.","excerpt":"A short summary of the post.","author":"Your Name","published":true,"tags":[{"name":"Technology","slug":"technology"}]}'
   ```

2. **Using the Swagger UI**:
   
   a. Navigate to http://localhost:3000/api-docs (production) or http://localhost:3002/api-docs (local development)
   b. Find the POST /api/posts endpoint
   c. Click "Try it out"
   d. Enter the JSON payload in the request body
   e. Click "Execute"

### Retrieving Blog Posts

1. **Get all posts**:
   
   Send a GET request to `/api/posts`
   
   Example using curl:
   ```bash
   # For production deployment
   curl http://localhost:3000/api/posts
   
   # For local development
   curl http://localhost:3002/api/posts
   ```

2. **Get a specific post by slug**:
   
   Send a GET request to `/api/posts/{slug}`
   
   Example using curl:
   ```bash
   # For production deployment
   curl http://localhost:3000/api/posts/my-first-blog-post
   
   # For local development
   curl http://localhost:3002/api/posts/my-first-blog-post
   ```

3. **Get posts by tag**:
   
   Send a GET request to `/api/tags/{tag_slug}/posts`
   
   Example using curl:
   ```bash
   # For production deployment
   curl http://localhost:3000/api/tags/technology/posts
   
   # For local development
   curl http://localhost:3002/api/tags/technology/posts
   ```

### Updating a Blog Post

1. Send a PUT request to `/api/posts/{slug}` with the updated JSON payload
   
   Example using curl:
   ```bash
   # For production deployment
   curl -X PUT http://localhost:3000/api/posts/my-first-blog-post \
     -H "Content-Type: application/json" \
     -d '{"title":"Updated Blog Post","content":"This is the updated content.","excerpt":"An updated summary.","author":"Your Name","published":true,"tags":[{"name":"Technology","slug":"technology"}]}'
   
   # For local development
   curl -X PUT http://localhost:3002/api/posts/my-first-blog-post \
     -H "Content-Type: application/json" \
     -d '{"title":"Updated Blog Post","content":"This is the updated content.","excerpt":"An updated summary.","author":"Your Name","published":true,"tags":[{"name":"Technology","slug":"technology"}]}'
   ```

### Deleting a Blog Post

1. Send a DELETE request to `/api/posts/{slug}`
   
   Example using curl:
   ```bash
   # For production deployment
   curl -X DELETE http://localhost:3000/api/posts/my-first-blog-post
   
   # For local development
   curl -X DELETE http://localhost:3002/api/posts/my-first-blog-post
   ```

## Troubleshooting

### Common Issues

#### Server Won't Start

**Problem**: The server fails to start with a "port already in use" error.

**Solution**: The application will automatically try to find an available port. If you want to specify a port, you can modify the `api_port` setting in the configuration.

#### Database Errors

**Problem**: You encounter database-related errors.

**Solution**: 
1. Check that SQLite is properly installed
2. Ensure the database file has the correct permissions
3. Try initializing a new database:
   ```bash
   cargo run --bin blog_api_server -- --init-db
   ```

#### API Request Failures

**Problem**: API requests return errors or unexpected results.

**Solution**:
1. Check the request format against the API documentation
2. Ensure all required fields are provided
3. Verify that the content types and headers are correct
4. Check the server logs for more detailed error information

## FAQ

### General Questions

**Q: How do I customize the CV template?**

A: The CV template is defined in the `templates` directory. You can modify the HTML templates to customize the appearance of the CV.

**Q: Can I use Markdown in blog posts?**

A: Yes, the blog post content field supports Markdown formatting. The content will be rendered as HTML when displayed.

**Q: How do I back up the database?**

A: You can use the provided backup script:
```bash
./scripts/backup_database.sh --database /path/to/your/database.db
```

For more detailed information about database backup and restore procedures, including automated backups and Docker integration, see the [BACKUP_RESTORE.md](BACKUP_RESTORE.md) documentation.

**Q: How do I restore the database from a backup?**

A: You can use the provided restore script:
```bash
./scripts/restore_database.sh --backup /path/to/backup.sql.gz --database /path/to/your/database.db
```

**Q: How do I set up automated backups?**

A: You can use the provided script to set up cron jobs for daily, weekly, and monthly backups:
```bash
./scripts/setup_backup_cron.sh --database /path/to/your/database.db
```

**Q: How do I deploy to production?**

A: For production deployment, we recommend using the Docker container with the provided deployment scripts. See the [DEPLOYMENT_CONSOLIDATED.md](DEPLOYMENT_CONSOLIDATED.md) file for detailed instructions.

### API Questions

**Q: What is the rate limit for API requests?**

A: The default rate limit is 100 requests per minute per IP address. This can be configured in the server settings.

**Q: How do I authenticate API requests?**

A: Currently, the API does not require authentication. Authentication will be added in a future update.

**Q: Can I get the API responses in a different format than JSON?**

A: Currently, the API only supports JSON responses. Support for other formats may be added in the future.

**Q: How do I report a bug or request a feature?**

A: Please open an issue on the GitHub repository with a detailed description of the bug or feature request.