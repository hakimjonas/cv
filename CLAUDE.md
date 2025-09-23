# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based personal website with a dynamic CV generator and blog system. The project consists of two main components:
1. **CV generator and static website** (main application) - Generates static HTML/PDF from JSON data
2. **Blog API server** - REST API with SQLite backend for blog functionality

The codebase uses functional programming principles with immutable data structures (`im` crate) and follows a modular architecture with clear separation between data, business logic, and presentation layers.

## Development Commands

### Building and Running
```bash
# Main CV application (generates static website)
cargo run                    # Development build
cargo run --release          # Production build

# Blog API server
cargo run --bin simple_blog_api_server    # Start blog API server

# Individual test utilities
cargo run --bin blog_tester               # Test blog functionality
cargo run --bin security_test             # Run security tests
cargo run --bin test_blog_core            # Test blog core
```

### Testing
```bash
# Comprehensive test suite (ALWAYS run this before committing)
./scripts/test.sh            # Runs all tests including unit, integration, and API tests

# Individual test groups
cargo test                   # Unit tests
cargo test --lib             # Library tests only
cargo test blog_data_test    # Blog data tests
cargo test --test blog_property_test  # Property-based tests
cargo clippy                 # Code quality checks (required)
cargo fmt                    # Code formatting (required)

# Security and validation
cargo run --bin security_test # Run security validation tests
```

### Local Development Environment
```bash
# Docker-based local development
./scripts/deploy-local.sh start     # Start development environment on port 3002
./scripts/deploy-local.sh stop      # Stop development environment
./scripts/deploy-local.sh logs      # View logs
./scripts/deploy-local.sh status    # Check status
./scripts/deploy-local.sh rebuild   # Rebuild application
```

### Deployment
```bash
# Local deployment script
./scripts/deploy-local.sh start

# Other deployment scripts available
./scripts/deploy.sh          # Production deployment
./scripts/deploy-aws.sh      # AWS deployment
```

## Architecture

### Code Organization

The codebase follows a modular Rust architecture with functional programming principles:

**Core Application Layer:**
- **Main Application** (`src/main.rs`): CV generator entry point with async/await patterns
- **Library Interface** (`src/lib.rs`): Public API exposing all modules
- **Configuration** (`src/unified_config.rs`, `src/config.rs`): Multi-source configuration management

**Data and Storage Layer:**
- **CV Data Models** (`src/cv_data.rs`): Core data structures using `im` for immutability
- **Database Layer** (`src/db/`): Repository pattern with connection pooling (`r2d2`)
- **Migration System** (`src/migrate.rs`): Database schema migrations and data transformation

**Blog System:**
- **Blog API** (`src/simple_blog_api.rs`, `src/blog_api.rs`): REST endpoints with Axum
- **Blog Data** (`src/blog_data.rs`): Blog post models and validation
- **Authentication** (`src/auth.rs`, `src/simple_auth.rs`): Git-based auth system

**Generation and Output:**
- **HTML Generator** (`src/html_generator.rs`): Askama template-based HTML generation
- **PDF Generator** (`src/typst_generator.rs`): Typst-based PDF generation
- **Asset Processing** (`src/bundler.rs`): Static asset bundling and optimization

**Security and Middleware:**
- **CSRF Protection** (`src/csrf_protection.rs`): Cross-site request forgery protection
- **Content Security Policy** (`src/content_security_policy.rs`): CSP header management
- **Rate Limiting** (`src/rate_limiter.rs`): API rate limiting implementation

**External Integrations:**
- **GitHub Integration** (`src/github.rs`, `src/github_cache.rs`): GitHub API client with caching
- **Git Identity** (`src/git_identity.rs`): Git configuration extraction for authentication

### Key Technologies

- **Axum** web framework for the API server
- **SQLite** database with Rusqlite
- **Askama** templating engine for HTML generation
- **Serde** for JSON serialization
- **Git2** for Git-based authentication
- **Tokio** async runtime

### Authentication System

The project uses a unique Git-based authentication system that derives user identity from Git configuration rather than traditional user/password authentication. Key components:

- `src/auth.rs`: Core authentication logic
- `src/git_identity.rs`: Git identity extraction
- `src/credentials.rs`: Credential management
- Configuration via `config.toml` or environment variables

### Database

SQLite database with:
- Blog posts and metadata
- User authentication data derived from Git
- Migration system in `src/migrate.rs`
- Repository pattern in `src/db/`

### Configuration

The application supports multiple configuration sources:
- `config.toml` (primary configuration file)
- Environment variables
- Git configuration (for user identity)
- Command-line arguments

Key configuration files:
- `config.toml.template`: Template for configuration
- `.env.local`: Local development environment variables

### Security Features

- CSRF protection (`src/csrf_protection.rs`)
- Content Security Policy (`src/content_security_policy.rs`)
- Rate limiting (`src/rate_limiter.rs`)
- Input validation and sanitization

## Development Workflow

### Essential Pre-commit Steps
1. **Testing**: ALWAYS run `./scripts/test.sh` before committing
2. **Code Quality**: Run `cargo clippy` and `cargo fmt` (required)
3. **Security**: Run `cargo run --bin security_test` for security validation

### Setup Process
1. **Configuration**: Copy `config.toml.template` to `config.toml` and configure
2. **Local Environment**: Use `./scripts/deploy-local.sh start` for full Docker-based development
3. **Database**: Use `--migrate-to-db` flag to convert JSON data to SQLite if needed

### Development Patterns
- **Functional Programming**: Use immutable data structures from `im` crate
- **Error Handling**: Use `anyhow::Result` for most functions, `thiserror` for custom errors
- **Async/Await**: Main application and blog API use Tokio async runtime
- **Configuration**: Support multiple sources (TOML, env vars, Git config, CLI args)
- **Database**: Use repository pattern with connection pooling via `r2d2`

## Port Configuration

- Main application: Configurable (default 3002 for local dev)
- Blog API: Port 3000 (production) or 3002 (local dev)
- Health checks available at `/health` endpoint

## File Structure Notes

- `templates/`: Askama HTML templates for web pages
- `static/`: Static assets (CSS, JS, images)
- `data/`: CV data in JSON format and database files
- `docker/`: Docker configuration for different environments
- `scripts/`: Deployment and testing scripts
- `src/bin/`: Additional binary utilities (blog_tester, security_test, etc.)

## Important Implementation Details

### Binary Targets
The project includes several binary targets in `src/bin/`:
- `simple_blog_api_server.rs`: Main blog API server
- `blog_tester.rs`: Blog functionality testing utility
- `security_test.rs`: Security validation tests
- `test_blog_core.rs`: Core blog functionality tests

### Configuration System
The configuration system supports hierarchical loading:
1. `config.toml` file (primary)
2. Environment variables
3. Git configuration (for user identity)
4. Command-line arguments (highest priority)

### Authentication Architecture
Uses Git-based authentication system that derives user identity from Git configuration rather than traditional credentials. This unique approach integrates tightly with developer workflows.

### Data Models
- CV data uses immutable `Vector` types from `im` crate
- Blog data uses traditional Rust structs with Serde serialization
- Database operations use repository pattern with connection pooling

### Security Implementation
- CSRF protection using `axum_csrf`
- Content Security Policy headers
- Rate limiting for API endpoints
- Input validation and sanitization using `ammonia`
- Secure credential storage using `keyring` crate

# DevHub Context for AI Agents

This project uses DevHub for intelligent project analysis and context management.

## Available DevHub Tools (via MCP)

When working in this project, you have access to these DevHub tools:

1. **get-bundle-context** - Get complete project context including:
   - Project type and frameworks
   - File analysis with importance scoring
   - Tech stack detection
   - Development patterns
   - Git history and recent commits

2. **get-current-branch-context** - Auto-detect context from current git branch

3. **get-jira-issue** - Retrieve Jira ticket details (if configured)

4. **get-pr-details** - Get GitHub/GitLab PR information

5. **get-pr-comments** - Fetch unresolved PR review comments

## Project Intelligence

DevHub automatically analyzes:
- Project type (web app, library, CLI tool, etc.)
- Frameworks (React, Django, FastAPI, etc.)
- Tech stack (languages, databases, tools)
- Code organization patterns
- Testing approach
- Development stage

## Usage

You don't need to ask the user about project structure - use DevHub tools to understand it automatically.

Example: Instead of asking "What framework does this project use?", call get-bundle-context to know instantly.

## Key Development Guidelines

### Code Quality Requirements
- **ALWAYS** run `./scripts/test.sh` before committing changes
- **ALWAYS** run `cargo clippy` and fix all warnings before committing
- **ALWAYS** run `cargo fmt` to format code consistently
- Run `cargo run --bin security_test` to validate security implementations

### Testing Strategy
- Use property-based testing with `proptest` for data validation
- Test both async and sync code paths
- Include integration tests for API endpoints
- Test security features (CSRF, rate limiting, input validation)
- Use the `blog_tester` binary for end-to-end blog functionality testing

### Architecture Principles
- Use functional programming patterns with immutable data (`im` crate)
- Implement proper error handling with `anyhow::Result` and `thiserror`
- Use the repository pattern for database access
- Implement comprehensive security middleware
- Support multiple configuration sources with hierarchical precedence

### Security Best Practices
- Never commit secrets or tokens to the repository
- Use the Git-based authentication system for user identity
- Implement CSRF protection for state-changing operations
- Apply rate limiting to all API endpoints
- Validate and sanitize all user inputs
- Use Content Security Policy headers
