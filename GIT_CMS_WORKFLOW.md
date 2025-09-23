# 🗃️ Git-as-CMS Blog Workflow

## 🎯 **The Vision: Git = Database**

Transform Git into a powerful, version-controlled CMS where:
- **Blog branch** = Your content database
- **Commits** = Database transactions
- **Push** = Publish/deploy
- **History** = Built-in backup/rollback
- **Collaboration** = Multiple writers naturally supported

## 🌳 **Simple Branch Strategy**

```
main branch:    [Application Code] + [Templates] + [Styling]
                ↓
               Full rebuild (2-3 minutes)

blog branch:    [Content Database] = data/blog/**/*.md
                ↓
               Fast blog rebuild (30 seconds)
               ↓
               Auto-sync to main
```

## ⚡ **Content Operations = Git Operations**

### **Create Post**
```bash
git checkout blog
echo "---\ntitle: New Post\n---\n# Content" > data/blog/posts/2024-01-15-new-post.md
git add data/blog/posts/2024-01-15-new-post.md
git commit -m "blog: add post about new feature"
git push origin blog
# → Triggers fast deployment
```

### **Edit Post**
```bash
git checkout blog
vim data/blog/posts/2024-01-15-new-post.md
git add data/blog/posts/2024-01-15-new-post.md
git commit -m "blog: update post with examples"
git push origin blog
# → Fast redeploy with changes
```

### **Delete Post**
```bash
git checkout blog
git rm data/blog/posts/2024-01-15-old-post.md
git commit -m "blog: remove outdated post"
git push origin blog
# → Post removed from site
```

### **Bulk Operations**
```bash
git checkout blog
# Edit multiple files, add new ones, remove others
git add data/blog/
git commit -m "blog: content update - 3 new posts, 2 updates, 1 removal"
git push origin blog
# → All changes deployed atomically
```

## 🔄 **Automated Workflow**

### **Blog Branch Push**:
1. **Detect**: Only `data/blog/` changed
2. **Build**: Fast blog-only rebuild (skip PDF, CV generation)
3. **Deploy**: Push to GitHub Pages
4. **Sync**: Auto-merge to main (optional)
5. **Time**: ~30 seconds total

### **Main Branch Push**:
1. **Detect**: Code/template changes
2. **Build**: Full site rebuild
3. **Deploy**: Complete regeneration
4. **Time**: ~2-3 minutes total

## 🛠️ **Simple Tooling**

### **Blog Creation Helper**
```bash
# Simple script for new posts
./scripts/new-post.sh "My Post Title"
# → Creates file, opens editor, helps with commit
```

### **Content Management**
```bash
# List posts
./scripts/list-posts.sh

# Publish draft (move from drafts/ to posts/)
./scripts/publish.sh "post-slug"

# Archive old post
./scripts/archive.sh "post-slug"
```

## 🔒 **Safety & Reliability**

### **Built-in Backup**
- Every change is versioned
- Complete history preserved
- Easy rollback: `git revert <commit>`
- Multiple backups: GitHub, local clones

### **Atomic Operations**
- Each commit = atomic change
- No partial updates
- Consistent state always maintained

### **Collaboration Ready**
- Multiple writers can work simultaneously
- Git handles conflicts naturally
- Branch protection rules for quality control

## 📊 **Performance Benefits**

### **Fast Blog Updates**
```yaml
Blog-only changes:
- Skip: Rust compilation, CV generation, PDF creation
- Run: Blog markdown → HTML conversion only
- Time: 30 seconds vs 3 minutes (6x faster)
```

### **Smart Caching**
- Rust toolchain cached
- Dependencies cached
- Only regenerate what changed

## 🎨 **Content Creator Experience**

### **Local Development**
```bash
# Quick local preview
git checkout blog
./scripts/new-post.sh "Draft Post"
cargo run --bin cv  # Local build
# Preview at dist/blog.html

# When ready
git add . && git commit -m "blog: add draft post"
git push origin blog
# → Live in 30 seconds
```

### **Web-based Editing**
- Edit directly on GitHub web interface
- Commit directly through GitHub UI
- Perfect for quick edits and fixes

### **Mobile Publishing**
- Use GitHub mobile app
- Edit markdown files directly
- Commit and push from phone

## 🔧 **Implementation Plan**

### **Phase 1: Branch Setup**
```bash
# Create blog branch
git checkout -b blog
git push -u origin blog

# Set up branch protection
# - Allow direct pushes for blog content
# - Require status checks
# - Auto-merge to main (optional)
```

### **Phase 2: Smart Workflow**
- Update GitHub Actions for blog branch detection
- Implement fast blog-only builds
- Add auto-sync to main branch

### **Phase 3: Helper Scripts**
- Blog creation script
- Content management utilities
- Local preview helpers

## 🎯 **End Result**

You get a **static site** with **CMS-like convenience**:

✅ **Safe**: Version controlled, atomic operations
✅ **Simple**: Just edit markdown, commit, push
✅ **Fast**: 30-second blog updates
✅ **Flexible**: Full Git workflow (branches, PRs, collaboration)
✅ **Reliable**: Built-in backup, rollback, history
✅ **Scalable**: Add writers by giving Git access

**It's like having a headless CMS, but using Git as the database!** 🚀