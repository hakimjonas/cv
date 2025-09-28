# 🚀 CV Generator

> A fast, modern CV generator that creates beautiful HTML and PDF outputs from JSON data

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![GitHub Pages](https://img.shields.io/badge/demo-GitHub%20Pages-brightgreen.svg)](https://hakimjonas.github.io/cv/)

## 🎯 Quick Start

### 1. Fork & Clone
```bash
# Fork this repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/cv.git
cd cv
```

### 2. Add Your Data
```bash
# Switch to the content branch
git checkout content

# Edit your CV data
nano data/cv_data.json
```

Update the JSON with your information:
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
    { "username": "yourusername" }
  ],
  "experiences": [],
  "education": [],
  "skill_categories": [],
  "projects": [],
  "languages": {},
  "certifications": []
}
```

### 3. Deploy
```bash
# Commit your data
git add data/cv_data.json
git commit -m "Add my CV data"
git push origin content

# Enable GitHub Pages:
# Go to Settings → Pages → Source: "GitHub Actions"
```

Your site will be live at: `https://yourusername.github.io/cv`

## 🔧 Branch Setup

This project uses **two separate branches**:

- **`main`** → Application code (Rust, templates, CSS)
- **`content`** → Your personal CV data (JSON files)

**⚠️ Important:** Never merge `content` into `main` - they stay separate!

## 🛠️ Local Development

### Prerequisites
- [Rust](https://rustup.rs/)
- [GitHub CLI](https://cli.github.com/)
- [Typst](https://typst.app/) (for PDF generation)

### Setup & Run
```bash
# Install dependencies
cargo build

# Authenticate GitHub CLI
gh auth login

# Generate CV locally (uses test data on main branch)
cargo run

# Serve locally
cd dist && python3 -m http.server 3000
```

Visit: `http://localhost:3000`

## 🎨 Key Features

- ✅ **Automatic GitHub Integration** - Fetches your latest projects
- ✅ **Responsive Design** - Works on all devices
- ✅ **PDF Export** - Professional PDF generation
- ✅ **Fast Builds** - Sub-second regeneration
- ✅ **Dark/Light Themes** - Toggle with the switch in header

## 📝 Content Management

### 📊 CV Data (`content` branch)
Edit `data/cv_data.json` with your professional information:

<details>
<summary><strong>📋 Complete CV Data Structure</strong></summary>

```json
{
  "personal_info": {
    "name": "Your Name",
    "title": "Your Professional Title",
    "email": "your.email@example.com",
    "phone": "+1234567890",
    "website": "https://yoursite.com",
    "location": "Your City, Country",
    "summary": "Your professional summary...",
    "social_links": {
      "LinkedIn": "https://linkedin.com/in/yourprofile",
      "GitHub": "https://github.com/yourusername",
      "Twitter": "https://twitter.com/yourhandle"
    }
  },
  "experiences": [
    {
      "company": "Company Name",
      "position": "Your Position",
      "start_date": "2020-01",
      "end_date": "2023-12",
      "location": "City, Country",
      "description": "What you did at this job...",
      "achievements": [
        "Specific achievement with metrics",
        "Another accomplishment"
      ],
      "technologies": ["Rust", "TypeScript", "React"]
    }
  ],
  "education": [
    {
      "institution": "University Name",
      "degree": "Bachelor of Science",
      "field": "Computer Science",
      "start_date": "2016-09",
      "end_date": "2020-05",
      "location": "City, Country",
      "gpa": "3.8",
      "achievements": ["Magna Cum Laude", "Dean's List"]
    }
  ],
  "skill_categories": [
    {
      "name": "Programming Languages",
      "skills": ["Rust", "TypeScript", "Python", "Go"]
    },
    {
      "name": "Technologies",
      "skills": ["React", "Docker", "AWS", "PostgreSQL"]
    }
  ],
  "projects": [
    {
      "name": "Project Name",
      "description": "What this project does...",
      "url": "https://project.com",
      "repository": "https://github.com/you/project",
      "technologies": ["Rust", "WebAssembly"],
      "highlights": ["Key feature", "Impact metrics"]
    }
  ],
  "languages": {
    "English": "Native",
    "Spanish": "Conversational",
    "French": "Basic"
  },
  "certifications": [
    "AWS Solutions Architect",
    "Certified Kubernetes Administrator"
  ],
  "github_sources": [
    { "username": "yourusername" },
    { "organization": "yourcompany" }
  ]
}
```
</details>

### 📰 Blog Posts (`content` branch)
Create blog posts in `content/blog/` directory:

```bash
# Switch to content branch
git checkout content

# Create a new blog post
mkdir -p content/blog
cat > content/blog/my-first-post.md << 'EOF'
---
title: "My First Blog Post"
date: 2024-01-15
tags: ["technology", "rust", "programming"]
published: true
excerpt: "A brief description of your post..."
---

# My First Blog Post

Your blog content here using **Markdown** syntax.

## Code Examples

```rust
fn main() {
    println!("Hello, world!");
}
```

## Lists and Links

- Point one
- Point two
- [Link to somewhere](https://example.com)
EOF
```

**Blog Post Front Matter:**
- `title`: Post title (required)
- `date`: Publication date in YYYY-MM-DD format
- `tags`: Array of tags for categorization
- `published`: `true` to publish, `false` to keep as draft
- `excerpt`: Short description for post previews

### 📄 Static Pages (`content` branch)
Create static pages in `content/pages/` directory:

```bash
# Create an about page
cat > content/pages/about.md << 'EOF'
---
title: "About Me"
slug: "about"
---

# About Me

Tell your story here. This page will be available at `/about.html`.

## Background

Your background information...

## Interests

What you're passionate about...
EOF

# Create a contact page
cat > content/pages/contact.md << 'EOF'
---
title: "Contact"
slug: "contact"
---

# Get In Touch

How people can reach you...
EOF
```

### 🧭 Menu Configuration (`main` branch)
Edit `config/site.json` to customize the navigation menu:

```json
{
  "menu": [
    {
      "label": "Home",
      "path": "/",
      "item_type": "home"
    },
    {
      "label": "CV",
      "path": "/cv.html",
      "item_type": "cv"
    },
    {
      "label": "About",
      "path": "/about.html",
      "item_type": "page"
    },
    {
      "label": "Projects",
      "path": "/projects.html",
      "item_type": "projects"
    },
    {
      "label": "Blog",
      "path": "/blog.html",
      "item_type": "blog"
    },
    {
      "label": "GitHub",
      "path": "https://github.com/yourusername",
      "external": true
    }
  ]
}
```

## 🎨 Customization

### 🌈 Color Schemes
Change the color scheme by editing `config/site.json`:

```json
{
  "colorscheme": {
    "name": "Dracula",
    "source": "iterm2",
    "url": "https://github.com/mbadolato/iTerm2-Color-Schemes"
  }
}
```

**Available sources:**
- `"iterm2"` - [iTerm2 Color Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes)
- `"github"` - GitHub-based color schemes
- `"local"` - Local color scheme files

**Popular color schemes:**
- Rose Pine (light/dark)
- Dracula
- Nord
- Solarized Dark/Light
- Monokai
- Gruvbox

### 🔤 Fonts
Customize fonts in `config/site.json`:

```json
{
  "fonts": {
    "primary": "JetBrainsMono",
    "fallback": "FiraCode, monospace",
    "source": "nerd-fonts",
    "base_size": "16px",
    "weight_regular": 400,
    "weight_bold": 700
  }
}
```

**Font sources:**
- `"nerd-fonts"` - [Nerd Fonts collection](https://www.nerdfonts.com/)
- `"google-fonts"` - Google Fonts
- `"system"` - System fonts

**Popular programming fonts:**
- JetBrains Mono
- Fira Code
- Cascadia Code
- SF Mono
- Inter (for UI text)

### 📱 PDF Styling
Customize PDF output in `config/site.json`:

```json
{
  "typst": {
    "theme": {
      "name": "grotesk-cv",
      "version": "1.0.2"
    },
    "customization": {
      "colors": {
        "fill": "#f4f1eb",
        "accent": "#d4d2cc",
        "text_light": "#ededef",
        "text_medium": "#78787e",
        "text_dark": "#3c3c42"
      },
      "layout": {
        "paper_size": "a4",
        "left_pane_width": "71%",
        "font": "HK Grotesk",
        "font_size": "9pt"
      }
    }
  }
}
```

## 📁 Structure

```
main branch:     Code, templates, styles
content branch:
  ├── data/cv_data.json        # Your CV data
  ├── content/blog/            # Blog posts (.md)
  └── content/pages/           # Static pages (.md)
GitHub Actions:  Auto-deploy on changes
GitHub Pages:    Live site hosting
```

## 🚀 Deployment Details

**Automatic deployment triggers:**
- Push to `content` branch → Rebuilds with your data
- Push to `main` branch → Full code tests + rebuild

**Manual deployment:**
```bash
cargo run              # Generate site in dist/
# Deploy dist/ folder to your hosting platform
```

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

**🦀 Built with Rust • Fork and deploy in 5 minutes!**