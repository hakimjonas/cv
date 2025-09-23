# 📝 Dual-Branch Blog Workflow

## 🎯 **The Vision**
Combine the **simplicity of static sites** with the **flexibility of dynamic blog management** using a smart dual-branch workflow.

## 🌳 **Branch Strategy**

### **`main` Branch - Code & Core Content**
- **Purpose**: Core application code, templates, styling, CV data
- **Triggers**: Major features, bug fixes, design changes
- **Deployment**: Full site rebuild and deploy
- **Protection**: Requires review for major changes

### **`blog` Branch - Blog Content Only**
- **Purpose**: Blog posts, drafts, content updates
- **Triggers**: New posts, post updates, content edits
- **Deployment**: Fast blog-only rebuild
- **Workflow**: Direct push allowed, auto-deploy

## ⚡ **Workflow Benefits**

### **For Developers (Code Changes)**
```bash
# Work on main branch for features
git checkout main
git pull origin main
# Make code changes
git add . && git commit -m "feat: new feature"
git push origin main
# → Triggers full site rebuild
```

### **For Content Writers (Blog Posts)**
```bash
# Work on blog branch for content
git checkout blog
git pull origin blog
# Add new blog post
echo "---\ntitle: My New Post\n---\n# Content" > data/blog/posts/2024-01-15-new-post.md
git add . && git commit -m "blog: add new post about topic"
git push origin blog
# → Triggers fast blog-only rebuild
```

## 🚀 **Deployment Strategy**

### **Option A: Separate Workflows**
- **`main` branch**: Full site generation (5-10 minutes)
- **`blog` branch**: Blog-only generation (1-2 minutes)

### **Option B: Smart Detection**
- Single workflow that detects changed files
- If only blog content changed → fast blog rebuild
- If code changed → full rebuild

### **Option C: Hybrid Auto-Merge**
- Blog changes automatically merge to main
- Deploys from main but with fast-path detection

## 🛠 **Implementation Plan**

### **Phase 1: Branch Setup**
1. Create `blog` branch from `main`
2. Protect both branches appropriately
3. Set up branch-specific workflows

### **Phase 2: Smart Workflows**
1. **Fast Blog Workflow**: Only regenerates blog when `data/blog/` changes
2. **Full Site Workflow**: Regenerates everything when code changes
3. **Auto-sync**: Keep branches in sync for blog content

### **Phase 3: Content Management Tools**
1. **Blog CLI**: `cargo run --bin blog -- new "Post Title"`
2. **Draft System**: Posts with `published: false`
3. **Preview System**: Deploy drafts to preview URL

## 📁 **File Organization**

```
data/blog/
├── posts/           # Published posts
│   ├── 2024-01-15-post.md
│   └── 2024-01-16-another.md
├── drafts/          # Draft posts (published: false)
│   └── 2024-01-17-draft.md
└── templates/       # Blog post templates
    └── template.md
```

## 🔄 **Auto-Sync Strategy**

### **Blog → Main Sync**
```yaml
# On blog branch push:
1. Fast blog rebuild
2. Deploy to staging
3. Auto-merge to main (if clean)
4. Deploy to production
```

### **Main → Blog Sync**
```yaml
# On main branch push:
1. Full site rebuild
2. Sync any new blog infrastructure to blog branch
3. Deploy to production
```

## ⚙️ **Workflow Commands**

### **Content Creator Workflow**
```bash
# Quick blog post
./scripts/blog-new.sh "My Post Title"
# → Creates draft, opens editor
# → Commits to blog branch
# → Auto-deploys

# Publish draft
./scripts/blog-publish.sh "my-post-slug"
# → Sets published: true
# → Pushes to blog branch
```

### **Developer Workflow**
```bash
# Feature development
git checkout main
# Make changes, test locally
cargo run --bin cv  # Test full build
git push origin main
# → Full rebuild and deploy
```

## 🎨 **Advanced Features**

### **1. Preview System**
- **Draft Posts**: Deploy to `preview.yourdomain.com`
- **PR Previews**: Each PR gets preview URL
- **Branch Previews**: `blog-branch.yourdomain.com`

### **2. Content Scheduling**
```yaml
# In frontmatter:
title: "Future Post"
date: 2024-06-01
published: true
scheduled: true  # Only publish after date
```

### **3. Multi-Author Support**
```yaml
# In frontmatter:
author: "Guest Author"
author_email: "guest@example.com"
co_authors: ["Author 2", "Author 3"]
```

### **4. Content Analytics**
- Track which posts are being updated
- Monitor build times for optimization
- Content performance metrics

## 🔐 **Security & Permissions**

### **Branch Protection**
- **Main**: Require PR review for code changes
- **Blog**: Allow direct push for content, but with checks
- **Auto-merge**: Only for blog content, with validation

### **Content Validation**
- Frontmatter schema validation
- Markdown linting
- Image optimization
- Link checking

## 📊 **Performance Optimization**

### **Fast Blog Builds**
```rust
// Only regenerate if blog content changed
if changed_files.iter().any(|f| f.starts_with("data/blog/")) {
    generate_blog_only();  // 30 seconds
} else {
    generate_full_site();  // 2 minutes
}
```

### **Incremental Updates**
- Only rebuild changed posts
- Smart asset copying
- Cached dependency builds

## 🎯 **End User Experience**

### **For Blog Authors**
1. **Simple**: Just edit markdown files
2. **Fast**: Changes live in 1-2 minutes
3. **Flexible**: Preview, schedule, collaborate
4. **Git-native**: All benefits of version control

### **For Site Maintainers**
1. **Clean**: Separate concerns (code vs content)
2. **Safe**: Code changes don't affect content workflow
3. **Efficient**: Fast content updates, thorough code reviews
4. **Scalable**: Add more content creators easily

This gives you **static site performance** with **CMS-like convenience**! 🚀