# API Client Fix

## Issue

The blog client was experiencing an error when trying to log in:

```
Login failed: BlogAPI.login is not a function
```

## Root Cause

There were two different versions of the `api-client.js` file in the project:

1. `/static/js/api-client.js` - This version had the authentication functions (login, logout, etc.) defined.
2. `/dist/js/api-client.js` - This version did NOT have the authentication functions defined.

The blog client was loading the API client from `/static/js/api-client.js`, but the server was likely serving the file from `/dist/js/api-client.js`, which didn't have the login function.

## Solution

The solution was to copy the updated API client from the static directory to the dist directory:

```bash
cp /home/hakim/personal/cv/static/js/api-client.js /home/hakim/personal/cv/dist/js/api-client.js
```

Now both versions of the file have the login function defined, so regardless of which one is being served, the login functionality works.

## Recommendations

To prevent similar issues in the future:

1. **Use a build process**: Set up a proper build process that automatically copies files from the source directory to the distribution directory. This ensures that changes to source files are consistently reflected in the distribution files.

2. **Version your API client**: Add a version number to your API client and log it when the client is loaded. This makes it easier to identify which version is being used.

3. **Add a check for required functions**: Add a self-check in the API client that verifies all required functions are defined when the client is loaded. This would catch similar issues early.

4. **Document the file structure**: Clearly document the purpose of each directory (static vs. dist) and how files should be updated.

5. **Consider using a module bundler**: Tools like Webpack, Rollup, or Parcel can help manage dependencies and ensure consistent builds.

## Testing

After applying the fix, the login functionality was tested and confirmed to be working correctly. The login function is now available in both versions of the API client.