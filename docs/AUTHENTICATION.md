# Authentication Guide

## Overview

This document explains how authentication works in the blog application and how to use it.

## Authentication Flow

1. User enters their username and password in the login form
2. Client sends a POST request to `/api/auth/login` with the credentials
3. Server validates the credentials and returns a JWT token
4. Client stores the token in localStorage
5. Client includes the token in the Authorization header of subsequent requests
6. Server validates the token and allows or denies the request

## Using Authentication in the Blog Client

The blog client now includes authentication functionality:

1. A login tab has been added to the interface
2. An authentication status bar shows the current login status
3. Login and logout buttons allow users to authenticate and deauthenticate
4. Protected operations (creating, updating, and deleting posts) now require authentication

### How to Login

1. Click the "Login" button in the authentication status bar or select the "Login" tab
2. Enter your username and password
3. Click the "Login" button
4. If successful, you'll see a success message and the authentication status will update

### How to Logout

1. Click the "Logout" button in the authentication status bar
2. You'll be logged out and the authentication status will update

## Testing Authentication

To test the authentication functionality:

1. Open the blog client at http://localhost:3002/static/blog-client.html
2. Try to create a new post without logging in - you should see an "Authentication required" error
3. Login using valid credentials
4. Try to create a new post again - it should now succeed
5. Logout and verify that you can no longer create posts

## Default User Accounts

The system comes with the following default user accounts:

- Admin user:
  - Username: admin
  - Password: admin123
  - Role: Admin

- Author user:
  - Username: author
  - Password: author123
  - Role: Author

- Editor user:
  - Username: editor
  - Password: editor123
  - Role: Editor

## Technical Implementation

### API Client

The API client (api-client.js) has been updated to include authentication functionality:

- Token storage and retrieval using localStorage
- User info storage and retrieval
- Authentication status checking
- Login and logout methods
- Including the authentication token in request headers

### Blog Client

The blog client (blog-client.html) has been updated to include authentication UI:

- Authentication status bar
- Login and logout buttons
- Login form
- User info display
- JavaScript code to handle login, logout, and updating the authentication status

## Troubleshooting

If you encounter authentication issues:

1. Check the browser console for error messages
2. Verify that you're using the correct credentials
3. Try clearing localStorage and logging in again
4. Ensure the server is running and accessible

If you need to reset your password or create a new account, contact the system administrator.