# 📚 Git-as-CMS Blog Management Guide

This guide shows you how to use the Git-as-CMS workflow for managing your blog content.

## 🎯 **Quick Start**

### **1. Switch to Blog Branch**
```bash
git checkout blog
```

### **2. Create a New Post**
```bash
./scripts/blog-new.sh "My Awesome Post"
```

### **3. Edit and Publish**
```bash
# Edit the created file
vim data/blog/posts/2024-09-23-my-awesome-post.md

# Commit and deploy
git add .
git commit -m "blog: add post about awesome topic"
git push origin blog
```

**🚀 Your post will be live in ~30 seconds!**

## 📝 **Content Operations**

### **Create New Post**
```bash
# Published post
./scripts/blog-new.sh "Post Title"

# Draft post
./scripts/blog-new.sh --draft "Work in Progress"

# Post with tags
./scripts/blog-new.sh --tags "rust,web,tutorial" "Learning Rust"
```

### **Manage Drafts**
```bash
# List all posts and drafts
./scripts/blog-list.sh

# List only drafts
./scripts/blog-list.sh --drafts

# Publish a draft
./scripts/blog-publish.sh "my-draft-slug"
```

### **Preview Locally**
```bash
# Build and preview
./scripts/blog-preview.sh

# Build and open browser
./scripts/blog-preview.sh --open

# Watch for changes
./scripts/blog-preview.sh --watch
```

## 🌳 **Branch Workflow**

### **Blog Branch (Content Database)**
- **Purpose**: Content creation and management
- **Contains**: Blog posts, drafts, content updates
- **Deployment**: Fast blog-only build (~30 seconds)
- **Auto-sync**: Changes automatically merge to main

### **Main Branch (Application Code)**
- **Purpose**: Code, templates, styling, infrastructure
- **Contains**: Rust code, HTML templates, CSS, configuration
- **Deployment**: Full site rebuild (~2-3 minutes)
- **Protection**: Requires review for code changes

## 📄 **Content Structure**

### **File Organization**
```
data/blog/
├── posts/           # Published posts
│   ├── 2024-09-23-welcome.md
│   └── 2024-09-24-rust-tips.md
├── drafts/          # Draft posts
│   └── 2024-09-25-work-in-progress.md
└── templates/       # Blog post templates
    └── template.md
```

### **Post Format**
```markdown
---
title: "Your Post Title"
date: 2024-09-23
tags: ["rust", "web", "tutorial"]
slug: "your-post-title"
published: true
excerpt: "A brief description for listings and meta tags."
author: "Your Name"
---

# Your Post Title

Your markdown content here...

## Sections

- Use standard markdown
- Include code blocks
- Add images and links
- Write engaging content

```

## 🚀 **Deployment Workflow**

### **Blog Branch Push → Fast Deploy**
1. **Detect**: Only `data/blog/` content changed
2. **Build**: Fast blog-only regeneration (skips PDF, CV)
3. **Deploy**: Push to GitHub Pages
4. **Sync**: Auto-merge blog content to main branch
5. **Time**: ~30 seconds total

### **Main Branch Push → Full Deploy**
1. **Detect**: Code, templates, or infrastructure changed
2. **Build**: Complete site regeneration
3. **Deploy**: Full site deployment
4. **Time**: ~2-3 minutes total

## 🛠️ **Common Workflows**

### **Daily Blogging**
```bash
# Morning routine
git checkout blog
git pull origin blog

# Write new post
./scripts/blog-new.sh "Today's Topic"
# Edit the file...

# Publish
git add .
git commit -m "blog: add daily post about topic"
git push origin blog
# → Live in 30 seconds
```

### **Batch Content Updates**
```bash
# Switch to blog branch
git checkout blog

# Edit multiple posts, add new ones, remove old ones
./scripts/blog-new.sh "Post 1"
./scripts/blog-new.sh "Post 2"
# Edit existing posts...

# Commit all changes atomically
git add data/blog/
git commit -m "blog: content update - 2 new posts, 3 updates"
git push origin blog
# → All changes deployed together
```

### **Draft Workflow**
```bash
# Create draft
./scripts/blog-new.sh --draft "Future Post"

# Work on it over time
git add .
git commit -m "blog: update draft with new sections"
git push origin blog
# → Draft saved but not published

# When ready
./scripts/blog-publish.sh "future-post"
git add .
git commit -m "blog: publish future post"
git push origin blog
# → Now live!
```

## 🔧 **Advanced Features**

### **Content Scheduling**
```yaml
# In post frontmatter:
date: 2024-12-01  # Future date
published: true
scheduled: true   # Will show after date
```

### **Multi-Author Support**
```yaml
# In post frontmatter:
author: "Guest Author"
author_email: "guest@example.com"
co_authors: ["Author 2", "Author 3"]
```

### **Rich Metadata**
```yaml
# In post frontmatter:
image: "/images/post-banner.jpg"
featured: true
series: "Rust Tutorial Series"
reading_time: 10
```

## 🔒 **Safety Features**

### **Built-in Backup**
- Every change is version controlled
- Complete history preserved in Git
- Easy rollback: `git revert <commit>`
- Multiple copies: GitHub, local clones

### **Atomic Operations**
- Each commit is an atomic content change
- No partial updates or broken states
- Rollback entire content updates if needed

### **Collaboration Safety**
- Git handles merge conflicts naturally
- Branch protection rules prevent accidents
- Review process for sensitive changes

## 📊 **Performance Benefits**

### **Blog-Only Updates (30 seconds)**
- ✅ Markdown parsing and HTML generation
- ✅ Blog template rendering
- ✅ Static file copying
- ❌ Skips Rust compilation
- ❌ Skips PDF generation
- ❌ Skips CV data processing

### **Smart Caching**
- Cargo dependencies cached
- Build artifacts reused
- Only regenerates changed content

## 🆘 **Troubleshooting**

### **Build Failures**
```bash
# Check build locally
./scripts/blog-preview.sh

# View deployment logs
# Go to GitHub Actions tab in your repo
```

### **Content Not Appearing**
```bash
# Check frontmatter syntax
./scripts/blog-list.sh

# Verify published: true
grep -n "published:" data/blog/posts/your-post.md
```

### **Branch Issues**
```bash
# Sync with remote
git checkout blog
git pull origin blog

# Check current branch
git branch --show-current

# Force sync main with blog
git checkout main
git merge blog
git push origin main
```

## 🎨 **Content Creation Tips**

### **Writing Effective Posts**
1. **Start with a compelling title**
2. **Write a clear excerpt** (used in listings and SEO)
3. **Use descriptive tags** (helps with discovery)
4. **Structure with headers** (improves readability)
5. **Include code examples** (if technical content)

### **SEO Best Practices**
- Use descriptive slugs
- Write compelling excerpts
- Include relevant tags
- Add images with alt text
- Use proper heading structure

### **Performance Tips**
- Optimize images before adding
- Keep posts reasonably sized
- Use code syntax highlighting
- Link to other relevant posts

## 🌟 **Benefits Summary**

✅ **Static Site Performance** - Fast loading, CDN-friendly
✅ **CMS-like Convenience** - Easy content management
✅ **Developer-Friendly** - Git workflow, version control
✅ **Free Hosting** - GitHub Pages costs nothing
✅ **Scalable** - Add writers by giving Git access
✅ **Reliable** - Built-in backup and rollback
✅ **Fast Updates** - 30-second blog deployments
✅ **Collaborative** - Multiple writers, review process

**It's like having a headless CMS, but using Git as the database!** 🚀

---

*Need help? Check the scripts in `./scripts/blog-*.sh` or see the workflow documentation in `GIT_CMS_WORKFLOW.md`*