# Git-Based Authentication Deploy Changes

This document summarizes the changes made to the deploy scripts to support the new Git-based single-user authentication system.

## Changes Made

### 1. Docker Configuration Updates

#### Dockerfile and Dockerfile.local
- Added Git as a dependency in both build and runtime stages
- Ensures Git is available for authentication and identity detection

#### startup.sh
- Added Git configuration from environment variables
- Added colored output functions for better logging
- Provides warnings if Git identity is not configured

### 2. Docker Compose Updates

#### docker-compose.yml and docker-compose.local.yml
- Added environment variables for Git identity:
  - `GIT_USER_NAME`: User's name from Git config
  - `GIT_USER_EMAIL`: User's email from Git config
  - `GITHUB_USERNAME`: GitHub username
  - `GITHUB_TOKEN`: Optional GitHub API token

### 3. Deploy Script Updates

#### deploy-local.sh
- Added function to check Git configuration
- Added function to create environment file with Git configuration
- Added function to create and update config.toml from template
- Updated all docker-compose commands to use the environment file
- Improved usage information with Git-related environment variables

### 4. Configuration Template

#### config.toml.template
- Created template with owner/identity configuration
- Includes settings for Git-based authentication
- Provides development and server configuration
- Supports automatic detection of Git identity

## Testing Instructions

### 1. Test Git Configuration Detection

```bash
# Test with global Git config
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
./scripts/deploy-local.sh start

# Check if config.toml was created with correct values
cat config.toml
```

### 2. Test with Custom Environment Variables

```bash
# Test with custom environment variables
GIT_USER_NAME="Custom Name" GIT_USER_EMAIL="custom@example.com" GITHUB_USERNAME="customuser" ./scripts/deploy-local.sh start

# Check if config.toml was created with custom values
cat config.toml
```

### 3. Check Application Health

```bash
# Check application status
./scripts/deploy-local.sh status

# Check application logs
./scripts/deploy-local.sh logs
```

### 4. Test Authentication

```bash
# Access the application
curl http://localhost:3002/health

# Try to create a session (should use Git identity)
curl -X POST http://localhost:3002/api/blog/session
```

## Troubleshooting

If you encounter issues with Git-based authentication:

1. **Check Git Configuration**: Ensure Git is properly configured with `git config --global user.name` and `git config --global user.email`

2. **Check Environment Variables**: Verify environment variables are correctly set in `.env.local`

3. **Check config.toml**: Verify the owner section in `config.toml` has the correct values

4. **Check Container Logs**: Use `./scripts/deploy-local.sh logs` to check for any error messages

5. **Rebuild the Application**: If necessary, use `./scripts/deploy-local.sh rebuild` to rebuild the application with the latest changes