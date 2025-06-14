# Project Roadmap

This document outlines the current state of the project, completed work, and the plan for future development with a focus on deployment.

## Project Status

The project currently includes:

1. A dynamic CV generator that produces HTML and PDF outputs from a single data source
2. A blog system with a RESTful API for creating, reading, updating, and deleting posts
3. GitHub integration for fetching repository information
4. Docker support for easy deployment

## Completed Work

The project has successfully completed several important milestones:

1. ✅ **Database Architecture Improvements**:
   - Implemented proper connection pooling
   - Created a migrations system for database schema
   - Implemented a repository layer for blog operations

2. ✅ **Logging Enhancements**:
   - Added proper logging with tracing throughout the codebase
   - Set up global subscribers for logging in main binary entry points
   - Implemented structured logging for errors

3. ✅ **Testing Improvements**:
   - Implemented property-based testing for the blog functionality
   - Created generators for BlogPost, Tag, and other data types
   - Added tests for serialization/deserialization roundtrips
   - Implemented tests for idempotency of operations

4. ✅ **Error Handling Improvements**:
   - Implemented proper error types for blog operations
   - Replaced anyhow::Result with custom error types
   - Added comprehensive error conversion implementations
   - Ensured consistent use of Result monads throughout the codebase

5. ✅ **Functional Programming Transition**:
   - Ensured all data structures use `im` crate's immutable collections where appropriate
   - Refactored key methods in blog_data.rs to use functional approaches
   - Refactored db/repository.rs to use functional approaches
   - Eliminated side effects in data processing functions in db/repository.rs

## Next Priorities

The following tasks should be prioritized next, with a focus on deployment:

1. **Implement CI/CD Pipeline**:
   - [ ] Set up GitHub Actions for automated testing
   - [ ] Configure automated deployment to GitHub Pages for the CV site
   - [ ] Configure automated deployment to a server for the blog API
   - [ ] Add status badges to README.md

2. **Enhance Deployment Process**:
   - [ ] Improve Docker configuration for production deployment
   - [ ] Create deployment documentation
   - [ ] Add health checks and monitoring
   - [ ] Implement zero-downtime deployment

3. **Enhance Error Handling**:
   - [ ] Add proper error recovery mechanisms
     - [ ] Implement retry logic for database operations
     - [ ] Add transaction rollback and recovery for write operations
     - [ ] Add fallback mechanisms for read operations
     - [ ] Implement circuit breaker pattern
     - [ ] Add rate limiting
     - [ ] Implement graceful degradation

4. **Improve Test Coverage**:
   - [ ] Add comprehensive integration tests
   - [ ] Implement proper mocks for external dependencies
   - [ ] Add tests for edge cases and error conditions

## Medium-Term Goals

1. **Deployment and Infrastructure Improvements**:
   - Move to a full-async SQLite implementation (sqlx or tokio-rusqlite)
   - Improve performance through caching and optimized database queries
   - Add metrics and monitoring for production deployments
   - Implement a comprehensive documentation system with examples
   - Set up automated backups and disaster recovery

2. **Feature Enhancements**:
   - Implement a proper admin interface for the blog
   - Add support for more advanced blog features like comments and social sharing
   - Implement a proper theme system for the blog frontend
   - Explore alternative database backends (PostgreSQL, MongoDB)
   - Implement a plugin system for extensibility

## Deployment Options

### CV Site Deployment

#### Option 1: Traditional Web Hosting
1. Run `cargo run --release` to generate the production build
2. Upload all contents of the `dist/` directory to your web hosting service
3. Ensure your server is configured to use the provided configuration files

#### Option 2: GitHub Pages
1. Create a GitHub repository for your website
2. Add a GitHub Actions workflow to build and deploy your site:
   ```yaml
   name: Build and Deploy
   on:
     push:
       branches: [ main ]
   jobs:
     build-and-deploy:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - name: Install Rust
           uses: actions-rs/toolchain@v1
           with:
             toolchain: stable
             override: true
         - name: Install Typst
           run: cargo install typst-cli
         - name: Build site
           run: cargo run --release
         - name: Deploy to GitHub Pages
           uses: JamesIves/github-pages-deploy-action@4.1.4
           with:
             branch: gh-pages
             folder: dist
   ```

#### Option 3: Netlify
1. Connect your GitHub repository to Netlify
2. Set the build command to:
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source $HOME/.cargo/env && cargo install typst-cli && cargo run --release
   ```
3. Set the publish directory to `dist`

### Blog API Server Deployment

#### Option 1: Using Docker
The easiest way to deploy is with Docker:
```
./deploy.sh
```

Or manually:
```
docker-compose up -d
```

#### Option 2: Manual Deployment
1. Build the release binary: `cargo build --release`
2. Copy the binary and static assets to your server
3. Run the binary: `./blog_api_server`

## Implementation Strategy for Deployment Focus

1. Start with setting up a CI/CD pipeline using GitHub Actions:
   - Configure automated testing for all pull requests
   - Set up automated deployment to GitHub Pages for the CV site
   - Configure automated deployment to a server for the blog API

2. Enhance the Docker configuration for production deployment:
   - Optimize the Dockerfile for smaller image size and faster builds
   - Add health checks and monitoring
   - Configure proper logging and error reporting

3. Improve the deployment documentation:
   - Create detailed deployment guides for different environments
   - Document the CI/CD pipeline and how to use it
   - Add troubleshooting information

4. Implement zero-downtime deployment:
   - Configure proper container orchestration
   - Set up load balancing
   - Implement rolling updates

5. In parallel, continue enhancing error handling and improving test coverage to ensure the application is robust and reliable in production.

## Success Criteria Progress

- ✅ All tests pass consistently
- ✅ No database locking issues
- ✅ All operations are properly functional and immutable in blog_data.rs and db/repository.rs
- ⏳ Performance is improved (in progress)
- ⏳ Code is simpler and easier to understand (in progress)
- ⏳ CI/CD pipeline is implemented (not started)
- ⏳ Zero-downtime deployment is configured (not started)
- ⏳ Monitoring and alerting are set up (not started)

## How to Contribute

If you're picking up work on this project, focus on the "Next Priorities" section first. The highest priority is now implementing a CI/CD pipeline and enhancing the deployment process to ensure the application can be deployed reliably and efficiently.

For detailed information about the implementation plan for each priority, please refer to the sections above.