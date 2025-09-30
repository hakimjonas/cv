# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A high-performance CV/portfolio generator built in Rust that creates beautiful HTML and PDF outputs from JSON data. Designed with functional programming principles, immutable data structures (using the `im` crate), and intelligent caching for sub-second builds.

## Branch Architecture

**CRITICAL**: This project uses a unique two-branch architecture:

- **`main`** - Application code (Rust, templates, CSS, GitHub workflows)
- **`content`** - Personal CV data (JSON files, blog posts, static pages)

**⚠️ NEVER merge `content` into `main` or vice versa** - they remain completely separate.

When working on the codebase:
- Code changes → commit to `main`
- Data changes → user manages in `content` branch (not your concern)

## Build & Run Commands

```bash
# Build the project
cargo build

# Run the generator (uses test data from data/cv_data.json on main branch)
cargo run --bin cv

# Run with debug logging
RUST_LOG=debug cargo run --bin cv

# Run with info logging
RUST_LOG=info cargo run --bin cv

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Check without building
cargo check

# Serve generated site locally (after running cargo run)
cd dist && python3 -m http.server 3000
# Visit http://localhost:3000
```

## Architecture & Key Modules

The codebase follows functional programming principles with clear separation of concerns:

### Core Data Flow
```
cv_data (JSON) → github (fetch projects) → html_generator/typst_generator → dist/
                            ↓
                      github_cache (TTL-based caching for 77% perf improvement)
```

### Module Responsibilities

- **`src/main.rs`** - Main entry point, orchestrates the generation pipeline
- **`src/cv_data.rs`** - Core data structures (Cv, Project, Experience, etc.) using immutable `im::Vector`
- **`src/github.rs`** - GitHub API integration via `gh` CLI (no token management needed)
- **`src/github_cache.rs`** - TTL-based caching system for GitHub API responses (1-hour TTL for projects, 4-hour for avatars)
- **`src/html_generator/`** - HTML generation system (modular, uses Askama templates)
  - `mod.rs` - Main orchestration
  - `html_generators.rs` - Page generation functions
  - `config_generators.rs` - Server config files (.htaccess, robots.txt, etc.)
  - `asset_processor.rs` - Static asset copying
- **`src/typst_generator/`** - PDF generation via Typst
- **`src/css_generator.rs`** - Dynamic CSS generation from color schemes
- **`src/colorscheme_provider.rs`** - Fetches color schemes from multiple sources (iTerm2, GitHub, local)
- **`src/unified_config.rs`** - Centralized configuration management
- **`src/site_config.rs`** - Site-level configuration (menu, fonts, themes)
- **`src/performance.rs`** - Build profiling and performance tracking
- **`src/blog_posts.rs`** - Markdown blog post processing
- **`src/markdown_pages.rs`** - Static page processing

### Templates
- **`templates/`** - Askama templates for HTML generation
- Templates use `.html` extension and are processed at compile time

### Configuration Files
- **`config/site.json`** - Site configuration (menu, fonts, color schemes, Typst theme)
- **`data/cv_data.json`** - Test CV data (on main branch only)
- **`data/language_icons.json`** - Programming language icon mappings

## Key Implementation Details

### GitHub Integration
- Uses `gh` CLI for all GitHub operations (no manual token management)
- Caching is TTL-based: 1 hour for projects, 4 hours for avatars
- Cache stored in `cache/github_cache.json`
- Authentication handled automatically by `gh` CLI

### Performance Optimization
- Intelligent caching achieves 77% performance improvement
- Built-in profiling via `performance::BuildProfiler`
- All operations are timed and reported at the end
- Look for "📊" emoji in logs for performance metrics

### Data Structures
- Use `im::Vector` and `im::HashMap` for immutable collections
- All CV data is immutable once loaded
- Projects can be filtered/transformed functionally

### HTML Generation Pipeline
1. Load CV data from JSON
2. Fetch GitHub projects (with caching)
3. Generate color scheme CSS
4. Generate font CSS
5. Generate all HTML pages (index, cv, projects, blog, static pages)
6. Generate deployment configs (.htaccess, robots.txt, manifest.json, etc.)
7. Optimize CSS/JS assets
8. Copy static assets to dist/

### PDF Generation
- Uses Typst CLI to compile markup to PDF
- Theme configured in `config/site.json` under `typst` key
- Temporary `.typ` file created, compiled, then deleted
- Profile image must be local file for PDF (HTML can use remote URL)

### Testing
- Unit tests in each module (use `#[cfg(test)]` blocks)
- Test files follow standard Rust conventions
- Focus on pure function testing where possible

## Common Development Tasks

### Adding a New CV Section
1. Update `src/cv_data.rs` with new struct fields
2. Update JSON schema in test data (`data/cv_data.json`)
3. Add HTML template in `templates/` (if needed)
4. Update `src/html_generator/html_generators.rs` for rendering
5. Update `src/typst_generator/sections.rs` for PDF output

### Adding a New Configuration Option
1. Update `src/unified_config.rs` with new config field
2. Add default value and deserialization logic
3. Update `config/site.json` with example (if site-level)
4. Use the config in relevant generator modules

### Debugging Build Issues
- Check `RUST_LOG=debug cargo run` output for detailed logs
- Look for performance bottlenecks in profiler summary
- Verify GitHub cache is being used (look for cache hit messages)
- Check that `gh` CLI is authenticated: `gh auth status`

### Modifying Templates
- Templates are in `templates/` directory
- Use Askama syntax: `{{ variable }}`, `{% for %} {% endfor %}`, etc.
- Templates are checked at compile time - syntax errors fail the build
- Changes require rebuild (`cargo build`)

## Configuration Management

The application loads configuration from multiple sources (precedence order):
1. Command-line arguments (`--cache-path`, `--public-data`)
2. Environment variables (prefix: `CV__`)
3. Configuration file (`config.toml` - optional)
4. Default values (in `unified_config.rs`)

## GitHub Workflows

- **`.github/workflows/ci.yml`** - Tests and builds on main branch changes
- **`.github/workflows/content-deploy.yml`** - Deploys to GitHub Pages on content branch changes

The content deploy workflow:
1. Checks out content branch for data
2. Checks out main branch for code
3. Authenticates with `gh` CLI
4. Runs `cargo run` to generate site
5. Deploys `dist/` to GitHub Pages

## Dependencies

Key dependencies (see `Cargo.toml` for versions):
- **serde/serde_json** - JSON serialization
- **im** - Immutable collections
- **tokio** - Async runtime (for GitHub API)
- **reqwest** - HTTP client
- **askama** - Template engine
- **pulldown-cmark** - Markdown processing
- **anyhow** - Error handling
- **tracing** - Logging
- **chrono** - Date/time handling
- **minify-html** - HTML minification

## File Structure

```
cv/
├── src/              # Rust source code
├── templates/        # Askama HTML templates
├── static/           # Static assets (CSS, JS, fonts, images)
├── config/           # Configuration files
├── data/             # Test data (main branch only)
├── dist/             # Generated output (gitignored)
├── cache/            # GitHub API cache (gitignored)
└── .github/          # GitHub Actions workflows
```

## Important Notes

- **Immutability**: Prefer `im` collections over `std` collections for data structures
- **Error handling**: Use `anyhow::Result` and `.context()` for rich error messages
- **Logging**: Use `tracing` macros (`info!`, `debug!`, `warn!`, `error!`)
- **Testing**: Focus on testing pure functions, mock GitHub API calls in tests
- **Performance**: Always consider caching implications when adding new GitHub API calls
- **Typst compilation**: Requires `typst` CLI to be installed and in PATH