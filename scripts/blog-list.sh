#!/bin/bash
# List blog posts and drafts with metadata

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
BLOG_DIR="$PROJECT_ROOT/data/blog/posts"
DRAFTS_DIR="$PROJECT_ROOT/data/blog/drafts"

# Function to extract frontmatter field
extract_field() {
    local file="$1"
    local field="$2"
    local default="$3"

    if [[ -f "$file" ]]; then
        # Extract field from YAML frontmatter
        awk -v field="$field" -v default_val="$default" '
            /^---$/ && !in_fm { in_fm = 1; next }
            /^---$/ && in_fm { in_fm = 0; next }
            in_fm && $0 ~ "^" field ":" {
                sub("^" field ": *", "")
                gsub(/^["'\'']|["'\'']$/, "")  # Remove quotes
                print
                found = 1
                exit
            }
            END { if (!found) print default_val }
        ' "$file"
    else
        echo "$default"
    fi
}

# Function to list posts in a directory
list_posts() {
    local dir="$1"
    local title="$2"
    local status="$3"

    if [[ ! -d "$dir" ]]; then
        echo -e "${YELLOW}Directory not found: $dir${NC}"
        return
    fi

    echo -e "${BLUE}${title}${NC}"
    echo "$(printf '%*s' ${#title} '' | tr ' ' '=')"
    echo ""

    local count=0
    for file in "$dir"/*.md; do
        if [[ -f "$file" ]]; then
            local filename=$(basename "$file")
            local title_field=$(extract_field "$file" "title" "Untitled")
            local date_field=$(extract_field "$file" "date" "No date")
            local author_field=$(extract_field "$file" "author" "Unknown")
            local tags_field=$(extract_field "$file" "tags" "[]")
            local published_field=$(extract_field "$file" "published" "false")

            # Clean up tags (remove brackets and quotes)
            tags_field=$(echo "$tags_field" | sed 's/\[//g; s/\]//g; s/"//g; s/'\''//g; s/, */ /g')

            echo -e "${CYAN}📄 $filename${NC}"
            echo -e "   Title: ${YELLOW}$title_field${NC}"
            echo -e "   Date: $date_field"
            echo -e "   Author: $author_field"
            if [[ -n "$tags_field" && "$tags_field" != " " ]]; then
                echo -e "   Tags: $tags_field"
            fi
            if [[ "$status" == "posts" ]]; then
                if [[ "$published_field" == "true" ]]; then
                    echo -e "   Status: ${GREEN}Published${NC}"
                else
                    echo -e "   Status: ${YELLOW}Unpublished${NC}"
                fi
            else
                echo -e "   Status: ${YELLOW}Draft${NC}"
            fi
            echo ""

            ((count++))
        fi
    done

    if [[ $count -eq 0 ]]; then
        echo -e "${YELLOW}No $status found.${NC}"
        echo ""
    else
        echo -e "${GREEN}Total: $count $status${NC}"
        echo ""
    fi
}

# Function to show usage
usage() {
    echo -e "${BLUE}Blog Post Lister${NC}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -p, --posts     Show only published posts"
    echo "  -d, --drafts    Show only drafts"
    echo "  -s, --summary   Show summary statistics"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Show all posts and drafts"
    echo "  $0 --posts      # Show only published posts"
    echo "  $0 --drafts     # Show only drafts"
    echo "  $0 --summary    # Show summary statistics"
    echo ""
    exit 1
}

# Parse command line arguments
SHOW_POSTS=true
SHOW_DRAFTS=true
SHOW_SUMMARY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--posts)
            SHOW_POSTS=true
            SHOW_DRAFTS=false
            shift
            ;;
        -d|--drafts)
            SHOW_POSTS=false
            SHOW_DRAFTS=true
            shift
            ;;
        -s|--summary)
            SHOW_SUMMARY=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}" >&2
            usage
            ;;
    esac
done

# Create directories if they don't exist
mkdir -p "$BLOG_DIR" "$DRAFTS_DIR"

# Show summary if requested
if [[ "$SHOW_SUMMARY" == "true" ]]; then
    echo -e "${BLUE}📊 Blog Statistics${NC}"
    echo "=================="
    echo ""

    # Count posts
    published_count=$(find "$BLOG_DIR" -name "*.md" -type f 2>/dev/null | wc -l)
    draft_count=$(find "$DRAFTS_DIR" -name "*.md" -type f 2>/dev/null | wc -l)
    total_count=$((published_count + draft_count))

    echo -e "📚 Published posts: ${GREEN}$published_count${NC}"
    echo -e "📝 Draft posts: ${YELLOW}$draft_count${NC}"
    echo -e "📄 Total posts: ${CYAN}$total_count${NC}"
    echo ""

    # Show recent activity
    if [[ $published_count -gt 0 ]]; then
        echo -e "${BLUE}🕒 Recent Published Posts:${NC}"
        find "$BLOG_DIR" -name "*.md" -type f -printf '%T@ %p\n' 2>/dev/null | \
            sort -nr | head -3 | while read timestamp file; do
            title=$(extract_field "$file" "title" "Untitled")
            date_str=$(date -d "@$timestamp" "+%Y-%m-%d")
            echo -e "   • ${CYAN}$(basename "$file")${NC} - $title ($date_str)"
        done
        echo ""
    fi

    if [[ $draft_count -gt 0 ]]; then
        echo -e "${BLUE}✏️  Recent Drafts:${NC}"
        find "$DRAFTS_DIR" -name "*.md" -type f -printf '%T@ %p\n' 2>/dev/null | \
            sort -nr | head -3 | while read timestamp file; do
            title=$(extract_field "$file" "title" "Untitled")
            date_str=$(date -d "@$timestamp" "+%Y-%m-%d")
            echo -e "   • ${CYAN}$(basename "$file")${NC} - $title ($date_str)"
        done
        echo ""
    fi

    exit 0
fi

# Main listing
echo -e "${BLUE}📚 Blog Content Overview${NC}"
echo "========================="
echo ""

if [[ "$SHOW_POSTS" == "true" ]]; then
    list_posts "$BLOG_DIR" "Published Posts" "posts"
fi

if [[ "$SHOW_DRAFTS" == "true" ]]; then
    list_posts "$DRAFTS_DIR" "Draft Posts" "drafts"
fi

# Show quick tips
echo -e "${BLUE}💡 Quick Tips:${NC}"
echo "• Create new post: ./scripts/blog-new.sh \"Post Title\""
echo "• Publish draft: ./scripts/blog-publish.sh <slug>"
echo "• Build site: cargo run --bin cv"
echo "• Deploy: git add . && git commit -m \"blog: update\" && git push origin blog"
echo ""