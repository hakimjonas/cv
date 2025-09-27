# 🛠️ Developer Guide

> A comprehensive guide for contributing to and extending the CV Generator

## 📋 Table of Contents

- [Architecture Overview](#architecture-overview)
- [Development Setup](#development-setup)
- [Code Organization](#code-organization)
- [Performance Guidelines](#performance-guidelines)
- [Contributing Guidelines](#contributing-guidelines)
- [Testing Strategy](#testing-strategy)
- [Release Process](#release-process)

## 🏗️ Architecture Overview

The CV Generator follows functional programming principles with immutable data structures and pure functions wherever possible.

### Core Principles

1. **Immutability**: Use `im` crate collections (`Vector`, `HashMap`) for all data structures
2. **Pure Functions**: Minimize side effects, prefer functions that return new values
3. **Modular Design**: Each module has a single responsibility
4. **Performance First**: Sub-second builds with intelligent caching
5. **Type Safety**: Leverage Rust's type system for correctness

### Architecture Diagram

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Data Layer    │    │  Business Logic │    │  Presentation   │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • cv_data.rs    │───▶│ • github.rs     │───▶│ • html_generator │
│ • site_config   │    │ • performance   │    │ • typst_generator│
│ • github_cache  │    │ • css_generator │    │ • templates/     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                ▲
                                │
                       ┌─────────────────┐
                       │  Infrastructure │
                       ├─────────────────┤
                       │ • github_cache  │
                       │ • performance   │
                       │ • asset_processor│
                       └─────────────────┘
```

## 🚀 Development Setup

### Prerequisites

```bash
# Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install GitHub CLI
# macOS: brew install gh
# Ubuntu: sudo apt install gh
# Windows: winget install GitHub.cli

# Install Typst for PDF generation
cargo install typst-cli

# Authenticate with GitHub
gh auth login
```

### Local Development

```bash
# Clone and setup
git clone https://github.com/hakimjonas/cv.git
cd cv

# Build the project
cargo build

# Run with debug logging
RUST_LOG=debug cargo run --bin cv

# Run tests
cargo test

# Run with performance profiling
cargo run --bin cv 2>&1 | grep "📊"
```

## 📁 Code Organization

### Module Structure

```
src/
├── lib.rs                    # Public API and documentation
├── main.rs                   # CLI entry point
├── cv_data.rs                # Core data structures
├── github.rs                 # GitHub API integration
├── github_cache.rs           # Caching system (77% perf improvement)
├── performance.rs            # Profiling and optimization
├── css_generator.rs          # Dynamic CSS generation
├── html_generator/           # HTML generation system
│   ├── mod.rs               # Main coordination (264 lines)
│   ├── html_generators.rs   # Core HTML pages (281 lines)
│   ├── config_generators.rs # Server configs (441 lines)
│   ├── asset_processor.rs   # Asset handling (218 lines)
│   └── utils.rs             # Utilities (156 lines)
├── typst_generator/          # PDF generation
├── blog_posts.rs             # Blog system with tagging
├── markdown_pages.rs         # Static page generation
└── site_config.rs            # Configuration management
```

### Module Guidelines

1. **Line Count**: Keep modules under 500 lines (current max: 441 lines)
2. **Single Responsibility**: Each module should have one clear purpose
3. **Documentation**: Every public function needs doc comments with examples
4. **Error Handling**: Use `anyhow::Result` for error propagation
5. **Testing**: Include unit tests for all business logic

## ⚡ Performance Guidelines

### Current Performance Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **Build Time** | <5s | 524ms | ✅ Excellent |
| **GitHub API** | <500ms | 0ms (cached) | ✅ Perfect |
| **CSS Generation** | <100ms | ~50ms | ✅ Excellent |
| **HTML Generation** | <1s | ~200ms | ✅ Excellent |

### Performance Best Practices

#### 1. Use the Performance Module

```rust
use crate::performance::BuildProfiler;

let mut profiler = BuildProfiler::new();

profiler.time_operation("operation_name", || {
    // Your code here
});

profiler.print_summary(); // Always profile new features
```

#### 2. Cache Everything Possible

```rust
// Example: Configuration-based cache invalidation
fn needs_regeneration(config: &Config, output_path: &str) -> Result<bool> {
    let config_hash = calculate_config_hash(config);
    let existing_hash = read_hash_from_file(output_path)?;
    Ok(config_hash != existing_hash)
}
```

#### 3. Use Immutable Data Structures

```rust
use im::Vector;

// ✅ Good: Immutable, efficient sharing
let mut projects = Vector::new();
projects.push_back(project);

// ❌ Avoid: Mutable Vec when sharing data
let mut projects = Vec::new();
```

#### 4. Parallel Processing

```rust
use rayon::prelude::*;

// Process multiple items in parallel
let results: Vec<_> = items
    .par_iter()
    .map(|item| process_item(item))
    .collect();
```

### Performance Monitoring

Always run performance tests when making changes:

```bash
# Build with profiling enabled
cargo run --bin cv

# Look for the performance summary
# Target: Total build time < 1000ms
```

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_functionality() {
        let mut cache = GitHubCache::default();
        cache.cache_projects("user", Vector::new());
        assert!(cache.get_projects("user").is_some());
    }
}
```

### Integration Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test github_cache

# Run with output
cargo test -- --nocapture
```

### Performance Tests

```rust
#[test]
fn test_build_performance() {
    let start = std::time::Instant::now();

    // Run build process
    generate_html(&cv, &config, "test_output.html").unwrap();

    let duration = start.elapsed();
    assert!(duration.as_millis() < 5000, "Build took too long: {}ms", duration.as_millis());
}
```

## 🤝 Contributing Guidelines

### Code Style

1. **Formatting**: Use `cargo fmt` before committing
2. **Linting**: Run `cargo clippy` and fix all warnings
3. **Documentation**: Document all public APIs with examples
4. **Testing**: Add tests for new functionality
5. **Performance**: Profile any changes that could affect build time

### Pull Request Process

1. **Branch Naming**: Use descriptive names (`feature/cache-optimization`, `fix/memory-leak`)
2. **Commit Messages**: Follow conventional commits format
3. **Testing**: Ensure all tests pass locally
4. **Performance**: Include performance impact in PR description
5. **Documentation**: Update relevant documentation

### Example PR Checklist

- [ ] Code follows project style guidelines
- [ ] All tests pass locally
- [ ] New code has appropriate test coverage
- [ ] Documentation updated for API changes
- [ ] Performance impact measured and documented
- [ ] No new compiler warnings
- [ ] Commit messages are clear and descriptive

## 🔄 Release Process

### Version Management

We follow semantic versioning (SemVer):

- **MAJOR**: Breaking changes to public API
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, performance improvements

### Release Steps

1. **Update Version**: Bump version in `Cargo.toml`
2. **Update Changelog**: Document all changes
3. **Performance Check**: Ensure build time still <1s
4. **Create Tag**: `git tag v1.2.3`
5. **Push Release**: GitHub Actions handles deployment

### Performance Benchmarks

Before each release, verify performance targets:

```bash
# Run multiple builds and average the times
for i in {1..5}; do
    echo "Build $i:"
    time cargo run --bin cv
done
```

Target metrics:
- **Average build time**: <1000ms
- **Cache hit rate**: >90%
- **Memory usage**: <100MB peak

## 🐛 Debugging

### Common Issues

#### 1. GitHub API Rate Limiting

```bash
# Check auth status
gh auth status

# Login if needed
gh auth login
```

#### 2. Cache Issues

```bash
# Clear cache
rm -rf cache/

# Rebuild from scratch
cargo run --bin cv
```

#### 3. Performance Degradation

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin cv

# Look for timing information
# Check for cache misses
```

### Debug Logging

```rust
use log::{debug, info, warn, error};

debug!("Processing {} projects", projects.len());
info!("✅ Build completed in {}ms", duration.as_millis());
warn!("⚠️ Cache miss for user: {}", username);
error!("❌ Failed to fetch data: {}", error);
```

## 📚 Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust fundamentals
- [Askama Templates](https://djc.github.io/askama/) - Template engine documentation
- [im crate](https://docs.rs/im/) - Immutable data structures
- [anyhow](https://docs.rs/anyhow/) - Error handling best practices

## 🎯 Performance Targets

Our 10/10 performance targets:

| Metric | Target | Current Status |
|--------|--------|----------------|
| **Build Time** | <5s | ✅ 524ms (90% under target) |
| **First Paint** | <1s | ✅ ~200ms |
| **GitHub Integration** | <2s | ✅ 0ms (cached) |
| **Memory Usage** | <100MB | ✅ ~50MB |
| **Bundle Size** | <500KB | ✅ ~300KB |

---

*Built with ❤️ in Rust following functional programming principles*