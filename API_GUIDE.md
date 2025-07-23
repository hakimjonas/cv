# API Guide

## Table of Contents
1. [Introduction](#introduction)
2. [API Overview](#api-overview)
3. [Authentication](#authentication)
4. [Endpoints](#endpoints)
5. [Request/Response Formats](#requestresponse-formats)
6. [Error Handling](#error-handling)
7. [Pagination](#pagination)
8. [Rate Limiting](#rate-limiting)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

## Introduction

This guide provides comprehensive documentation for developers who want to integrate with our Blog API. The API allows you to programmatically manage blog posts and tags, enabling you to build custom interfaces or integrate blog content into other applications.

## API Overview

The Blog API is a RESTful API that uses standard HTTP methods and returns JSON responses. The base URL for all API requests is:

```
http://localhost:3000/api
```

For production deployments, replace `localhost:3000` with your domain.

### Interactive Documentation

We provide interactive API documentation using Swagger UI, which allows you to explore the API and make test requests directly from your browser:

```
http://localhost:3000/api-docs
```

## Authentication

Currently, the API does not require authentication. This will be added in a future update.

## Endpoints

### Blog Posts

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/posts` | Get all blog posts |
| GET | `/posts/{slug}` | Get a specific blog post by slug |
| POST | `/posts` | Create a new blog post |
| PUT | `/posts/{slug}` | Update an existing blog post |
| DELETE | `/posts/{slug}` | Delete a blog post |
| GET | `/posts/published` | Get all published blog posts |
| GET | `/posts/featured` | Get featured blog posts |

### Tags

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/tags` | Get all tags |
| GET | `/tags/{slug}/posts` | Get all posts with a specific tag |

## Request/Response Formats

### Blog Post Object

```json
{
  "id": 1,
  "title": "My Blog Post",
  "slug": "my-blog-post",
  "date": "2025-07-23",
  "author": "Jane Doe",
  "excerpt": "A short summary of the post",
  "content": "The full content of the blog post...",
  "published": true,
  "featured": false,
  "image": "/images/blog-post.jpg",
  "tags": [
    {
      "id": 1,
      "name": "Technology",
      "slug": "technology"
    }
  ],
  "metadata": {
    "readTime": "5 min",
    "category": "Tech News"
  }
}
```

### Tag Object

```json
{
  "id": 1,
  "name": "Technology",
  "slug": "technology"
}
```

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of a request:

| Status Code | Description |
|-------------|-------------|
| 200 | OK - The request was successful |
| 201 | Created - A new resource was successfully created |
| 400 | Bad Request - The request was invalid or cannot be served |
| 404 | Not Found - The requested resource does not exist |
| 422 | Unprocessable Entity - The request was well-formed but contains semantic errors |
| 500 | Internal Server Error - Something went wrong on the server |

Error responses include a JSON object with details about the error:

```json
{
  "error": {
    "message": "Blog post not found",
    "code": "not_found"
  }
}
```

## Pagination

For endpoints that return multiple items (like `/posts`), the API supports pagination using the following query parameters:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `page` | The page number to retrieve | 1 |
| `per_page` | The number of items per page | 10 |

Example:
```
GET /api/posts?page=2&per_page=20
```

The response includes pagination metadata:

```json
{
  "data": [
    {
      "id": 1,
      "title": "Example Post 1",
      "slug": "example-post-1"
    },
    {
      "id": 2,
      "title": "Example Post 2",
      "slug": "example-post-2"
    }
  ],
  "pagination": {
    "total": 45,
    "per_page": 20,
    "current_page": 2,
    "last_page": 3,
    "next_page_url": "/api/posts?page=3&per_page=20",
    "prev_page_url": "/api/posts?page=1&per_page=20"
  }
}
```

## Rate Limiting

To ensure the stability of the service, the API implements rate limiting. By default, clients are limited to 100 requests per minute per IP address.

Rate limit information is included in the response headers:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | The maximum number of requests allowed per minute |
| `X-RateLimit-Remaining` | The number of requests remaining in the current window |
| `X-RateLimit-Reset` | The time at which the current rate limit window resets (Unix timestamp) |

If you exceed the rate limit, you'll receive a 429 Too Many Requests response.

## Best Practices

### Efficient Data Retrieval

1. **Use pagination** for endpoints that return multiple items to avoid retrieving unnecessary data.
2. **Request only what you need** by using query parameters to filter results.
3. **Cache responses** when appropriate to reduce the number of API calls.

### Error Handling

1. **Always check status codes** to ensure your request was successful.
2. **Implement proper error handling** in your application to gracefully handle API errors.
3. **Log error responses** for debugging purposes.

### Performance Optimization

1. **Minimize the number of API calls** by batching requests when possible.
2. **Use conditional requests** with the `If-Modified-Since` header to avoid retrieving unchanged data.
3. **Implement client-side caching** to reduce the load on the API server.

## Examples

### Creating a Blog Post

**Request:**
```bash
curl -X POST http://localhost:3000/api/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Getting Started with Rust",
    "excerpt": "A beginner's guide to Rust programming language",
    "content": "# Introduction\n\nRust is a systems programming language...",
    "author": "Jane Doe",
    "published": true,
    "tags": [
      {
        "name": "Rust",
        "slug": "rust"
      },
      {
        "name": "Programming",
        "slug": "programming"
      }
    ]
  }'
```

**Response:**
```json
{
  "id": 1,
  "title": "Getting Started with Rust",
  "slug": "getting-started-with-rust",
  "date": "2025-07-23",
  "author": "Jane Doe",
  "excerpt": "A beginner's guide to Rust programming language",
  "content": "# Introduction\n\nRust is a systems programming language...",
  "published": true,
  "featured": false,
  "image": null,
  "tags": [
    {
      "id": 1,
      "name": "Rust",
      "slug": "rust"
    },
    {
      "id": 2,
      "name": "Programming",
      "slug": "programming"
    }
  ],
  "metadata": {}
}
```

### Retrieving Blog Posts with Pagination

**Request:**
```bash
curl "http://localhost:3000/api/posts?page=1&per_page=2"
```

**Response:**
```json
{
  "data": [
    {
      "id": 2,
      "title": "Advanced Rust Techniques",
      "slug": "advanced-rust-techniques",
      "date": "2025-07-24",
      "author": "John Smith",
      "excerpt": "Exploring advanced features of Rust",
      "content": "# Advanced Rust\n\nIn this post, we'll explore...",
      "published": true,
      "featured": true,
      "image": "/images/rust-advanced.jpg",
      "tags": [
        {
          "id": 1,
          "name": "Rust",
          "slug": "rust"
        },
        {
          "id": 3,
          "name": "Advanced",
          "slug": "advanced"
        }
      ],
      "metadata": {
        "readTime": "10 min"
      }
    },
    {
      "id": 1,
      "title": "Getting Started with Rust",
      "slug": "getting-started-with-rust",
      "date": "2025-07-23",
      "author": "Jane Doe",
      "excerpt": "A beginner's guide to Rust programming language",
      "content": "# Introduction\n\nRust is a systems programming language...",
      "published": true,
      "featured": false,
      "image": null,
      "tags": [
        {
          "id": 1,
          "name": "Rust",
          "slug": "rust"
        },
        {
          "id": 2,
          "name": "Programming",
          "slug": "programming"
        }
      ],
      "metadata": {}
    }
  ],
  "pagination": {
    "total": 45,
    "per_page": 2,
    "current_page": 1,
    "last_page": 23,
    "next_page_url": "/api/posts?page=2&per_page=2",
    "prev_page_url": null
  }
}
```

### Handling Errors

**Request (non-existent post):**
```bash
curl http://localhost:3000/api/posts/non-existent-post
```

**Response:**
```json
{
  "error": {
    "message": "Blog post not found",
    "code": "not_found"
  }
}
```

## SDK Libraries

We're working on official SDK libraries for popular programming languages to make integration even easier. In the meantime, you can use any HTTP client library to interact with the API.

## Support

If you encounter any issues or have questions about the API, please:

1. Check the [FAQ section](USER_DOCUMENTATION.md#faq) in the user documentation
2. Open an issue on our GitHub repository
3. Contact our support team at support@example.com