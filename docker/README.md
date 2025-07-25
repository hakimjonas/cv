# Docker Configuration for CV Project

## Recent Changes

### Fix for Docker Build Process

#### Issue
The Docker build process was failing with the following errors:
```
error: couldn't read `src/bin/blog_api_server.rs`: No such file or directory (os error 2)
error: couldn't read `src/bin/test_blog_core.rs`: No such file or directory (os error 2)
error: couldn't read `src/bin/blog_tester.rs`: No such file or directory (os error 2)
error: couldn't read `src/bin/security_test.rs`: No such file or directory (os error 2)
```

#### Root Cause
The Dockerfile.local was creating a dummy `src/main.rs` file to pre-build dependencies, but it wasn't creating the binary target files specified in Cargo.toml. When Cargo tried to build the project, it was looking for these binary files but couldn't find them.

#### Solution
Modified the Dockerfile.local to create dummy files for all binary targets defined in Cargo.toml:
1. Changed `mkdir src` to `mkdir -p src/bin` to create the bin directory
2. Added lines to create dummy files for each binary target:
   - src/bin/blog_tester.rs
   - src/bin/security_test.rs
   - src/bin/test_blog_core.rs
   - src/bin/blog_api_server.rs

#### Why This Works
During the Docker build phase, all the binary target files specified in Cargo.toml now exist, even though they're just dummy files with a simple `fn main() {}` function. This allows Cargo to successfully build the dependencies without errors.

When the container is actually run, the real source code is mounted as volumes (as specified in docker-compose.local.yml), replacing these dummy files with the actual implementation.

### Fix for Port Conflict Issue

#### Issue
When starting the local development environment, the following error occurred:
```
Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint docker-blog-api-1 (6bf029d5db630afd98e247a73f39299451ec534c60d41ed7aaa5633519253e9e): Bind for 0.0.0.0:3002 failed: port is already allocated
```

#### Root Cause
The Docker container was trying to bind to port 3002 on the host machine, but that port was already in use by another process.

#### Solution
1. Modified docker-compose.local.yml to make the host port configurable via an environment variable:
   ```yaml
   ports:
     - "${HOST_PORT:-3002}:3000"
   ```

2. Updated deploy-local.sh script to:
   - Check if port 3002 is already in use
   - Automatically find and use an available port if 3002 is not available
   - Display the correct URLs with the actual port being used

#### How It Works
- If port 3002 is available, it will be used (maintaining backward compatibility)
- If port 3002 is not available, the script will automatically find an available port
- The script will display the correct URLs with the actual port being used
- Users can also manually specify a port with `HOST_PORT=<port> ./scripts/deploy-local.sh start`

### Fix for Container Restart Issue

#### Issue
The container was repeatedly restarting with exit code 101, and the health check was failing with status "unhealthy":
```
docker-blog-api-1   blog-api-local:0.1.0-dev   "cargo run --bin blo…"   blog-api   5 minutes ago   Restarting (101) 14 seconds ago
```

#### Root Cause
The application was panicking during startup with the following error:
```
thread 'main' panicked at src/image_api.rs:212:10:
Path segments must not start with `:`. For capture groups, use `{capture}`. If you meant to literally match a segment starting with a colon, call `without_v07_checks` on the router.
```

This was due to an incompatibility between the route path format used in the code and the version of the Axum web framework being used. Axum 0.8.x requires using the `{capture}` syntax for path parameters instead of the colon syntax (`:param`).

#### Solution
Updated the route paths in `src/image_api.rs` to use the new syntax:

Changed from:
```
Router::new()
    .route("/api/images", post(upload_image))
    .route("/api/images", get(list_images))
    .route("/api/images/:filename", get(get_image))
    .route("/api/images/:filename", delete(delete_image))
    .with_state(state)
```

To:
```
Router::new()
    .route("/api/images", post(upload_image))
    .route("/api/images", get(list_images))
    .route("/api/images/{filename}", get(get_image))
    .route("/api/images/{filename}", delete(delete_image))
    .with_state(state)
```

#### Why This Works
The new syntax is compatible with Axum 0.8.x, which is the version being used in the project. Axum 0.8.x introduced a breaking change in how path parameters are defined, requiring the use of curly braces instead of colons for path parameters.

## Testing the Changes
To test the changes, run:
```bash
./scripts/deploy-local.sh rebuild
```

This will:
1. Stop any running containers
2. Rebuild the Docker images with --no-cache (ensuring our changes are applied)
3. Start the containers again

If the changes are correct, the build should succeed without the previous errors, and the application should be accessible at the URLs displayed by the script.