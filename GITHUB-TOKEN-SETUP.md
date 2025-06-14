# GitHub Token Setup

This document explains how to set up a GitHub token for the CV generator application.

## Overview

The CV generator application uses a GitHub token to fetch information about your GitHub projects. Without a token, the application will be subject to lower rate limits when making API requests to GitHub.

## Token Retrieval Logic

The application checks for a GitHub token in the following order:

1. GitHub Actions environment (if running in GitHub Actions)
2. Secure storage
3. Git config
4. Environment variable

## Setting Up a GitHub Token

You can set up a GitHub token using one of the following methods:

### Method 1: Using the Command-Line Interface

The application provides a command-line interface for setting the GitHub token in secure storage:

```bash
cv --set-token <your-token>
```

This is the recommended method as it stores the token securely.

### Method 2: Using Git Config

You can set the GitHub token in your Git config using the following command:

```bash
git config --global cv.github.token <your-token>
```

### Method 3: Using Environment Variables

You can set the GitHub token as an environment variable:

```bash
export GITHUB_TOKEN=<your-token>
```

This method is less secure but can be useful for temporary usage.

## Removing a GitHub Token

To remove a GitHub token from secure storage:

```bash
cv --remove-token
```

To remove a GitHub token from Git config:

```bash
git config --global --unset cv.github.token
```

## Creating a GitHub Token

To create a GitHub token:

1. Go to your GitHub account settings
2. Click on "Developer settings"
3. Click on "Personal access tokens"
4. Click on "Generate new token"
5. Give your token a name and select the appropriate scopes (at minimum, you need the `repo` scope for public repositories)
6. Click on "Generate token"
7. Copy the token and use one of the methods above to set it in the application

## Troubleshooting

If you're having issues with the GitHub token:

1. Check if the token is set correctly using one of the methods above
2. Check if the token has the appropriate scopes
3. Check if the token has expired (tokens can expire if you set an expiration date when creating them)
4. Try setting the token using a different method

## Recent Changes

The application now checks Git config for the GitHub token, which was missing in previous versions. This means that if you previously set the token in Git config, it will now be found by the application.