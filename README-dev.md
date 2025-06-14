# Development Guidelines

This document provides detailed development guidelines for the project, including functional programming principles, best practices, and coding standards.

## Table of Contents

- [Development Philosophy](#development-philosophy)
- [Functional Programming Principles](#functional-programming-principles)
- [Code Organization](#code-organization)
- [Testing Strategy](#testing-strategy)
- [Error Handling](#error-handling)
- [Logging](#logging)
- [Database Access](#database-access)
- [Performance Considerations](#performance-considerations)
- [Contributing Guidelines](#contributing-guidelines)

## Development Philosophy

This project follows a functional programming approach with a focus on:

- **Immutability**: Using immutable data structures to prevent side effects
- **Pure Functions**: Functions that don't modify state and return the same output for the same input
- **Type Safety**: Leveraging Rust's type system to catch errors at compile time
- **Error Handling**: Using Result and Option types for comprehensive error handling
- **Testability**: Writing code that is easy to test and maintain

## Functional Programming Principles

### Immutable Data Structures

We use the `im` crate for immutable data structures. When working with collections:

```
// Prefer this:
use im::{Vector, HashMap};

let mut vec = Vector::new();
let new_vec = vec.push_back(item); // Returns a new vector

// Over this:
let mut vec = Vec::new();
vec.push(item); // Modifies the original vector
```

### Pure Functions

Functions should be pure whenever possible:

```
// Prefer this:
fn add_tag(post: &BlogPost, tag: Tag) -> BlogPost {
    let mut tags = post.tags.clone();
    let new_tags = tags.push_back(tag);
    BlogPost {
        tags: new_tags,
        ..post.clone()
    }
}

// Over this:
fn add_tag(post: &mut BlogPost, tag: Tag) {
    post.tags.push(tag);
}
```

### Function Composition

Use function composition to build complex operations:

```
// Prefer this:
let result = data
    .iter()
    .filter(|item| item.is_valid())
    .map(|item| transform(item))
    .collect::<Vector<_>>();

// Over this:
let mut result = Vector::new();
for item in data.iter() {
    if item.is_valid() {
        result.push_back(transform(item));
    }
}
```

## Code Organization

### Project Structure

The project is organized into the following modules:

- `cv_data`: Data models for the CV
- `html_generator`: HTML generation logic
- `typst_generator`: PDF generation logic
- `blog_data`: Data models for the blog
- `db`: Database access layer
  - `repository`: Repository pattern implementation
  - `migrations`: Database schema migrations
  - `error`: Custom error types for database operations

### Module Guidelines

- Each module should have a clear, single responsibility
- Public interfaces should be well-documented
- Implementation details should be private when possible
- Use feature flags for optional functionality

## Testing Strategy

### Test Types

The project uses several types of tests:

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test the API endpoints
3. **Property-Based Tests**: Test with randomly generated inputs
4. **Idempotency Tests**: Ensure operations can be applied multiple times with the same result

### Writing Tests

```
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_tag() {
        let post = BlogPost::new("title", "content");
        let tag = Tag::new("rust");
        
        let updated_post = add_tag(&post, tag.clone());
        
        assert!(updated_post.tags.contains(&tag));
        assert_eq!(updated_post.tags.len(), 1);
        
        // Original post should be unchanged
        assert_eq!(post.tags.len(), 0);
    }
}
```

### Property-Based Testing

We use property-based testing for complex operations:

```
#[test]
fn property_serialization_roundtrip() {
    proptest!(|(post in blog_post_generator())| {
        let json = serde_json::to_string(&post).unwrap();
        let deserialized: BlogPost = serde_json::from_str(&json).unwrap();
        assert_eq!(post, deserialized);
    });
}
```

## Error Handling

### Custom Error Types

Define custom error types for each module:

```
#[derive(Debug, thiserror::Error)]
pub enum BlogError {
    #[error("Post not found: {0}")]
    PostNotFound(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}
```

### Error Propagation

Use the `?` operator for error propagation:

```
fn get_post(slug: &str) -> Result<BlogPost, BlogError> {
    let conn = establish_connection()?;
    let post = repository.find_by_slug(&conn, slug)?;
    
    if !post.is_published {
        return Err(BlogError::ValidationError("Post is not published".to_string()));
    }
    
    Ok(post)
}
```

## Logging

Use structured logging with the `tracing` crate:

```
use tracing::{info, error, instrument};

#[instrument(skip(conn))]
fn create_post(conn: &Connection, post: BlogPost) -> Result<(), BlogError> {
    info!(slug = %post.slug, "Creating new blog post");
    
    match repository.create(conn, &post) {
        Ok(_) => {
            info!(slug = %post.slug, "Blog post created successfully");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, slug = %post.slug, "Failed to create blog post");
            Err(e.into())
        }
    }
}
```

## Database Access

### Connection Pooling

Use connection pooling for database access:

```
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

lazy_static! {
    static ref POOL: Pool<SqliteConnectionManager> = {
        let manager = SqliteConnectionManager::file("blog.db");
        Pool::new(manager).expect("Failed to create pool")
    };
}

fn get_connection() -> Result<r2d2::PooledConnection<SqliteConnectionManager>, BlogError> {
    POOL.get().map_err(|e| BlogError::ConnectionPoolError(e.to_string()))
}
```

### Repository Pattern

Use the repository pattern for database access:

```
pub trait BlogRepository {
    fn find_all(&self, conn: &Connection) -> Result<Vector<BlogPost>, BlogError>;
    fn find_by_slug(&self, conn: &Connection, slug: &str) -> Result<BlogPost, BlogError>;
    fn create(&self, conn: &Connection, post: &BlogPost) -> Result<(), BlogError>;
    fn update(&self, conn: &Connection, post: &BlogPost) -> Result<(), BlogError>;
    fn delete(&self, conn: &Connection, slug: &str) -> Result<(), BlogError>;
}
```

## Performance Considerations

### Lazy Evaluation

Use lazy evaluation when processing large collections:

```
// Prefer this:
let result = data
    .iter()
    .filter(|item| expensive_check(item))
    .take(10)
    .collect::<Vector<_>>();

// Over this:
let filtered = data
    .iter()
    .filter(|item| expensive_check(item))
    .collect::<Vector<_>>();
let result = filtered.iter().take(10).collect::<Vector<_>>();
```

### Caching

Use caching for expensive operations:

```
use once_cell::sync::Lazy;
use std::sync::Mutex;

static CACHE: Lazy<Mutex<HashMap<String, BlogPost>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn get_post_with_cache(slug: &str) -> Result<BlogPost, BlogError> {
    let cache = CACHE.lock().unwrap();
    
    if let Some(post) = cache.get(slug) {
        return Ok(post.clone());
    }
    
    let post = get_post(slug)?;
    
    let mut cache = CACHE.lock().unwrap();
    cache.insert(slug.to_string(), post.clone());
    
    Ok(post)
}
```

## Contributing Guidelines

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and ensure they pass
5. Submit a pull request

### Code Review Checklist

- Does the code follow the functional programming principles?
- Are there appropriate tests?
- Is error handling comprehensive?
- Is the code well-documented?
- Does the code follow the project's coding style?
- Are there any performance concerns?

### Commit Message Format

Use the following format for commit messages:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Where `<type>` is one of:
- feat: A new feature
- fix: A bug fix
- docs: Documentation only changes
- style: Changes that do not affect the meaning of the code
- refactor: A code change that neither fixes a bug nor adds a feature
- perf: A code change that improves performance
- test: Adding missing tests or correcting existing tests
- chore: Changes to the build process or auxiliary tools