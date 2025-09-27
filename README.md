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

## 📁 Structure

```
main branch:     Code, templates, styles
content branch:  data/cv_data.json (your data)
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