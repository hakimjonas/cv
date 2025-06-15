# Personal Website with Dynamic CV Generator

[![Rust CI](https://github.com/yourusername/personal-website/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/personal-website/actions/workflows/ci.yml)
[![Deploy CV](https://github.com/yourusername/personal-website/actions/workflows/deploy.yml/badge.svg)](https://github.com/yourusername/personal-website/actions/workflows/deploy.yml)
[![Deploy Blog API](https://github.com/yourusername/personal-website/actions/workflows/deploy-blog-api.yml/badge.svg)](https://github.com/yourusername/personal-website/actions/workflows/deploy-blog-api.yml)

This project generates a personal website with a dynamically generated CV in both HTML and PDF formats from a single Rust data source. It leverages Rust for content generation, HTML/CSS for the web interface, and Typst for PDF output.

## Features

- **Single Source of Truth**: All CV data is stored in a single JSON file
- **Multiple Output Formats**: Generates both HTML and PDF versions of your CV
- **Responsive Design**: The website works on all devices
- **Customizable**: Easy to customize the design and content
- **Functional Programming**: Uses immutable data structures and functional programming principles

> **Project Roadmap**: See [PROJECT_ROADMAP.md](PROJECT_ROADMAP.md) for the current project status, completed work, and the plan for future development with a focus on deployment.

## Directory Structure

```
.
├── .github/             # GitHub configuration
│   └── workflows/       # GitHub Actions workflows
│       ├── ci.yml       # CI workflow for testing and linting
│       ├── deploy.yml   # Workflow for deploying CV to GitHub Pages
│       └── deploy-blog-api.yml # Workflow for deploying blog API
├── Cargo.toml           # Rust project configuration
├── DEPLOYMENT.md        # Deployment documentation
├── Dockerfile           # Docker configuration for blog API
├── PROJECT_ROADMAP.md   # Project roadmap and status
├── README-dev.md        # Development guidelines
├── README.md            # This file
├── data/
│   └── cv_data.json     # CV data in JSON format
├── deploy.sh            # Deployment script for blog API
├── dist/                # Generated output files
│   ├── cv.html          # Generated HTML CV
│   ├── cv.pdf           # Generated PDF CV
│   ├── index.html       # Copied from static/
│   └── style.css        # Copied from static/
├── docker-compose.yml   # Docker Compose configuration
├── src/
│   ├── main.rs          # Main application entry point
│   ├── cv_data.rs       # CV data model
│   ├── html_generator.rs # HTML generation logic
│   ├── typst_generator.rs # PDF generation logic
│   ├── blog_api.rs      # Blog API server
│   ├── blog_data.rs     # Blog data model
│   └── db/              # Database access layer
│       ├── mod.rs       # Database module entry point
│       ├── repository.rs # Repository pattern implementation
│       ├── migrations.rs # Database schema migrations
│       └── error.rs     # Custom error types for database operations
├── static/
│   ├── index.html       # Static landing page
│   ├── style.css        # CSS styles for the website
│   ├── blog-client.html # Blog client interface
│   └── js/              # JavaScript files
│       └── blog-debug.js # Debug tool for blog API
└── templates/
    └── cv.html          # Askama template for CV HTML
```

## Installation

1. Install Rust and Cargo:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Typst CLI:
   ```bash
   cargo install typst-cli
   ```

3. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/personal-website.git
   cd personal-website
   ```

4. Build the project:
   ```bash
   cargo build --release
   ```

> **For Developers**: See [README-dev.md](README-dev.md) for detailed development guidelines, including functional programming principles and best practices.

## Usage

1. Customize your CV data in `data/cv_data.json`

2. Generate your website:

   For development:
   ```bash
   cargo run
   ```

   For production:
   ```bash
   cargo run --release
   ```

3. The generated files will be in:
   - Development build: `dist/` directory
   - Production build: `dist/` directory

   Each directory contains:
   - `index.html`: The landing page
   - `cv.html`: The HTML version of your CV
   - `cv.pdf`: The PDF version of your CV
   - `style.css`: The CSS styles for the website
   - Additional optimized assets and configuration files (in production build)

4. Deploy the contents of the `dist/` directory to your web server or hosting service

## Deployment

> **Detailed Deployment Guide**: For comprehensive deployment instructions, including local development setup, CI/CD pipeline configuration, Docker configuration, and troubleshooting, see [DEPLOYMENT.md](DEPLOYMENT.md).

### Local Development and Testing

Before deploying to production, it's recommended to test your changes in a local development environment:

```bash
# Make the script executable
chmod +x deploy-local.sh

# Copy necessary files from dist to static directory
cp dist/cv.html dist/cv.pdf dist/projects.html static/

# Start the local development environment
./deploy-local.sh start
```

This will start a development environment with hot reloading, making it easy to test changes before deployment. The copy step ensures that all pages (including CV and Projects) are available in the local environment. For more details, see [DEPLOYMENT.md](DEPLOYMENT.md#local-development-and-testing).

### Production Deployment

The production build (created with `cargo run --release`) includes several optimizations and server configuration files:

1. **Minified HTML, CSS, and JavaScript**: All text-based assets are minified to reduce file size
2. **Gzipped versions**: Pre-compressed versions of files are created for faster delivery
3. **Server configuration files**:
   - `.htaccess` for Apache servers
   - `web.config` for IIS servers
   - `_headers` and `_redirects` for Netlify
   - `robots.txt` for search engines
   - `manifest.json` and `service-worker.js` for Progressive Web App (PWA) support

### Deployment Options

This project supports multiple deployment options:

1. **CI/CD Pipeline**: Automated deployment using GitHub Actions
2. **Traditional Web Hosting**: Manual deployment to any web server
3. **GitHub Pages**: Automated or manual deployment to GitHub Pages
4. **Netlify**: Automated deployment to Netlify
5. **Docker**: Containerized deployment for the blog API

For detailed instructions on each deployment option, see [DEPLOYMENT.md](DEPLOYMENT.md).

## Customization

### CV Data

Edit the `data/cv_data.json` file to update your personal information, experiences, education, skills, projects, languages, and certifications.

### HTML Templates

- `templates/cv.html`: Modify this file to change the structure of the HTML CV
- `static/index.html`: Modify this file to change the landing page
- `static/style.css`: Modify this file to change the styling of the website

### PDF Template

The PDF template is generated in code in the `src/typst_generator.rs` file. Modify the `generate_typst_markup` function to change the structure and styling of the PDF CV.

## GitHub Integration

This project can fetch your GitHub repositories and include them in your CV. To use this feature:

1. Add your GitHub username and/or organization to the `github_sources` field in `data/cv_data.json`:
   ```json
   {
     "github_sources": [
       {
         "username": "yourusername",
         "organization": null
       },
       {
         "username": null,
         "organization": "yourorganization"
       }
     ]
   }
   ```

2. **Important: GitHub API Rate Limiting**

   The GitHub API has rate limits:
   - Unauthenticated requests: 60 requests per hour
   - Authenticated requests: 5,000 requests per hour

   To avoid rate limiting, you can provide a GitHub API token using one of the following methods (in order of preference):

   a. Create a personal access token on GitHub:
      - Go to GitHub Settings → Developer settings → Personal access tokens
      - Generate a new token with the `public_repo` scope

   b. Store the token in your git config (recommended for local development):
      ```bash
      git config --global cv.github.token "your-token-here"
      ```
      This stores the token securely in your global git config, which is not committed to version control.

   c. Set the token as an environment variable:
      ```bash
      GITHUB_TOKEN=your-token-here cargo run
      ```

3. **For Production Deployment with GitHub Actions**

   When deploying with GitHub Actions, you can use GitHub Secrets to store your token securely:

   a. Add your token as a secret in your GitHub repository:
      - Go to your repository → Settings → Secrets and variables → Actions
      - Click "New repository secret"
      - Name: `GITHUB_TOKEN`
      - Value: your personal access token

   b. In your GitHub Actions workflow file, make the secret available as an environment variable:
      ```yaml
      jobs:
        build:
          runs-on: ubuntu-latest
          env:
            GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          steps:
            - uses: actions/checkout@v3
            - name: Build CV
              run: cargo run --release
      ```

   The application will automatically detect when it's running in GitHub Actions and use the token from the environment.

## Blog Functionality

This project includes a blog functionality that allows you to create and manage blog posts. The blog features:

- **SQLite Database**: Blog posts are stored in a SQLite database
- **RESTful API**: A complete API for creating, reading, updating, and deleting blog posts
- **Tagging System**: Support for categorizing posts with tags
- **Markdown Support**: Write blog posts in Markdown format

### Blog API Endpoints

- `GET /api/blog` - Get all blog posts
- `GET /api/blog/{slug}` - Get a specific blog post
- `POST /api/blog` - Create a new blog post
- `PUT /api/blog/{slug}` - Update a blog post
- `DELETE /api/blog/{slug}` - Delete a blog post
- `GET /api/blog/tags` - Get all tags
- `GET /api/blog/published` - Get all published posts
- `GET /api/blog/featured` - Get all featured posts
- `GET /api/blog/tag/{tag_slug}` - Get all posts with a specific tag

## Testing

The project includes comprehensive tests for all functionality. Run them with our consolidated test script:

```bash
# Run all tests
./test.sh
```

The test suite includes:

- **Unit Tests**: Tests for individual functions and methods
- **Integration Tests**: Tests for the API endpoints
- **Blog Functionality Tests**: Tests for the blog core and API
- **GitHub API Tests**: Tests for the GitHub integration
- **Property-Based Tests**: Tests with randomly generated inputs

You can also run specific test groups manually:

```bash
# Run all tests with Rust's test framework
cargo test

# Run with clippy to check for code quality issues
cargo clippy --tests

# Test blog core functionality
cargo run --bin blog_tester

# Start blog API server for manual testing
cargo run --bin blog_api_server
```

> **For Developers**: See [README-dev.md](README-dev.md) for detailed information about our testing strategy, including property-based testing and idempotency tests.

### Blog API Server

A robust blog API server built with Rust using Axum framework and SQLite database.

#### Features

- RESTful API for blog management
- SQLite database for data persistence
- Support for blog posts, tags, and metadata
- Docker support for easy deployment

#### Prerequisites

- Rust 1.87.0 or higher
- SQLite3
- Docker and Docker Compose (for containerized deployment)

## Blog API Server Deployment

> **Detailed Deployment Guide**: For comprehensive deployment instructions for the blog API server, including Docker configuration, health checks, and zero-downtime deployment, see [DEPLOYMENT.md](DEPLOYMENT.md).

### Quick Start

The easiest way to deploy the blog API server is with Docker:

```bash
./deploy.sh
```

This script will:
- Build the Docker image
- Start the service or perform a rolling update if it's already running
- Check the health status of the container
- Provide detailed error messages if deployment fails

## Website Structure

The website consists of three main pages:

1. **Home Page (index.html)**: The main landing page that serves as the blog frontend, displaying recent blog posts and personal information.
2. **CV Page (cv.html)**: Displays the HTML version of the CV with a link to download the PDF version.
3. **Projects Page (projects.html)**: Displays GitHub projects with details fetched from the GitHub API.

All pages share a consistent header and footer, loaded dynamically via JavaScript, ensuring a unified user experience across the site. The header and footer are implemented as HTML modules in the `static/modules/` directory and are loaded by the `modules.js` script, which ensures that all necessary CSS files are loaded before rendering the modules.

## Frontend Access

You can access the frontend of the application at the following URLs:

- **Production Website**: https://yourusername.github.io/personal-website/
  - Home/Blog: https://yourusername.github.io/personal-website/
  - CV: https://yourusername.github.io/personal-website/cv.html
  - Projects: https://yourusername.github.io/personal-website/projects.html

- **Blog API (Production)**: http://your-server-ip:3000
  - API Endpoints: http://your-server-ip:3000/api/blog
  - Blog Client: http://your-server-ip:3000/static/blog-client.html
  - Debug Tool: http://your-server-ip:3000/static/blog-debug.html

- **Local Development**:
  - Home/Blog: http://localhost:3002/static/index.html
  - Blog Client: http://localhost:3002/static/blog-client.html
  - Debug Tool: http://localhost:3002/static/blog-debug.html

> **Note**: Replace `yourusername` with your actual GitHub username and `your-server-ip` with the actual IP address or domain name of your server.

## Client

For development and testing purposes, you can also access the standalone blog client at `http://localhost:3002/static/blog-client.html` (for local development) or `http://localhost:3000/static/blog-client.html` (for production).

## Troubleshooting

If you encounter any issues:

1. Use the debug tool at `http://localhost:3002/static/blog-debug.html` (for local development)
2. Check the logs with `docker-compose logs blog-api`
3. Verify you're using the correct port (3002) for local development
4. Refer to the troubleshooting section in [DEPLOYMENT.md](DEPLOYMENT.md)

## License

MIT

## Dependencies

- [serde](https://serde.rs/): For JSON serialization/deserialization
- [askama](https://github.com/djc/askama): For HTML templating
- [im](https://docs.rs/im/): For immutable data structures
- [anyhow](https://docs.rs/anyhow/): For error handling
- [reqwest](https://github.com/seanmonstar/reqwest): For HTTP requests
- [tokio](https://tokio.rs/): For async runtime
- [git2](https://github.com/rust-lang/git2-rs): For Git configuration access
- [flate2](https://github.com/rust-lang/flate2-rs): For Gzip compression
- [minify-html](https://github.com/wilsonzlin/minify-html): For HTML minification
- [regex](https://github.com/rust-lang/regex): For CSS minification
- [axum](https://github.com/tokio-rs/axum): For the web server and API
- [rusqlite](https://github.com/rusqlite/rusqlite): For SQLite database interactions
- [typst-cli](https://github.com/typst/typst): For PDF generation (external dependency)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
