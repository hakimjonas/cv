#!/bin/bash
# Blog post creation script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
BLOG_DIR="$PROJECT_ROOT/data/blog/posts"
DRAFTS_DIR="$PROJECT_ROOT/data/blog/drafts"

# Ensure directories exist
mkdir -p "$BLOG_DIR" "$DRAFTS_DIR"

# Function to generate slug from title
generate_slug() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-\|-$//g'
}

# Function to show usage
usage() {
    echo -e "${BLUE}Blog Post Creator${NC}"
    echo ""
    echo "Usage: $0 [OPTIONS] \"Post Title\""
    echo ""
    echo "Options:"
    echo "  -d, --draft     Create as draft (default: published)"
    echo "  -t, --tags      Comma-separated tags (e.g., \"rust,web,programming\")"
    echo "  -a, --author    Author name (default: from git config)"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 \"My New Blog Post\""
    echo "  $0 --draft \"Work in Progress Post\""
    echo "  $0 --tags \"rust,programming\" \"Learning Rust\""
    echo ""
    exit 1
}

# Default values
PUBLISHED=true
TAGS=""
AUTHOR=""
TITLE=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--draft)
            PUBLISHED=false
            shift
            ;;
        -t|--tags)
            TAGS="$2"
            shift 2
            ;;
        -a|--author)
            AUTHOR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [[ -z "$TITLE" ]]; then
                TITLE="$1"
            else
                echo -e "${RED}Error: Multiple titles provided. Use quotes for titles with spaces.${NC}" >&2
                exit 1
            fi
            shift
            ;;
    esac
done

# Check if title was provided
if [[ -z "$TITLE" ]]; then
    echo -e "${RED}Error: Post title is required${NC}" >&2
    usage
fi

# Get author from git config if not provided
if [[ -z "$AUTHOR" ]]; then
    AUTHOR=$(git config user.name 2>/dev/null || echo "Anonymous")
fi

# Generate filename
DATE=$(date +%Y-%m-%d)
SLUG=$(generate_slug "$TITLE")
FILENAME="$DATE-$SLUG.md"

# Determine target directory
if [[ "$PUBLISHED" == "true" ]]; then
    TARGET_DIR="$BLOG_DIR"
    STATUS="published"
else
    TARGET_DIR="$DRAFTS_DIR"
    STATUS="draft"
fi

FILEPATH="$TARGET_DIR/$FILENAME"

# Check if file already exists
if [[ -f "$FILEPATH" ]]; then
    echo -e "${RED}Error: File already exists: $FILEPATH${NC}" >&2
    exit 1
fi

# Process tags
TAGS_YAML=""
if [[ -n "$TAGS" ]]; then
    TAGS_YAML="tags: ["
    IFS=',' read -ra TAG_ARRAY <<< "$TAGS"
    for i in "${!TAG_ARRAY[@]}"; do
        TAG=$(echo "${TAG_ARRAY[$i]}" | xargs)  # Trim whitespace
        if [[ $i -eq 0 ]]; then
            TAGS_YAML+="\"$TAG\""
        else
            TAGS_YAML+=", \"$TAG\""
        fi
    done
    TAGS_YAML+="]"
else
    TAGS_YAML="tags: []"
fi

# Create blog post content
cat > "$FILEPATH" << EOF
---
title: "$TITLE"
date: $DATE
$TAGS_YAML
slug: "$SLUG"
published: $PUBLISHED
excerpt: "A brief description of your post. This will appear in blog listings and meta descriptions."
author: "$AUTHOR"
---

# $TITLE

Write your blog post content here using Markdown.

## Introduction

Start with an engaging introduction that hooks your readers.

## Main Content

Add your main content here. You can use:

- **Bold text**
- *Italic text*
- \`Inline code\`
- [Links](https://example.com)

### Code Blocks

\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

### Lists

1. Numbered lists
2. Are great for
3. Step-by-step content

### Quotes

> Use blockquotes for important information or quotes from other sources.

## Conclusion

Wrap up your post with a conclusion that summarizes key points or calls for action.

---

*Happy coding! 🦀*
EOF

# Success message
echo -e "${GREEN}✅ Blog post created successfully!${NC}"
echo ""
echo -e "${BLUE}Details:${NC}"
echo -e "  Title: ${YELLOW}$TITLE${NC}"
echo -e "  Status: ${YELLOW}$STATUS${NC}"
echo -e "  File: ${YELLOW}$FILEPATH${NC}"
echo -e "  Slug: ${YELLOW}$SLUG${NC}"
if [[ -n "$TAGS" ]]; then
    echo -e "  Tags: ${YELLOW}$TAGS${NC}"
fi
echo ""

# Offer to open in editor
echo -e "${BLUE}Next steps:${NC}"
echo "1. Edit the post content in: $FILEPATH"
if [[ "$PUBLISHED" == "false" ]]; then
    echo "2. When ready to publish, move to: $BLOG_DIR/"
    echo "   Or use: ./scripts/blog-publish.sh \"$SLUG\""
fi
echo "3. Commit and push to trigger deployment"
echo ""

# Check if we're on the right branch
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
if [[ "$CURRENT_BRANCH" != "blog" && "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "${YELLOW}💡 Tip: Consider switching to the 'blog' branch for content-only changes:${NC}"
    echo "   git checkout -b blog || git checkout blog"
    echo ""
fi

# Ask if user wants to open the file
if command -v code >/dev/null 2>&1; then
    read -p "Open in VS Code? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        code "$FILEPATH"
    fi
elif [[ -n "$EDITOR" ]]; then
    read -p "Open in \$EDITOR ($EDITOR)? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        $EDITOR "$FILEPATH"
    fi
fi

echo -e "${GREEN}Happy writing! ✍️${NC}"