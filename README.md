# Personal Website with Dynamic CV Generator and Blog

[![Rust CI](https://github.com/yourusername/personal-website/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/personal-website/actions/workflows/ci.yml)
[![Deploy CV](https://github.com/yourusername/personal-website/actions/workflows/deploy.yml/badge.svg)](https://github.com/yourusername/personal-website/actions/workflows/deploy.yml)
[![Deploy Blog API](https://github.com/yourusername/personal-website/actions/workflows/deploy-blog-api.yml/badge.svg)](https://github.com/yourusername/personal-website/actions/workflows/deploy-blog-api.yml)

This project is a comprehensive personal website solution that includes a dynamically generated CV and a full-featured blog system. It's built with Rust using functional programming principles and modern web technologies. The CV is generated in both HTML and PDF formats from a single data source, and the blog system provides a RESTful API for content management.

## Features

- **Dynamic CV Generation**: Creates both HTML and PDF versions of your CV from a single JSON data source
- **Blog System**: Full-featured blog with a RESTful API for content management
- **GitHub Integration**: Automatically fetches and displays your GitHub repositories
- **Responsive Design**: Mobile-friendly interface that works on all devices
- **Dark/Light Theme**: User-selectable theme with automatic system preference detection
- **Functional Programming**: Built with immutable data structures and functional programming principles
- **Docker Support**: Easy deployment with Docker and Docker Compose
- **CI/CD Pipeline**: Automated testing and deployment with GitHub Actions

> **Project Roadmap**: See [PROJECT_ROADMAP.md](PROJECT_ROADMAP.md) for the current project status, completed work, and the plan for future development with a focus on deployment.

## Directory Structure

```
.
├── Cargo.toml           # Rust project configuration
├── Cargo.lock           # Rust dependency lock file
├── blog_api_server.rs   # Blog API server entry point
├── blog_tester.rs       # Blog testing utility
├── blog.db              # SQLite database for blog content
├── DEPLOYMENT.md        # Deployment documentation
├── deploy.sh            # Deployment script for blog API
├── deploy-local.sh      # Local deployment script
├── Dockerfile           # Docker configuration for blog API
├── Dockerfile.local     # Docker configuration for local development
├── docker-compose.yml   # Docker Compose configuration for production
├── docker-compose.local.yml # Docker Compose configuration for local development
├── PROJECT_ROADMAP.md   # Project roadmap and status
├── README-dev.md        # Development guidelines
├── README.md            # This file
├── test.sh              # Test runner script
├── data/                # Data files
│   └── cv_data.json     # CV data in JSON format
├── src/                 # Rust source code
│   ├── main.rs          # Main application entry point
│   ├── cv_data.rs       # CV data model
│   ├── html_generator.rs # HTML generation logic
│   ├── typst_generator.rs # PDF generation logic
│   ├── blog_api.rs      # Blog API server logic
│   ├── blog_data.rs     # Blog data model
│   └── db/              # Database access layer
│       ├── mod.rs       # Database module entry point
│       ├── repository.rs # Repository pattern implementation
│       ├── migrations.rs # Database schema migrations
│       └── error.rs     # Custom error types for database operations
├── static/              # Static web assets
│   ├── blog-client.html # Blog client interface
│   ├── style.css        # Main CSS styles
│   ├── css/             # CSS files
│   │   └── components/  # Component-specific CSS
│   │       └── header.css # Header component styles
│   ├── js/              # JavaScript files
│   │   ├── scripts.js   # Main JavaScript functionality
│   │   └── blog-debug.js # Debug tool for blog API
│   └── img/             # Image assets
├── templates/           # Askama HTML templates
│   ├── base.html        # Base template with common structure
│   ├── index.html       # Template for landing page
│   ├── cv.html          # Template for CV page
│   ├── projects.html    # Template for projects page
│   └── partials/        # Partial templates
│       ├── header.html  # Header partial
│       ├── footer.html  # Footer partial
│       └── project-card.html # Project card partial
├── tests/               # Test files
└── test_data/           # Test data files
```

## Installation

### Prerequisites

- **Rust and Cargo** (version 1.70.0 or higher)
- **SQLite3** (for the blog database)
- **Node.js and npm** (for frontend development tools)
- **Docker and Docker Compose** (optional, for containerized deployment)

### Step 1: Install Rust and Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Step 2: Clone the Repository

```bash
git clone https://github.com/yourusername/personal-website.git
cd personal-website
```

### Step 3: Install Dependencies

```bash
# Install Rust dependencies
cargo build

# Install Node.js dependencies (for frontend tools)
npm install
```

### Step 4: Set Up the Database

The blog system uses SQLite for data storage. The database will be created automatically when you first run the application, but you can also initialize it manually:

```bash
# Initialize the database with schema
cargo run --bin blog_api_server -- --init-db
```

> **For Developers**: See [README-dev.md](README-dev.md) for detailed development guidelines, including functional programming principles and best practices.

## Usage

The project consists of two main components:
1. The CV generator and static website
2. The blog API server

### CV and Website

1. **Customize your CV data**:

   Edit the `data/cv_data.json` file to include your personal information, experiences, education, skills, projects, and more.

2. **Generate the website**:

   ```bash
   # For development
   cargo run

   # For production (with optimizations)
   cargo run --release
   ```

3. **View the generated files**:

   The generated files will be in the `dist/` directory:
   - `index.html`: The landing page/blog frontend
   - `cv.html`: The HTML version of your CV
   - `projects.html`: The projects page
   - `cv.pdf`: The PDF version of your CV

   Static assets (CSS, JavaScript, images) remain in the `static/` directory.

### Blog API Server

1. **Start the blog API server**:

   ```bash
   # Start the server in development mode
   cargo run --bin blog_api_server

   # Start the server in production mode
   cargo run --bin blog_api_server --release
   ```

   The server will start on port 3000 by default.

2. **Access the blog API**:

   - API Endpoints: http://localhost:3000/api/blog
   - Blog Client: http://localhost:3000/static/blog-client.html
   - Debug Tool: http://localhost:3000/static/blog-debug.html

3. **Test the blog functionality**:

   ```bash
   # Run the blog tester utility
   cargo run --bin blog_tester
   ```

### Local Development Environment

For a complete local development environment with hot reloading:

```bash
# Make the script executable
chmod +x deploy-local.sh

# Start the local development environment
./deploy-local.sh start
```

This will start a development server on port 3002 with the CV website and blog API.

## Deployment

> **Detailed Deployment Guide**: For comprehensive deployment instructions, including local development setup, CI/CD pipeline configuration, Docker configuration, and troubleshooting, see [DEPLOYMENT.md](DEPLOYMENT.md).

The project supports multiple deployment options for both the CV website and the blog API server.

### CV Website Deployment

The CV website is a static site that can be deployed to any web hosting service:

1. **GitHub Pages** (Recommended):
   ```bash
   # Build the project
   cargo run --release

   # Deploy to GitHub Pages using the provided script
   ./deploy.sh
   ```

2. **Traditional Web Hosting**:
   ```bash
   # Build the project
   cargo run --release

   # Upload the contents of the static/ directory to your web server
   ```

3. **Netlify**:
   - Connect your GitHub repository to Netlify
   - Set the build command to `cargo run --release`
   - Set the publish directory to `static/`

### Blog API Server Deployment

The blog API server can be deployed using Docker for easy setup and management:

1. **Docker Deployment** (Recommended):
   ```bash
   # Deploy using Docker Compose
   docker-compose up -d
   ```

2. **Manual Deployment**:
   ```bash
   # Build the server
   cargo build --release --bin blog_api_server

   # Run the server
   ./target/release/blog_api_server
   ```

### Local Development Environment

For local development and testing:

```bash
# Start the local development environment
./deploy-local.sh start

# Stop the local development environment
./deploy-local.sh stop
```

This will start a development server on port 3002 with hot reloading, making it easy to test changes before deployment.

> **Note**: The first time you start the local environment, it may take several minutes for the Rust application to compile. Subsequent starts will be faster as the compiled artifacts are cached.

### CI/CD Pipeline

The project includes GitHub Actions workflows for automated testing and deployment:

- **CI Workflow**: Runs tests and linting on every push and pull request
- **Deploy CV Workflow**: Deploys the CV website to GitHub Pages on pushes to the main branch
- **Deploy Blog API Workflow**: Deploys the blog API server to a production server on pushes to the main branch

For detailed instructions on setting up and customizing the CI/CD pipeline, see [DEPLOYMENT.md](DEPLOYMENT.md#cicd-pipeline).

## Customization

### CV Data

Edit the `data/cv_data.json` file to update your personal information, experiences, education, skills, projects, languages, and certifications.

### HTML Templates

- `templates/base.html`: Modify this file to change the common structure of all pages
- `templates/cv.html`: Modify this file to change the structure of the HTML CV
- `templates/index.html`: Modify this file to change the landing page
- `templates/projects.html`: Modify this file to change the projects page
- `templates/partials/`: Modify files in this directory to change reusable components
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

1. **Home Page**: The main landing page that serves as the blog frontend, displaying recent blog posts and personal information.
2. **CV Page**: Displays the HTML version of the CV with a link to download the PDF version.
3. **Projects Page**: Displays GitHub projects with details fetched from the GitHub API.

### Server-Side vs. Client-Side Processing

The website uses a modern template-based architecture with server-side rendering:

1. **Server-Side Processing**:
   - HTML pages are generated server-side using Askama templates
   - Templates are located in the `templates/` directory
   - The base template (`templates/base.html`) provides the common structure for all pages
   - Partial templates in `templates/partials/` provide reusable components like header and footer
   - The Rust application processes these templates and generates the final HTML

2. **Client-Side Processing**:
   - JavaScript is used only for interactive elements like theme switching and accordion functionality
   - The `scripts.js` file handles common functionality across all pages
   - Blog-specific JavaScript is contained in separate files

This approach ensures a consistent user experience across all pages while minimizing client-side JavaScript dependencies. Blog posts are designed to use the full width of their container for optimal readability.

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
  - Home/Blog: http://localhost:3002/
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

## Technologies

This project is built with modern technologies and follows functional programming principles:

### Backend

- **[Rust](https://www.rust-lang.org/)**: A systems programming language focused on safety, speed, and concurrency
- **[Axum](https://github.com/tokio-rs/axum)**: A web framework built on top of Tokio, hyper, and tower
- **[Tokio](https://tokio.rs/)**: An asynchronous runtime for Rust
- **[SQLite](https://www.sqlite.org/)**: A self-contained, serverless database engine
- **[Askama](https://github.com/djc/askama)**: A type-safe, compiled templating engine for Rust

### Frontend

- **HTML5/CSS3**: Modern web standards for structure and styling
- **JavaScript**: For interactive elements and theme switching
- **[Nerdfont Icons](https://www.nerdfonts.com/)**: Icon font for UI elements

### Data Management

- **[Serde](https://serde.rs/)**: A framework for serializing and deserializing Rust data structures
- **[im](https://docs.rs/im/)**: Immutable data structures for Rust
- **[Rusqlite](https://github.com/rusqlite/rusqlite)**: SQLite bindings for Rust
- **[R2D2](https://github.com/sfackler/r2d2)**: A connection pool for Rust

### Error Handling and Logging

- **[Thiserror](https://github.com/dtolnay/thiserror)**: For defining custom error types
- **[Anyhow](https://github.com/dtolnay/anyhow)**: For flexible error handling
- **[Tracing](https://github.com/tokio-rs/tracing)**: A framework for structured, contextual logging

### Testing

- **[Proptest](https://github.com/AltSysrq/proptest)**: Property-based testing for Rust
- **Rust's built-in testing framework**: For unit and integration tests

### Deployment and Infrastructure

- **[Docker](https://www.docker.com/)**: For containerization
- **[GitHub Actions](https://github.com/features/actions)**: For CI/CD pipelines
- **[GitHub Pages](https://pages.github.com/)**: For hosting the static website

## License

This project is licensed under the MIT License - see the LICENSE file for details.
