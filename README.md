# CV Generator

A fast CV generator that creates HTML and PDF outputs from JSON data.

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Getting Started

**Using GitHub Template (Recommended)**

Click "Use this template" button at the top of this repository. This creates a fresh copy without the original author's content branch.

Then:

```bash
git clone https://github.com/YOUR_USERNAME/cv.git
cd cv
git checkout -b content
```

**Or Fork (Manual cleanup required)**

If you fork instead, delete the original `content` branch and create your own:

```bash
git clone https://github.com/YOUR_USERNAME/cv.git
cd cv
git push origin --delete content
git checkout -b content
git push -u origin content
```

Edit `data/cv_data.json` with your information:
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

Commit and push:

```bash
git add data/cv_data.json
git commit -m "Update CV data"
git push origin content
```

Enable GitHub Pages in your repository settings: Settings → Pages → Source: "GitHub Actions"

Your site will be live at `https://yourusername.github.io/cv`

## Branch Structure

This project uses two separate branches:

- `main` - Application code (templates, styles, build system)
- `content` - Your CV data and blog posts

Keep these branches separate. Never merge `content` into `main`.

## Configuration

Create `config/site.json` on the `content` branch to customize your site. The `main` branch contains generic defaults as fallback.

### Menu

Edit the navigation menu:

```json
{
  "menu": [
    { "label": "Home", "path": "index.html", "item_type": "home" },
    { "label": "CV", "path": "cv.html", "item_type": "cv" },
    { "label": "Projects", "path": "projects.html", "item_type": "projects" },
    { "label": "Blog", "path": "blog.html", "item_type": "blog" }
  ]
}
```

### Color Scheme

Choose a color scheme:

```json
{
  "colorscheme": {
    "name": "Dracula",
    "source": "iterm2",
    "url": "https://github.com/mbadolato/iTerm2-Color-Schemes"
  }
}
```

Available sources: `iterm2`, `github`, or `local`

Popular schemes: Dracula, Nord, Solarized, Gruvbox, Rose Pine

### Fonts

Configure fonts:

```json
{
  "fonts": {
    "primary": "JetBrainsMono",
    "fallback": "FiraCode, monospace",
    "source": "nerd-fonts"
  }
}
```

Font sources: `nerd-fonts`, `google-fonts`, or `system`

### PDF Generation

Configure PDF output using Typst:

```json
{
  "typst": {
    "theme": {
      "name": "grotesk-cv",
      "version": "1.0.2"
    },
    "customization": {
      "layout": {
        "paper_size": "a4",
        "font": "HK Grotesk",
        "font_size": "9pt"
      }
    }
  }
}
```

Paper sizes: `a4`, `letter`, `legal`, `a3`, `a5`, `a6`

## GitHub Integration

The generator automatically fetches your GitHub projects and avatar. It uses multiple fallback strategies for reliability:

**Authentication Priority:**
1. `GITHUB_TOKEN` - Automatically provided by GitHub Actions (5,000 req/hr)
2. `GH_TOKEN` - Optional user-provided token (5,000 req/hr)
3. `gh` CLI - Falls back to gh CLI if installed
4. Public API - Unauthenticated requests as last resort (60 req/hr)

**Setting a Token (Optional):**

If you want to test locally with higher rate limits, set the `GH_TOKEN` environment variable:

```bash
export GH_TOKEN=your_github_token
cargo run --bin cv
```

GitHub Actions automatically provides `GITHUB_TOKEN`, so no configuration is needed for deployment.

**Cache System:**

The generator caches GitHub data in `cache/github_cache.json` to avoid unnecessary API calls and speed up rebuilds. The cache is automatically managed and respects TTL settings.

## Content Files

All content files go on the `content` branch.

### CV Data

Edit `data/cv_data.json` with your professional information.

### Blog Posts

Create markdown files in `content/blog/`:

```markdown
---
title: "Post Title"
date: 2024-01-15
published: true
---

Your content here.
```

### Static Pages

Create markdown files in `content/pages/`:

```markdown
---
title: "Page Title"
slug: "page-slug"
---

Your content here.
```

## License

MIT License - see [LICENSE](LICENSE) file for details.