# Blog System Documentation

This document consolidates all blog-related documentation for this project.

## Overview

The blog functionality allows you to create, read, update, and delete blog posts. It consists of:

1. A data model for blog posts
2. An admin interface for managing blog posts
3. API endpoints for CRUD operations
4. Integration with the main application for displaying blog posts

## Blog Post Data Model

Blog posts are stored in a SQLite database with the following fields:

- `id`: Unique identifier for the post
- `title`: The title of the post
- `slug`: URL-friendly identifier derived from the title
- `date`: Publication date in YYYY-MM-DD format
- `author`: The author of the post
- `excerpt`: A brief summary of the post
- `content`: Full post content in Markdown format
- `published`: Boolean indicating if the post is published
- `featured`: Boolean indicating if the post is featured
- `image`: Optional URL to a featured image

Posts can also have:
- Tags: Categories for organizing posts
- Metadata: Key-value pairs for additional information

## API Endpoints

The following API endpoints are available for blog operations:

- `GET /api/blog` - Get all blog posts
- `POST /api/blog` - Create a new blog post
- `GET /api/blog/:slug` - Get a specific blog post
- `PUT /api/blog/:slug` - Update a specific blog post
- `DELETE /api/blog/:slug` - Delete a specific blog post
- `GET /api/blog/tags` - Get all tags
- `GET /api/blog/published` - Get all published posts
- `GET /api/blog/featured` - Get all featured posts
- `GET /api/blog/tag/:tag_slug` - Get posts by tag

## Testing

The blog system has comprehensive tests that can be run with the `test.sh` script.

### Testing the Core Blog Functionality

The core blog functionality is tested using the `blog_tester` binary, which tests CRUD operations for blog posts without starting a server.

### Testing the Blog API

The API is tested with a real server running on http://localhost:3000. The test script automatically starts and stops the server.

## Common Issues and Solutions

### Database Locking

If you encounter database locking issues, it's typically due to SQLite's WAL mode. These are generally harmless and operations are likely successful even if a lock error occurs during a commit.

### API Response Format

All API responses use JSON format. Errors return a JSON object with an `error` field containing the error message.

## Development Guidelines

1. Follow the immutable data structure pattern for all blog data operations
2. Use the provided helper methods for blog post manipulation
3. Always validate input data before saving to the database
4. Run the comprehensive test suite before submitting changes
