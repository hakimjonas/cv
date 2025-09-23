#!/bin/bash
# Build and preview blog locally

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
DIST_DIR="$PROJECT_ROOT/dist"

# Function to show usage
usage() {
    echo -e "${BLUE}Blog Preview Tool${NC}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -o, --open      Open browser after build"
    echo "  -w, --watch     Watch for changes and rebuild (requires inotify-tools)"
    echo "  -s, --serve     Start simple HTTP server after build"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Build and show preview info"
    echo "  $0 --open       # Build and open in browser"
    echo "  $0 --serve      # Build and start HTTP server"
    echo "  $0 --watch      # Watch for changes and rebuild"
    echo ""
    exit 1
}

# Parse command line arguments
OPEN_BROWSER=false
WATCH_MODE=false
SERVE_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--open)
            OPEN_BROWSER=true
            shift
            ;;
        -w|--watch)
            WATCH_MODE=true
            shift
            ;;
        -s|--serve)
            SERVE_MODE=true
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

# Function to build the site
build_site() {
    echo -e "${BLUE}🔨 Building site...${NC}"
    cd "$PROJECT_ROOT"

    if cargo run --bin cv; then
        echo -e "${GREEN}✅ Build successful!${NC}"
        return 0
    else
        echo -e "${RED}❌ Build failed!${NC}"
        return 1
    fi
}

# Function to start HTTP server
start_server() {
    local port=${1:-8000}

    echo -e "${BLUE}🌐 Starting HTTP server on port $port...${NC}"
    cd "$DIST_DIR"

    # Try different HTTP servers
    if command -v python3 >/dev/null 2>&1; then
        echo -e "${YELLOW}Using Python3 HTTP server${NC}"
        python3 -m http.server "$port"
    elif command -v python >/dev/null 2>&1; then
        echo -e "${YELLOW}Using Python HTTP server${NC}"
        python -m SimpleHTTPServer "$port"
    elif command -v node >/dev/null 2>&1; then
        if command -v npx >/dev/null 2>&1; then
            echo -e "${YELLOW}Using Node.js http-server${NC}"
            npx http-server -p "$port"
        else
            echo -e "${RED}Node.js found but npx not available${NC}"
            exit 1
        fi
    else
        echo -e "${RED}No HTTP server available. Install python3 or node.js${NC}"
        exit 1
    fi
}

# Function to open browser
open_browser() {
    local url="http://localhost:8000"

    echo -e "${BLUE}🌐 Opening browser...${NC}"

    if command -v xdg-open >/dev/null 2>&1; then
        xdg-open "$url" >/dev/null 2>&1 &
    elif command -v open >/dev/null 2>&1; then
        open "$url" >/dev/null 2>&1 &
    elif command -v firefox >/dev/null 2>&1; then
        firefox "$url" >/dev/null 2>&1 &
    elif command -v google-chrome >/dev/null 2>&1; then
        google-chrome "$url" >/dev/null 2>&1 &
    else
        echo -e "${YELLOW}Could not detect browser. Open manually: $url${NC}"
    fi
}

# Function to watch for changes
watch_changes() {
    if ! command -v inotifywait >/dev/null 2>&1; then
        echo -e "${RED}inotifywait not found. Install inotify-tools:${NC}"
        echo "  sudo apt-get install inotify-tools  # Ubuntu/Debian"
        echo "  sudo yum install inotify-tools      # CentOS/RHEL"
        exit 1
    fi

    echo -e "${BLUE}👁️  Watching for changes...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""

    # Watch for changes in relevant directories
    inotifywait -m -r -e modify,create,delete,move \
        --include '\.(md|rs|html|css|js|toml)$' \
        "$PROJECT_ROOT/src" \
        "$PROJECT_ROOT/data" \
        "$PROJECT_ROOT/templates" \
        "$PROJECT_ROOT/static" \
        "$PROJECT_ROOT/Cargo.toml" 2>/dev/null | \
    while read path action file; do
        echo -e "${YELLOW}📝 Change detected: $action $file${NC}"
        if build_site; then
            echo -e "${GREEN}🔄 Site rebuilt successfully${NC}"
        else
            echo -e "${RED}💥 Build failed${NC}"
        fi
        echo ""
    done
}

# Main execution
echo -e "${BLUE}📚 Blog Preview Tool${NC}"
echo "==================="
echo ""

# Initial build
if ! build_site; then
    exit 1
fi

# Show build info
if [[ -d "$DIST_DIR" ]]; then
    echo ""
    echo -e "${BLUE}📁 Build output:${NC}"
    echo -e "   Directory: ${YELLOW}$DIST_DIR${NC}"
    if [[ -f "$DIST_DIR/blog.html" ]]; then
        echo -e "   Blog index: ${GREEN}✓${NC} $DIST_DIR/blog.html"
    fi
    if [[ -d "$DIST_DIR/blog" ]]; then
        post_count=$(find "$DIST_DIR/blog" -name "*.html" -type f 2>/dev/null | wc -l)
        echo -e "   Blog posts: ${GREEN}$post_count${NC} posts generated"
    fi
    echo ""
fi

# Handle different modes
if [[ "$WATCH_MODE" == "true" ]]; then
    watch_changes
elif [[ "$SERVE_MODE" == "true" ]]; then
    if [[ "$OPEN_BROWSER" == "true" ]]; then
        # Start server in background and open browser
        start_server 8000 &
        SERVER_PID=$!
        sleep 2  # Give server time to start
        open_browser
        echo -e "${YELLOW}Press Ctrl+C to stop server${NC}"
        wait $SERVER_PID
    else
        start_server 8000
    fi
elif [[ "$OPEN_BROWSER" == "true" ]]; then
    # Start server in background and open browser
    start_server 8000 &
    SERVER_PID=$!
    sleep 2
    open_browser
    echo -e "${YELLOW}Press Ctrl+C to stop server${NC}"
    wait $SERVER_PID
else
    # Just show preview info
    echo -e "${BLUE}🌐 Preview options:${NC}"
    echo "• View locally: file://$DIST_DIR/blog.html"
    echo "• Start server: $0 --serve"
    echo "• Watch changes: $0 --watch"
    echo "• Open browser: $0 --open"
    echo ""
    echo -e "${BLUE}💡 Quick commands:${NC}"
    echo "• List posts: ./scripts/blog-list.sh"
    echo "• New post: ./scripts/blog-new.sh \"Title\""
    echo "• Publish draft: ./scripts/blog-publish.sh <slug>"
fi