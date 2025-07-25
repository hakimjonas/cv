# Default User Accounts

## Overview

This document explains how to create and use the default user accounts for the blog application.

## Default Credentials

The system comes with the following default user accounts:

- Admin user:
  - Username: `admin`
  - Password: `admin123`
  - Role: Admin

- Author user:
  - Username: `author`
  - Password: `author123`
  - Role: Author

- Editor user:
  - Username: `editor`
  - Password: `editor123`
  - Role: Editor

## Creating Default Users

To create these default user accounts, run the following command:

```bash
cargo run --bin create_default_users
```

This script will:
1. Check if each user already exists
2. Create any missing default users
3. Display the credentials for reference

The script is safe to run multiple times - it will only create users that don't already exist.

## Using Default Users

Once the default users are created, you can log in using their credentials:

1. Open the blog client at http://localhost:3002/static/blog-client.html
2. Click the "Login" button in the authentication status bar or select the "Login" tab
3. Enter the username and password for one of the default users
4. Click the "Login" button

### User Permissions

Each user role has different permissions:

- **Admin**: Full access to all features, including creating, editing, and deleting any posts
- **Author**: Can create new posts and edit their own posts
- **Editor**: Can edit existing posts but cannot create new ones
- **Viewer**: Can only view content (no default viewer account is created)

## Troubleshooting

If you encounter issues with the default user accounts:

1. Make sure you've run the `create_default_users` script
2. Check that the database file exists at `data/blog.db`
3. Verify that you're using the correct credentials
4. Check the browser console for error messages
5. Ensure the server is running and accessible

## Security Note

These default accounts are intended for development and testing purposes only. In a production environment, you should:

1. Change the passwords for these accounts
2. Create new accounts with strong passwords
3. Consider disabling or removing the default accounts

## Technical Details

The default user accounts are created using the `UserRepository` in the application. The script is located at:

```
src/bin/create_default_users.rs
```

The script uses the same user creation logic as the rest of the application, ensuring that the users are properly created with hashed passwords and the correct roles.