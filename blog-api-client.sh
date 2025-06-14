#!/bin/bash

# Make this script executable
chmod +x "$0"

# Detect the blog API server port
detect_port() {
    for port in $(seq 3000 3010); do
        if curl -s http://localhost:$port >/dev/null; then
            echo $port
            return 0
        fi
    done
    echo "Could not detect blog API server port. Make sure it's running."
    exit 1
}

# Set port (use detected port if not specified)
PORT=${1:-$(detect_port)}
BASE_URL="http://localhost:$PORT"

echo "Blog API Client"
echo "---------------"
echo "Using API at: $BASE_URL"
echo 

# Menu function
show_menu() {
    echo "Select an operation:"
    echo "1. Get all posts"
    echo "2. Get post by slug"
    echo "3. Create new post"
    echo "4. Update post"
    echo "5. Delete post"
    echo "6. Change port"
    echo "0. Exit"
    echo -n "Enter choice [0-6]: "
    read choice

    case $choice in
        1) get_all_posts ;;
        2) get_post_by_slug ;;
        3) create_post ;;
        4) update_post ;;
        5) delete_post ;;
        6) change_port ;;
        0) exit 0 ;;
        *) echo "Invalid choice. Try again." && show_menu ;;
    esac
}

# Get all posts
get_all_posts() {
    echo "\nGetting all posts..."
    response=$(curl -s "$BASE_URL/api/blog")
    echo "$response" | jq .
    echo -n "Press Enter to continue"
    read
    show_menu
}

# Get post by slug
get_post_by_slug() {
    echo -n "\nEnter post slug: "
    read slug

    echo "Getting post with slug: $slug"
    response=$(curl -s "$BASE_URL/api/blog/$slug")
    echo "$response" | jq .
    echo -n "Press Enter to continue"
    read
    show_menu
}

# Create new post
create_post() {
    echo "\nCreating new post..."
    echo -n "Title: "
    read title
    echo -n "Slug (leave empty to generate from title): "
    read slug

    # Generate slug from title if not provided
    if [ -z "$slug" ]; then
        slug=$(echo "$title" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--/-/g')
        echo "Generated slug: $slug"
    fi

    echo -n "Content: "
    read content
    echo -n "Excerpt: "
    read excerpt
    echo -n "Author: "
    read author

    # Get today's date
    date=$(date +%Y-%m-%d)

    echo "Creating post with the following data:"
    echo "Title: $title"
    echo "Slug: $slug"
    echo "Date: $date"
    echo "Author: $author"
    echo "Excerpt: $excerpt"
    echo "Content: $content"
    echo -n "Proceed? (y/n): "
    read confirm

    if [ "$confirm" != "y" ]; then
        echo "Operation canceled."
        show_menu
        return
    fi

    post_data="{\
        \"title\": \"$title\",\
        \"slug\": \"$slug\",\
        \"date\": \"$date\",\
        \"author\": \"$author\",\
        \"excerpt\": \"$excerpt\",\
        \"content\": \"$content\",\
        \"published\": true,\
        \"featured\": false,\
        \"image\": null,\
        \"tags\": [],\
        \"metadata\": {}\
    }"

    echo "Sending request..."
    response=$(curl -s -X POST "$BASE_URL/api/blog" \
        -H "Content-Type: application/json" \
        -d "$post_data")

    echo "Response:"
    echo "$response" | jq .
    echo -n "Press Enter to continue"
    read
    show_menu
}

# Update post
update_post() {
    echo "\nUpdate existing post"
    echo -n "Enter slug of post to update: "
    read slug

    # Get current post data
    current_post=$(curl -s "$BASE_URL/api/blog/$slug")
    if [[ "$current_post" == *"Not Found"* ]]; then
        echo "Post not found: $slug"
        echo -n "Press Enter to continue"
        read
        show_menu
        return
    fi

    # Extract current values
    current_title=$(echo "$current_post" | jq -r '.title')
    current_content=$(echo "$current_post" | jq -r '.content')
    current_excerpt=$(echo "$current_post" | jq -r '.excerpt')
    current_author=$(echo "$current_post" | jq -r '.author')
    current_date=$(echo "$current_post" | jq -r '.date')
    current_published=$(echo "$current_post" | jq -r '.published')
    current_featured=$(echo "$current_post" | jq -r '.featured')

    echo "Current post data:"
    echo "Title: $current_title"
    echo "Content: $current_content"
    echo "Excerpt: $current_excerpt"
    echo "Author: $current_author"
    echo "Date: $current_date"
    echo "Published: $current_published"
    echo "Featured: $current_featured"

    echo "\nEnter new values (press Enter to keep current value):"
    echo -n "Title: "
    read title
    title=${title:-$current_title}

    echo -n "Content: "
    read content
    content=${content:-$current_content}

    echo -n "Excerpt: "
    read excerpt
    excerpt=${excerpt:-$current_excerpt}

    echo -n "Author: "
    read author
    author=${author:-$current_author}

    post_data="{\
        \"title\": \"$title\",\
        \"slug\": \"$slug\",\
        \"date\": \"$current_date\",\
        \"author\": \"$author\",\
        \"excerpt\": \"$excerpt\",\
        \"content\": \"$content\",\
        \"published\": $current_published,\
        \"featured\": $current_featured,\
        \"image\": null,\
        \"tags\": [],\
        \"metadata\": {}\
    }"

    echo "Sending update request..."
    response=$(curl -s -X PUT "$BASE_URL/api/blog/$slug" \
        -H "Content-Type: application/json" \
        -d "$post_data")

    echo "Response:"
    echo "$response" | jq .
    echo -n "Press Enter to continue"
    read
    show_menu
}

# Delete post
delete_post() {
    echo "\nDelete a post"
    echo -n "Enter slug of post to delete: "
    read slug

    echo "Are you sure you want to delete the post with slug '$slug'?"
    echo -n "Confirm deletion (y/n): "
    read confirm

    if [ "$confirm" != "y" ]; then
        echo "Deletion canceled."
        show_menu
        return
    fi

    echo "Deleting post..."
    response=$(curl -s -X DELETE "$BASE_URL/api/blog/$slug")

    echo "Response:"
    if [ -z "$response" ]; then
        echo "Post deleted successfully."
    else
        echo "$response" | jq .
    fi

    echo -n "Press Enter to continue"
    read
    show_menu
}

# Change port
change_port() {
    echo "\nCurrent port: $PORT"
    echo -n "Enter new port: "
    read new_port

    if [[ $new_port =~ ^[0-9]+$ ]]; then
        PORT=$new_port
        BASE_URL="http://localhost:$PORT"
        echo "API URL updated to: $BASE_URL"
    else
        echo "Invalid port number."
    fi

    echo -n "Press Enter to continue"
    read
    show_menu
}

# Start the program
show_menu
