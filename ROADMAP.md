# Project Roadmap

This document outlines the current state of the project, ongoing refactoring efforts, and the plan for future development.

## Project Status

The project currently includes:

1. A dynamic CV generator that produces HTML and PDF outputs from a single data source
2. A blog system with a RESTful API for creating, reading, updating, and deleting posts
3. GitHub integration for fetching repository information
4. Docker support for easy deployment

## Completed Work

1. ✅ Created a comprehensive refactoring plan
2. ✅ Fixed duplicate dependencies in Cargo.toml
3. ✅ Created a new database module with proper connection pooling
4. ✅ Implemented custom error types for database operations
5. ✅ Created a migrations system for database schema
6. ✅ Implemented a repository layer for blog operations
7. ✅ Updated the Blog API Module to use the new repository layer
8. ✅ Updated the Blog Tester to use the new database architecture
9. ✅ Fixed test script to properly handle dynamic port allocation
10. ✅ Cleaned up unused imports and variables
11. ✅ Implemented proper logging with tracing throughout the codebase

## Immediate Next Steps

### 1. ✅ Add Proper Logging

Implement proper logging throughout the codebase using the tracing crate:

- ✅ Set up a global subscriber for logging in main binary entry points
- ✅ Add structured logging to key operations
- ✅ Ensure errors are properly logged with context

### 2. ✅ Implement Property-Based Testing

Add property-based testing for the core data model and operations:

- ✅ Create generators for BlogPost, Tag, and other data types
- ✅ Test invariants such as serialization/deserialization roundtrips
- ✅ Test idempotency of operations like saving and updating posts

## Next Priorities

Now that the immediate next steps have been completed, the following tasks should be prioritized:

### 1. Enhance Error Handling

- ✅ Implement proper error types instead of using `anyhow` everywhere
- ✅ Use Result monads consistently throughout the codebase
- Add proper error recovery mechanisms

### 2. Complete Functional Programming Transition

- Ensure all data structures use `im` crate's immutable collections
- ✅ Replace mutable operations with functional transformations in blog_data.rs
  - ✅ Refactor with_added_tag
  - ✅ Refactor with_removed_tag
  - ✅ Refactor with_added_metadata
  - ✅ Refactor with_removed_metadata
- Replace remaining mutable operations with functional transformations
- Eliminate side effects in data processing functions

### 3. Improve Test Coverage

- Add comprehensive integration tests
- Implement proper mocks for external dependencies
- Add tests for edge cases and error conditions

## Medium-Term Goals

1. Implement a proper CI/CD pipeline with automated testing and deployment
2. Move to a full-async SQLite implementation (sqlx or tokio-rusqlite)
3. Improve performance through caching and optimized database queries
4. Add metrics and monitoring for production deployments
5. Implement a comprehensive documentation system with examples

## Long-Term Goals

1. Implement a proper admin interface for the blog
2. Add support for more advanced blog features like comments and social sharing
3. Implement a proper theme system for the blog frontend
4. Explore alternative database backends (PostgreSQL, MongoDB)
5. Implement a plugin system for extensibility

## Refactoring Goals

### 1. Return to Functional Programming Principles

- Ensure all data structures use `im` crate's immutable collections
- Replace all mutable operations with functional transformations
- Eliminate all side effects in data processing functions
- Clearly separate pure functions from IO operations
- Implement property-based testing for all pure functions ✅

### 2. Improve Database Architecture

- ✅ Replace ad-hoc SQLite setup with a proper connection pool
- Move to fully async database operations using `tokio-rusqlite` or `sqlx`
- ✅ Implement proper migration system for schema changes
- ✅ Create a clean repository layer that abstracts database operations

### 3. Enhance Error Handling

- ✅ Implement proper error types instead of using `anyhow` everywhere
- ✅ Use Result monads consistently throughout the codebase
- Add proper error recovery mechanisms
- Improve error reporting and diagnostics
- ✅ Implement structured logging for errors

### 4. Strengthen Testing

- ✅ Implement property-based testing for pure functions
- ✅ Create isolated test databases that don't interfere with each other
- Add proper mocks for external dependencies
- Implement integration tests that don't rely on network connectivity

## Success Criteria

1. ✅ All tests pass consistently
2. ✅ No database locking issues
3. All operations are properly functional and immutable
4. Performance is improved
5. Code is simpler and easier to understand

## How to Contribute

If you're working on this project, please follow these guidelines:

1. Create a branch for each logical unit of work
2. Write tests for all new code
3. Ensure all tests pass before submitting a PR
4. Document all public APIs using rustdoc
5. Add proper logging to all functions using the tracing crate
