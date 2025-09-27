# 🚀 CV Generator

> A blazing-fast, functional CV/portfolio generator built in Rust that creates beautiful HTML and PDF outputs from JSON data

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![GitHub Pages](https://img.shields.io/badge/demo-GitHub%20Pages-brightgreen.svg)](https://hakimjonas.github.io/cv/)
[![Performance](https://img.shields.io/badge/build_time-<1s-brightgreen.svg)](#performance)

## ✨ Features

- 🎯 **Single Source of Truth**: JSON-based CV data with automatic HTML/PDF generation
- 🚀 **GitHub Integration**: Automatically fetches and displays your latest GitHub projects
- 📱 **Responsive Design**: Beautiful, mobile-friendly interface
- 🎨 **Modern UI**: Clean, professional styling with excellent typography
- ⚡ **Blazing Fast**: Sub-second builds with intelligent caching (77% faster than v1)
- 🧠 **Smart Caching**: GitHub API responses cached with TTL for optimal performance
- 🔧 **Fork-Ready**: Complete separation between code and content for easy customization
- 📊 **Performance Profiling**: Built-in timing and optimization tools
- 🏗️ **Modular Architecture**: Clean, maintainable codebase following functional principles

## 🎪 Live Demo

Check out the live example: **[hakimjonas.github.io/cv](https://hakimjonas.github.io/cv)**

## 🚀 Quick Start (Fork & Deploy)

### 1. Fork This Repository
Click the "Fork" button above or visit: https://github.com/hakimjonas/cv/fork

### 2. Set Up Your Content Branch
```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/cv.git
cd cv

# Switch to the content branch
git checkout content

# Edit your CV data
nano data/cv_data.json
```

### 3. Customize Your Data
Update `data/cv_data.json` with your information:

```json
{
  "personal_info": {
    "name": "Your Name",
    "title": "Your Professional Title",
    "email": "your.email@example.com",
    "summary": "Your professional summary...",
    "social_links": {
      "LinkedIn": "https://linkedin.com/in/yourprofile",
      "GitHub": "https://github.com/yourusername"
    }
  },
  "github_sources": [
    {
      "username": "yourusername",
      "organization": null
    }
  ]
}
```

### 4. Commit & Push
```bash
# Commit your changes to the content branch
git add data/cv_data.json
git commit -m "Update CV data with my information"
git push origin content
```

### 5. Enable GitHub Pages
1. Go to your fork's Settings → Pages
2. Set Source to "GitHub Actions"
3. Your site will be available at: `https://yourusername.github.io/cv`

### 6. Configure Repository Settings (Optional)
Set these environment variables in Settings → Secrets and variables → Actions:

| Variable | Description | Default |
|----------|-------------|---------|
| `CV_REPO` | Repository path | `yourusername/cv` |
| `CV_BRANCH` | Content branch | `content` |
| `CV_DATA_PATH` | Data file path | `data/cv_data.json` |

## 🏗️ Architecture

This project uses a smart **branch-based architecture** with modular design:

```
main branch    → Application code (Rust, templates, styles)
content branch → Your personal CV data (JSON files)
gh-pages      → Auto-generated static site
```

### Code Architecture

```
src/
├── main.rs                    # Application entry point
├── cv_data.rs                 # Data structures & parsing
├── github.rs                  # GitHub API integration + caching
├── github_cache.rs            # Intelligent caching system
├── performance.rs             # Build profiling & optimization
├── css_generator.rs           # Dynamic CSS generation
├── html_generator/            # HTML generation system
│   ├── mod.rs                # Main coordination
│   ├── html_generators.rs    # Core HTML page generation
│   ├── config_generators.rs  # Server config generation
│   ├── asset_processor.rs    # Static asset handling
│   └── utils.rs              # Shared utilities
└── typst_generator/           # PDF generation system
```

**Branch Benefits:**
- ✅ Clean separation between code and content
- ✅ Easy to fork and customize
- ✅ Automatic deployments on data changes
- ✅ Version control for your CV history
- ✅ Leverages GitHub's native features

**⚠️ IMPORTANT:**
- **NEVER merge the `content` branch into `main`** - they are designed to remain separate
- The `content` branch contains your personal data and should only be updated with your CV information
- The `main` branch contains the application code and should only receive code improvements
- Both branches are protected and require pull request reviews for changes

## ⚡ Performance

This generator achieves **sub-second builds** through intelligent optimizations:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Total Build Time** | 2,266ms | 524ms | **77% faster** |
| **GitHub API Calls** | ~1,600ms | 0ms (cached) | **100% cached** |
| **CSS Generation** | Always rebuild | Smart cache | **90% cache hits** |
| **Asset Processing** | Serial | Parallel | **3x faster** |

### Performance Features

- **🧠 Intelligent Caching**: GitHub API responses cached with configurable TTL
- **⚡ Smart CSS**: Configuration-based cache invalidation
- **📊 Build Profiling**: Real-time performance measurement and reporting
- **🔄 Parallel Processing**: Concurrent asset processing and generation

## 🛠️ Local Development

### Prerequisites
- [Rust](https://rustup.rs/) (stable)
- [GitHub CLI](https://cli.github.com/) (for GitHub integration)
- [Typst](https://typst.app/) (for PDF generation)

### Setup
```bash
# Clone the repository
git clone https://github.com/hakimjonas/cv.git
cd cv

# Install Rust dependencies
cargo build

# Install GitHub CLI (if not already installed)
# macOS: brew install gh
# Ubuntu: sudo apt install gh
# Windows: winget install GitHub.cli

# Install Typst
cargo install typst-cli

# Authenticate GitHub CLI
gh auth login
```

### Development Commands
```bash
# Generate CV (HTML + PDF)
cargo run --bin cv

# Use local data instead of content branch
echo '{}' > data/cv_data.json  # Create local data file
cargo run --bin cv

# Override data source via environment variables
CV_REPO="otherusername/cv" cargo run --bin cv
```

## 📁 Project Structure

```
├── src/                           # 🦀 Rust application code
│   ├── main.rs                   # Main application entry point
│   ├── cv_data.rs                # CV data structures and parsing
│   ├── github.rs                 # GitHub API integration + caching
│   ├── github_cache.rs           # Intelligent caching system (77% faster)
│   ├── performance.rs            # Build profiling & optimization tools
│   ├── css_generator.rs          # Dynamic CSS generation with caching
│   ├── html_generator/           # 🏗️ Modular HTML generation system
│   │   ├── mod.rs               # Main coordination & orchestration
│   │   ├── html_generators.rs   # Core HTML page generation (281 lines)
│   │   ├── config_generators.rs # Server config generation (441 lines)
│   │   ├── asset_processor.rs   # Static asset handling (218 lines)
│   │   └── utils.rs             # Shared utilities & minification (156 lines)
│   ├── typst_generator/          # 📄 PDF generation system
│   │   ├── mod.rs               # Typst coordination
│   │   └── markup.rs            # Markup generation & formatting
│   ├── blog_posts.rs             # Blog system with tagging
│   ├── markdown_pages.rs         # Static page generation
│   └── site_config.rs            # Site configuration management
├── static/                       # 🎨 Static assets (CSS, JS, images)
├── templates/                    # 📄 HTML templates (Askama)
├── dist/                         # 📦 Generated output (HTML, PDF, assets)
├── cache/                        # 🧠 GitHub API cache storage
├── data/                         # 📊 CV data (only in content branch)
├── config/                       # ⚙️ Site configuration files
├── content/                      # 📝 Blog posts & static pages (Markdown)
├── IMPROVEMENT_PLAN.md           # 📋 8-week roadmap to 10/10 quality
└── .github/workflows/            # 🔄 CI/CD automation
```

## 🎨 Customization

### Styling
Edit CSS files in `static/css/` to customize the appearance:
- `static/css/main.css` - Main styles
- `static/css/components/` - Component-specific styles

### Templates
Modify HTML templates in `templates/`:
- `templates/cv.html` - Main CV template
- `templates/index.html` - Landing page template

### Data Structure
The CV data follows this JSON schema (see `data/cv_data.json` in content branch):

```json
{
  "personal_info": { "..." },      // Basic personal information
  "experiences": [ "..." ],        // Work experience
  "education": [ "..." ],          // Educational background
  "skill_categories": [ "..." ],   // Technical skills
  "projects": [ "..." ],           // Manual projects (GitHub ones are auto-fetched)
  "languages": { "..." },          // Spoken languages
  "certifications": [ "..." ],     // Professional certifications
  "github_sources": [ "..." ]      // GitHub accounts/orgs to fetch projects from
}
```

## 🔧 Configuration

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `CV_REPO` | Source repository for CV data | `username/cv` |
| `CV_BRANCH` | Branch containing CV data | `content` |
| `CV_DATA_PATH` | Path to CV data file | `data/cv_data.json` |
| `GITHUB_TOKEN` | GitHub API token (optional) | `ghp_...` |

### GitHub Integration
The application automatically fetches your latest GitHub projects using the GitHub CLI. Make sure you're authenticated:

```bash
gh auth login
gh auth status
```

Projects are filtered to show only:
- ✅ Public repositories
- ✅ Non-fork repositories
- ✅ Latest 10 projects (sorted by update date)

## 🚀 Deployment

### Automatic Deployment
The repository includes GitHub Actions workflows that automatically:

1. **Build** the CV when code changes (main branch)
2. **Rebuild** when CV data changes (content branch)
3. **Deploy** to GitHub Pages
4. **Generate** both HTML and PDF outputs

### Manual Deployment
```bash
# Build the site locally
cargo run --bin cv

# Deploy to GitHub Pages (if configured)
# The dist/ folder contains all generated files
```

## 🤝 Contributing

This project follows functional programming principles and emphasizes immutability. When contributing:

1. Use immutable data structures (`im` crate)
2. Prefer pure functions without side effects
3. Add comprehensive tests for new features
4. Follow existing code style and patterns

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with ❤️ using [Rust](https://www.rust-lang.org/)
- PDF generation powered by [Typst](https://typst.app/)
- Styled with modern CSS and [Nerd Fonts](https://www.nerdfonts.com/)
- Deployed on [GitHub Pages](https://pages.github.com/)

---

**Made with 🦀 Rust and functional programming principles**

*Fork this repository and have your professional CV website up and running in minutes!*# Force rebuild
# Test deployment with real CV data
