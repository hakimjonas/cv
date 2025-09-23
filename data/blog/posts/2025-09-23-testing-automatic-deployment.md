---
title: "Testing Automatic Deployment"
date: 2025-09-23
tags: ["automation", "git-as-cms", "testing"]
slug: "testing-automatic-deployment"
published: true
excerpt: "This post tests if new blog posts automatically trigger site rebuilds while maintaining proper branch separation."
author: "Hakim Jonas Ghoula"
---

# Testing Automatic Deployment

This post tests whether the Git-as-CMS workflow properly handles new blog posts automatically.

## What Should Happen

When this post is committed and pushed to the blog branch:

1. ✅ GitHub Actions detects blog-only changes
2. ✅ Triggers fast blog-only build (30 seconds)
3. ✅ Deploys updated static site automatically
4. ✅ Blog branch remains independent (no merge to main)

## Branch Separation Maintained

The workflow now properly maintains our dual-branch strategy:

- **Blog branch**: Living content database with all posts
- **Main branch**: Clean code infrastructure only

## Git-as-CMS Benefits

- 📝 Write posts in Markdown
- 🔄 Version-controlled content
- ⚡ Fast deployments for content changes
- 🛡️ Protected code infrastructure
- 🚀 Automatic static site regeneration

Perfect Git-as-CMS workflow in action! 🎉
