# GitHub OAuth Setup Guide

This guide explains how to set up GitHub OAuth for the CV application, allowing users to log in with their GitHub accounts.

## Prerequisites

- A GitHub account
- Access to the CV application codebase
- Basic understanding of OAuth 2.0 authentication flow

## Creating a GitHub OAuth App

1. Go to your GitHub account settings: https://github.com/settings/profile
2. Navigate to "Developer settings" in the left sidebar
3. Click on "OAuth Apps"
4. Click the "New OAuth App" button
5. Fill in the application details:
   - **Application name**: CV Application (or any name you prefer)
   - **Homepage URL**: http://localhost:3002 (for local development) or your production URL
   - **Application description**: (Optional) A brief description of your application
   - **Authorization callback URL**: http://localhost:3002/api/auth/github/callback (for local development) or your production callback URL
6. Click "Register application"
7. On the next page, you'll see your Client ID
8. Click "Generate a new client secret" to create a client secret
9. **Important**: Save both the Client ID and Client Secret securely. The Client Secret will only be shown once.

## Configuring the Application

There are several ways to configure GitHub OAuth credentials in the application:

### 1. Using Environment Variables

Set the following environment variables:

```bash
export CV__GITHUB_OAUTH__CLIENT_ID=your-github-client-id
export CV__GITHUB_OAUTH__CLIENT_SECRET=your-github-client-secret
export CV__GITHUB_OAUTH__REDIRECT_URL=http://localhost:3002/api/auth/github/callback
```

### 2. Using a Configuration File

Create or edit `config.toml` in the project root:

```toml
[github_oauth]
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
redirect_url = "http://localhost:3002/api/auth/github/callback"
```

### 3. Using Command-line Arguments

When starting the application, you can provide the GitHub OAuth credentials as command-line arguments:

```bash
cargo run --bin blog_api_server -- --github-oauth-client-id=your-github-client-id --github-oauth-client-secret=your-github-client-secret
```

## Verifying the Configuration

To verify that GitHub OAuth is properly configured:

1. Start the application
2. Navigate to http://localhost:3002/static/blog-client.html
3. Click the "Login with GitHub" button
4. You should be redirected to GitHub's authorization page
5. After authorizing, you should be redirected back to the application and logged in

## Troubleshooting

### Common Issues

1. **"GitHub OAuth is not properly configured" error**:
   - Check that you've set the correct Client ID and Client Secret
   - Ensure the environment variables or configuration file are properly loaded
   - Verify that the application can access the configuration

2. **Redirect URI mismatch error from GitHub**:
   - Ensure the redirect URL in your GitHub OAuth app settings exactly matches the one in your application configuration
   - For local development, it should be `http://localhost:3002/api/auth/github/callback`

3. **Authentication fails after GitHub authorization**:
   - Check the application logs for detailed error messages
   - Verify that the callback URL is correctly handling the authorization code
   - Ensure the application can communicate with GitHub's API

4. **Rate limiting issues**:
   - GitHub API has rate limits for unauthenticated requests
   - If you're experiencing rate limiting, ensure your GitHub OAuth credentials are correctly configured

### Debugging

To enable more detailed logging for OAuth-related issues:

```bash
export RUST_LOG=debug,cv::github_oauth=trace
```

This will show detailed logs for the GitHub OAuth process, which can help identify issues.

## Security Considerations

- Never commit your GitHub OAuth credentials to version control
- Use environment variables or a secure configuration management system in production
- Regularly rotate your client secret for better security
- Implement proper CSRF protection (already included in the application)
- Use HTTPS in production to protect the authorization code and tokens

## Additional Resources

- [GitHub OAuth Documentation](https://docs.github.com/en/developers/apps/building-oauth-apps/creating-an-oauth-app)
- [OAuth 2.0 Specification](https://oauth.net/2/)
- [Rust OAuth2 Crate Documentation](https://docs.rs/oauth2/latest/oauth2/)