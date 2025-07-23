# Architecture Documentation

## Table of Contents
1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Component Structure](#component-structure)
4. [Data Flow](#data-flow)
5. [Code Style and Conventions](#code-style-and-conventions)
6. [Error Handling Patterns](#error-handling-patterns)
7. [Testing Strategy](#testing-strategy)
8. [Performance Considerations](#performance-considerations)
9. [Security Considerations](#security-considerations)
10. [Frontend Architecture](#frontend-architecture)
11. [Database Architecture](#database-architecture)
12. [Deployment Architecture](#deployment-architecture)

## Overview

This document provides a comprehensive overview of the architecture, design patterns, and implementation details of the Personal Website with Dynamic CV Generator and Blog project. The project is built with Rust using functional programming principles and modern web technologies.

The system consists of two main components:
1. A static site generator that creates HTML and PDF versions of a CV from a single data source
2. A blog API server that provides a RESTful interface for content management

## System Architecture

The system follows a hybrid architecture:

1. **Static Site Generation**: The CV generator is a Rust application that processes data from a JSON file and generates static HTML and PDF files. This follows a compile-time rendering approach where content is generated during the build process.

2. **Dynamic API Server**: The blog component is a RESTful API server built with Axum that provides dynamic content management capabilities. This follows a more traditional client-server architecture.

### High-Level Architecture Diagram

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │     │                 │
│  Data Sources   │────▶│  Rust Backend   │────▶│  Static Output  │
│  (JSON, SQLite) │     │  (Generators)   │     │  (HTML, PDF)    │
│                 │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               │
                               ▼
                        ┌─────────────────┐     ┌─────────────────┐
                        │                 │     │                 │
                        │  Blog API       │◀───▶│  Client         │
                        │  Server (Axum)  │     │  (Browser)      │
                        │                 │     │                 │
                        └─────────────────┘     └─────────────────┘
```

## Component Structure

The project is organized into several key components:

### CV Generator Components

1. **Data Model** (`cv_data.rs`): Defines the data structures for CV information.
2. **HTML Generator** (`html_generator.rs`): Generates HTML files from CV data using Askama templates.
3. **PDF Generator** (`typst_generator.rs`): Generates PDF files from CV data using Typst.
4. **GitHub Integration** (`github.rs`): Fetches repository information from GitHub API.

### Blog API Components

1. **Data Model** (`blog_data.rs`): Defines the data structures for blog posts and tags.
2. **API Server** (`blog_api_server.rs`): Implements the RESTful API endpoints using Axum.
3. **Database Layer** (`db/`): Manages database connections and operations.
   - **Repository** (`db/repository.rs`): Implements the repository pattern for data access.
   - **Migrations** (`db/migrations.rs`): Handles database schema migrations.
   - **Error Handling** (`db/error.rs`): Defines custom error types for database operations.

### Shared Components

1. **Configuration** (`config.rs`): Manages application configuration.
2. **Logging** (`logging.rs`): Configures structured logging.
3. **Credentials** (`credentials.rs`): Manages secure storage of API tokens.

## Data Flow

### CV Generation Flow

1. The application loads CV data from a JSON file (`data/cv_data.json`).
2. If GitHub integration is enabled, it fetches repository information from the GitHub API.
3. The HTML generator processes the data through Askama templates to create HTML files.
4. The PDF generator processes the data through Typst to create a PDF file.
5. The generated files are written to the output directory (`dist/`).

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │     │             │
│  Load Data  │────▶│  Fetch GitHub │────▶│  Generate   │────▶│  Write to   │
│  from JSON  │     │  Repositories │     │  HTML/PDF  │     │  Output Dir │
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

### Blog API Flow

1. The client sends a request to one of the API endpoints.
2. The Axum router directs the request to the appropriate handler.
3. The handler processes the request, interacting with the database through the repository layer.
4. The response is formatted and returned to the client.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │     │             │
│  Client     │────▶│  Axum       │────▶│  Handler    │────▶│  Repository │
│  Request    │     │  Router     │     │  Function   │     │  Layer      │
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                                                   │
                                                                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │     │             │
│  Client     │◀────│  Response   │◀────│  Handler    │◀────│  Database   │
│  Response   │     │  Formatting │     │  Processing │     │  (SQLite)   │
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

## Code Style and Conventions

The project follows a functional programming approach with an emphasis on immutability, pure functions, and strong typing. The following conventions are used:

### Naming Conventions

- **Files**: Snake case (e.g., `html_generator.rs`)
- **Modules**: Snake case (e.g., `mod html_generator;`)
- **Functions**: Snake case (e.g., `generate_html()`)
- **Types/Structs/Enums**: Pascal case (e.g., `BlogPost`, `CvTemplate`)
- **Constants**: Screaming snake case (e.g., `GITHUB_TOKEN_KEY`)
- **Variables**: Snake case (e.g., `blog_post`, `html_content`)

### Functional Programming Principles

1. **Immutability**: The project uses the `im` crate for immutable data structures. This ensures that data structures are not modified in place, but instead new versions are created with the desired changes. This approach helps prevent side effects and makes code more predictable.

2. **Pure Functions**: Functions avoid side effects and return the same output for the same input. This makes them easier to test, reason about, and compose. For example, functions that add a tag to a blog post return a new blog post with the tag added, rather than modifying the original post.

3. **Function Composition**: Complex operations are built through function composition. The project uses iterators and combinators to create data processing pipelines, making the code more declarative and easier to understand.

### Module Organization

Each module has a clear, single responsibility:
- Public interfaces are well-documented with doc comments
- Implementation details are kept private when possible
- Related functionality is grouped together

## Error Handling Patterns

The project uses a comprehensive error handling approach with custom error types and the `Result` monad.

### Custom Error Types

Each module defines its own error type using the `thiserror` crate. This allows for specific error types that provide context about what went wrong. For example, the blog module defines errors like `PostNotFound`, `DatabaseError`, and `ValidationError`.

### Error Propagation

Errors are propagated using the `?` operator, which allows for concise error handling. Functions return early if an error occurs, and the error is automatically converted to the function's error type if a suitable `From` implementation exists.

### Error Conversion

The project implements the `From` trait for error conversion, allowing errors from dependencies to be automatically converted to the project's error types. This makes error handling more ergonomic and ensures that all errors are properly typed.

### Error Logging

Errors are logged using structured logging with the `tracing` crate. This provides context about where the error occurred and what was happening at the time. The project uses the `#[instrument]` attribute to automatically log function entries and exits.

## Testing Strategy

The project employs a comprehensive testing strategy with multiple types of tests:

### Unit Tests

Unit tests verify the behavior of individual functions and methods. They focus on testing a single unit of code in isolation, with dependencies mocked or stubbed as needed. The project uses Rust's built-in testing framework with the `#[test]` attribute.

### Integration Tests

Integration tests verify the behavior of the API endpoints. They test the interaction between different components of the system, such as the API handlers and the database. The project uses the Axum testing utilities to simulate HTTP requests and verify responses.

### Property-Based Tests

Property-based tests verify properties of the code using randomly generated inputs. Instead of testing specific examples, they test that certain properties hold for all possible inputs. The project uses the `proptest` crate for property-based testing.

### Idempotency Tests

Idempotency tests verify that operations can be applied multiple times with the same result. This is important for ensuring that API endpoints are idempotent, meaning that making the same request multiple times has the same effect as making it once.

## Performance Considerations

The project implements several performance optimizations:

### Lazy Evaluation

Lazy evaluation is used when processing large collections. Operations like filtering and mapping are only performed when the results are actually needed, which can significantly improve performance when only a subset of the results is used.

### Connection Pooling

Connection pooling is used for database access. This reduces the overhead of creating new database connections for each request, improving response times and reducing resource usage. The project uses the `r2d2` crate for connection pooling.

### Caching

Caching is used for expensive operations. Results of operations like database queries are cached in memory to avoid repeating the same work. The project uses the `once_cell` crate for thread-safe lazy initialization of caches.

### Minification and Compression

In production mode, HTML, CSS, and JavaScript files are minified and compressed. This reduces the size of the files sent to clients, improving load times. The project uses the `minify-html` crate for HTML minification and the `flate2` crate for gzip compression.

## Security Considerations

The project implements several security measures:

### Input Validation

All user inputs are validated before processing. This helps prevent injection attacks and ensures that the data is in the expected format. The project uses custom validation functions that return specific error types for different validation failures.

### Secure Credential Storage

API tokens are stored securely using the system's credential manager. This ensures that sensitive information is not stored in plain text. The project uses the `keyring` crate for secure credential storage.

### HTTP Security Headers

Security headers are added to HTTP responses. These headers help prevent common web vulnerabilities like cross-site scripting (XSS) and clickjacking. The project adds headers like `X-Content-Type-Options`, `X-Frame-Options`, and `Content-Security-Policy`.

## Frontend Architecture

The frontend follows a template-based architecture with server-side rendering:

### Template Structure

1. **Base Template** (`templates/base.html`): Provides the common structure for all pages.
2. **Page Templates**:
   - `templates/cv.html`: CV page template
   - `templates/index.html`: Landing page template
   - `templates/projects.html`: Projects page template
3. **Partial Templates** (`templates/partials/`):
   - `header.html`: Header component
   - `footer.html`: Footer component
   - `project-card.html`: Project card component

### Server-Side Rendering

The HTML is generated server-side using the Askama templating engine. This approach ensures that the content is available immediately when the page loads, improving both performance and SEO. The templates are compiled at build time, which catches errors early and improves runtime performance.

### Client-Side JavaScript

JavaScript is used only for interactive elements:

1. **Theme Switching**: Toggles between light and dark themes.
2. **Accordion Functionality**: Expands and collapses accordion sections.
3. **Blog Integration**: Fetches and displays blog posts on the landing page.

## Database Architecture

The blog system uses SQLite with a repository pattern:

### Database Schema

The database schema includes tables for blog posts, tags, and relationships. The schema is designed to be normalized, with foreign keys ensuring referential integrity. The project uses migrations to manage schema changes over time.

### Repository Pattern

The repository pattern is used for database access. This pattern provides an abstraction layer between the application logic and the database, making it easier to change the database implementation or add caching. Each repository is responsible for a specific domain entity, like blog posts or tags.

### Connection Pooling

Connection pooling is used for efficient database connections. This reduces the overhead of creating new connections for each request and ensures that the number of connections is limited to prevent resource exhaustion. The project uses the `r2d2` crate for connection pooling.

## Deployment Architecture

The project supports multiple deployment options:

### CV Site Deployment

The CV site is a static site that can be deployed to:
- GitHub Pages (using GitHub Actions)
- Traditional web hosting
- Netlify

### Blog API Deployment

The blog API server can be deployed using:
- Docker (with Docker Compose)
- Manual deployment of the binary
- CI/CD pipeline with GitHub Actions

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