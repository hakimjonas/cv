# Remaining Issues and Refactoring Plan

## Overview

This document outlines the remaining issues in the codebase after the initial refactoring to a Git-based authentication system. The refactoring has successfully removed deprecated files and updated some references, but there are still significant structural issues that need to be addressed.

## Current Status

- Removed deprecated files:
  - `src/credentials.rs`
  - `src/blog_api_test.rs`
  - `src/blog_manager_test.rs`
  - `src/blog_manager_test_async.rs`
  - `/home/hakim/personal/cv/tests/performance/blog_api_performance_test.rs`
  - `/home/hakim/personal/cv/tests/e2e/blog_api_e2e_test.rs`
  - `src/blog_data_async.rs`

- Fixed references to removed modules:
  - Updated `src/unified_config.rs` to remove references to `credentials` module
  - Disabled rate limiter in `src/simple_blog_api.rs`

- Fixed basic syntax errors:
  - Removed unused imports in `src/simple_auth.rs`
  - Removed unused imports in `src/simple_auth_middleware.rs`
  - Added comment to explain async/await usage in `extract_and_validate_token`
  - Removed unused imports in `src/simple_blog_api.rs`

## Major Structural Issues

### 1. Data Structure Mismatches

The `BlogPost` and `Tag` structures are defined differently in different parts of the codebase:

- In `src/blog_data.rs`:
  ```rust
  pub struct BlogPost {
      pub id: Option<i64>,
      pub title: String,
      pub slug: String,
      pub date: String,
      pub user_id: Option<i64>,
      pub author: String,
      pub excerpt: String,
      pub content: String,
      pub content_format: ContentFormat,
      pub published: bool,
      pub featured: bool,
      pub image: Option<String>,
      pub tags: Vector<Tag>,
      pub metadata: im::HashMap<String, String>,
  }
  ```

- In `src/db/repository.rs`:
  ```rust
  pub struct BlogPost {
      pub id: Option<i64>,
      pub title: String,
      pub slug: String,
      pub date: String,
      pub user_id: Option<i64>,
      pub author: String,
      pub excerpt: String,
      pub content: String,
      pub published: bool,
      pub featured: bool,
      pub image: Option<String>,
      pub tags: Vector<Tag>,
      pub metadata: HashMap<String, String>,
  }
  ```

- In `src/simple_blog_api.rs` (conversion functions), it's trying to access fields that don't exist:
  - `date_created`
  - `date_updated`
  - `image_url`
  - `reading_time`

### 2. Function Signature Mismatches

- In `src/image_api.rs`, the `create_image_api_router` function has a different signature than what's expected in `src/simple_blog_api.rs`.
- In `src/feature_flags.rs`, the `FeatureFlags::new()` function requires parameters that aren't provided in `src/simple_blog_api.rs`.
- In `src/feed.rs`, the `generate_rss_feed` and `generate_atom_feed` functions have different parameter orders than what's expected in `src/simple_blog_api.rs`.

### 3. Middleware Compatibility Issues

- The middleware functions in `src/simple_auth_middleware.rs` have compatibility issues with the Axum router in `src/simple_blog_api.rs`.

## Recommended Fixes

### 1. Update Data Structures

1. Decide on a single, consistent definition for `BlogPost` and `Tag` structures.
2. Update all references to these structures throughout the codebase.
3. Update the conversion functions in `src/simple_blog_api.rs` to match the actual structure of the data.

### 2. Fix Function Signatures

1. Update the `create_image_api_router` function in `src/image_api.rs` to match the expected signature.
2. Update the `FeatureFlags::new()` call in `src/simple_blog_api.rs` to provide the required parameters.
3. Update the calls to `generate_rss_feed` and `generate_atom_feed` in `src/simple_blog_api.rs` to match the expected parameter order.

### 3. Fix Middleware Compatibility

1. Update the middleware functions in `src/simple_auth_middleware.rs` to be compatible with the Axum router.
2. Ensure that the middleware functions are properly integrated with the Git-based authentication system.

## Core Functionality to Preserve

- Git-based authentication system
- Blog post management (create, read, update, delete)
- CV generation and display
- Project showcase

## Files Needing Complete Rewrites

- `src/simple_blog_api.rs`: Update conversion functions to match actual data structures
- `src/image_api.rs`: Fix function signature issues

## Prioritized Fixes

1. Update `BlogPost` and `Tag` conversion functions in `src/simple_blog_api.rs`
2. Fix function signatures in `src/image_api.rs`
3. Update middleware implementations to match new authentication system
4. Fix remaining type mismatches and trait implementation errors

## Conclusion

The codebase has been partially refactored to use a Git-based authentication system, but there are still significant structural issues that need to be addressed. The recommended fixes outlined in this document will help to complete the refactoring and ensure that the codebase is consistent and maintainable.