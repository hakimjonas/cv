# Markdown Editor Documentation

## Overview

The blog client now includes a markdown editor that allows you to write and preview markdown content. This document explains how to use the markdown editor and the content format selection feature.

## Features

- **Content Format Selection**: Choose between HTML and Markdown formats for your blog posts
- **Markdown Editor**: A full-featured markdown editor with toolbar for common formatting options
- **Live Preview**: See how your markdown will render as you type
- **Side-by-Side Mode**: Edit and preview your markdown content simultaneously
- **Fullscreen Mode**: Focus on your writing with a distraction-free fullscreen editor

## Using the Markdown Editor

### Creating a New Post

1. Navigate to the blog client at http://localhost:3002/static/blog-client.html
2. Click on the "Create Post" tab
3. Fill in the title and other fields as usual
4. Under "Content Format", select "Markdown"
5. The markdown editor will appear, replacing the standard textarea
6. Use the toolbar buttons to format your content or type markdown syntax directly
7. Click "Create Post" to save your post with markdown content

### Editing an Existing Post

1. Navigate to the blog client at http://localhost:3002/static/blog-client.html
2. Click on the "Edit Post" tab
3. Select a post from the dropdown menu
4. If the post was created with markdown, the "Markdown" format will be automatically selected
5. Edit your markdown content using the editor
6. Click "Update Post" to save your changes

## Markdown Editor Toolbar

The markdown editor includes a toolbar with the following options:

- **Bold**: Make text bold (`**bold**`)
- **Italic**: Make text italic (`*italic*`)
- **Heading**: Create headings (`# Heading`)
- **Quote**: Create a blockquote (`> quote`)
- **Unordered List**: Create a bulleted list (`- item`)
- **Ordered List**: Create a numbered list (`1. item`)
- **Link**: Insert a link (`[text](url)`)
- **Image**: Insert an image (`![alt](url)`)
- **Code**: Insert code blocks or inline code
- **Table**: Insert a markdown table
- **Preview**: Toggle preview mode
- **Side-by-Side**: Toggle side-by-side editing and preview
- **Fullscreen**: Toggle fullscreen mode
- **Guide**: Show markdown syntax guide

## Preview Modes

The markdown editor offers three preview modes:

1. **No Preview**: Just the editor is visible
2. **Preview**: Only the rendered HTML is visible
3. **Side-by-Side**: Editor and preview are shown side by side

To toggle between these modes, use the preview and side-by-side buttons in the toolbar.

## Markdown Syntax

The editor supports GitHub Flavored Markdown, which includes:

- Headings: `# H1`, `## H2`, etc.
- Emphasis: `*italic*`, `**bold**`
- Lists: `- item` or `1. item`
- Links: `[text](url)`
- Images: `![alt](url)`
- Code: `` `inline code` `` or ``` ```code block``` ```
- Tables: 
  ```
  | Header 1 | Header 2 |
  | -------- | -------- |
  | Cell 1   | Cell 2   |
  ```
- Blockquotes: `> quote`
- Horizontal rules: `---`

For a complete guide, click the "Guide" button in the toolbar.

## Technical Implementation

The markdown editor is implemented using [EasyMDE](https://github.com/Ionaru/easy-markdown-editor), a JavaScript markdown editor based on CodeMirror. The editor is initialized when the "Markdown" format is selected and hidden when "HTML" is selected.

When a post is created or updated, the content format is saved along with the content. When editing a post, the correct editor is displayed based on the saved content format.

## Troubleshooting

If you encounter issues with the markdown editor:

1. **Editor not appearing**: Make sure you've selected "Markdown" as the content format
2. **Preview not updating**: Try clicking the preview button again or refreshing the page
3. **Content not saving**: Check the browser console for errors and make sure you're logged in

If problems persist, try clearing your browser cache and reloading the page.