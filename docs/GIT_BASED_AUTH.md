# Git-Based Authentication System

## Overview

This document describes the new Git-based authentication system that simplifies the authentication process by using Git configuration for identity detection in a single-user application.

## Key Benefits

- **Zero setup friction**: Works with existing Git configuration
- **No user management complexity**: Single user assumed
- **Leverages developer tools**: Uses Git CLI that developers already have
- **Maintains all features**: CV generation, blog, project showcase remain intact
- **Development friendly**: Easy local testing and setup

## Components

### 1. Git Identity Service

The `GitIdentityService` extracts user identity from Git configuration:

- **Name**: From `git config --global user.name`
- **Email**: From `git config --global user.email`
- **GitHub Username**: Extracted from Git remote URL or GitHub CLI

### 2. Owner Configuration

The application configuration now includes an `OwnerConfig` that stores the owner's information:

- **Name**: The owner's name
- **GitHub Username**: The owner's GitHub username
- **Email**: The owner's email
- **Display Name**: Optional display name (defaults to name)
- **Bio**: Optional bio
- **Role**: The owner's role (defaults to "Author")

### 3. Simple Authentication Service

The `SimpleAuthService` replaces the database-backed authentication with Git-based identity:

- **Session Creation**: Creates a session based on Git identity
- **Token Generation**: Generates JWT tokens for authentication
- **Development Mode**: Automatic authentication for localhost in development mode

### 4. Simple Blog API

The `simple_blog_api` module provides a simplified API that uses the Git-based authentication:

- **Session Endpoint**: Creates a session based on Git identity
- **Blog Management**: Maintains all blog functionality
- **Simplified Authentication**: No login/registration required

## How to Use

### Running the Server

To run the server with Git-based authentication:

```bash
cargo run --bin simple_blog_api_server
```

### Environment Variables

- `DEV_MODE`: Set to `1` or `true` to enable development mode (default: `false`)
- `JWT_SECRET`: Secret key for JWT token signing (default: randomly generated)
- `TOKEN_EXPIRATION`: Token expiration time in seconds (default: 86400 - 24 hours)

### Development Mode

In development mode:

- Automatic authentication for localhost requests
- No need to manually create a session
- Helpful for local testing and development

### Git Configuration

Before running the server, ensure Git is properly configured:

```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

If you want to use GitHub features, ensure your repository has a GitHub remote:

```bash
git remote -v
# Should show something like:
# origin  git@github.com:username/repo.git (fetch)
# origin  git@github.com:username/repo.git (push)
```

## Client Usage

The new client is available at:

```
http://localhost:3000/static/simple-blog-client.html
```

The client automatically creates a session based on Git identity when loaded. No login form is displayed, and the user is automatically authenticated.

## Migrating from the Old System

The old authentication system is still available for backward compatibility. To use it:

```bash
cargo run --bin blog_api_server
```

The old client is available at:

```
http://localhost:3000/static/blog-client.html
```

## Technical Details

### Authentication Flow

1. Client loads and requests a session from `/api/session`
2. Server extracts Git identity and creates a session
3. Server returns a JWT token and user information
4. Client stores the token and uses it for subsequent requests

### Token Format

The JWT token contains:

- **sub**: User ID (always 1 for single-user system)
- **username**: Git username or GitHub username
- **role**: User role (defaults to "Author")
- **exp**: Expiration time
- **iat**: Issued at time

### API Endpoints

- **POST /api/session**: Creates a session based on Git identity
- **GET /api/posts**: Gets all blog posts
- **GET /api/posts/:slug**: Gets a specific blog post
- **POST /api/posts**: Creates a new blog post (requires authentication)
- **PUT /api/posts/:slug**: Updates a blog post (requires authentication)
- **DELETE /api/posts/:slug**: Deletes a blog post (requires authentication)

## Troubleshooting

### Git Configuration Issues

If you encounter errors about Git configuration:

1. Check your Git configuration:
   ```bash
   git config --global user.name
   git config --global user.email
   ```

2. Set Git configuration if missing:
   ```bash
   git config --global user.name "Your Name"
   git config --global user.email "your.email@example.com"
   ```

### Authentication Issues

If authentication fails:

1. Check the server logs for error messages
2. Ensure Git is properly configured
3. Try enabling development mode for local testing

### GitHub Username Detection

If GitHub username detection fails:

1. Ensure your repository has a GitHub remote
2. Try installing and authenticating with GitHub CLI:
   ```bash
   gh auth login
   ```

## Future Improvements

- Add support for custom roles and permissions
- Improve GitHub integration with additional features
- Add support for multiple Git identities
- Add support for Git configuration overrides