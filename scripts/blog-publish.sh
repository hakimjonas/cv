#!/bin/bash
# Publish a draft blog post by moving it from drafts to posts

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

# Function to show usage
usage() {
    echo -e "${BLUE}Blog Post Publisher${NC}"
    echo ""
    echo "Usage: $0 <post-slug>"
    echo ""
    echo "This script moves a draft post from drafts/ to posts/ and sets published: true"
    echo ""
    echo "Examples:"
    echo "  $0 my-draft-post"
    echo "  $0 2024-01-15-my-post"
    echo ""
    exit 1
}

# Check if slug was provided
if [[ $# -eq 0 ]]; then
    echo -e "${RED}Error: Post slug is required${NC}" >&2
    usage
fi

SLUG="$1"

# Find the draft file (try with and without date prefix)
DRAFT_FILE=""
for file in "$DRAFTS_DIR"/*"$SLUG"*.md; do
    if [[ -f "$file" ]]; then
        DRAFT_FILE="$file"
        break
    fi
done

if [[ -z "$DRAFT_FILE" ]]; then
    echo -e "${RED}Error: Draft file not found for slug: $SLUG${NC}" >&2
    echo -e "${YELLOW}Available drafts:${NC}"
    ls -1 "$DRAFTS_DIR"/*.md 2>/dev/null | sed 's|.*/||' | sed 's|\.md$||' || echo "  (no drafts found)"
    exit 1
fi

# Extract filename
FILENAME=$(basename "$DRAFT_FILE")
TARGET_FILE="$BLOG_DIR/$FILENAME"

# Check if published file already exists
if [[ -f "$TARGET_FILE" ]]; then
    echo -e "${RED}Error: Published file already exists: $TARGET_FILE${NC}" >&2
    exit 1
fi

# Read the draft content and update published flag
echo -e "${BLUE}Publishing draft: ${YELLOW}$FILENAME${NC}"

# Use sed to update the published flag
if grep -q "^published:" "$DRAFT_FILE"; then
    # Update existing published field
    sed 's/^published: false$/published: true/' "$DRAFT_FILE" > "$TARGET_FILE"
else
    # Add published field after the frontmatter header
    awk '
        /^---$/ && !first_marker { first_marker = 1; print; next }
        /^---$/ && first_marker && !second_marker {
            print "published: true"
            second_marker = 1
        }
        { print }
    ' "$DRAFT_FILE" > "$TARGET_FILE"
fi

# Remove the draft file
rm "$DRAFT_FILE"

# Success message
echo -e "${GREEN}✅ Blog post published successfully!${NC}"
echo ""
echo -e "${BLUE}Details:${NC}"
echo -e "  From: ${YELLOW}$DRAFT_FILE${NC}"
echo -e "  To: ${YELLOW}$TARGET_FILE${NC}"
echo ""

# Check if we're on the right branch
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
if [[ "$CURRENT_BRANCH" != "blog" ]]; then
    echo -e "${YELLOW}💡 Tip: Consider switching to the 'blog' branch for content changes:${NC}"
    echo "   git checkout blog"
    echo ""
fi

# Offer to commit the changes
echo -e "${BLUE}Next steps:${NC}"
echo "1. Review the published post: $TARGET_FILE"
echo "2. Commit and push to trigger deployment:"
echo "   git add ."
echo "   git commit -m \"blog: publish $FILENAME\""
echo "   git push origin blog"
echo ""

# Auto-stage the changes if user confirms
read -p "Stage changes for commit? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git add "$TARGET_FILE"
    if [[ -f "$DRAFT_FILE" ]]; then
        git rm "$DRAFT_FILE" 2>/dev/null || true
    fi
    echo -e "${GREEN}Changes staged. Ready to commit!${NC}"
    echo ""
    echo "Suggested commit message:"
    echo -e "${YELLOW}blog: publish $(basename "$FILENAME" .md)${NC}"
fi

echo -e "${GREEN}Happy publishing! 📚${NC}"