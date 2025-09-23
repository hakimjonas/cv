# DevHub Context for AI Agents

This project uses DevHub for intelligent project analysis and context management.

## Available DevHub Tools (via MCP)

When working in this project, you have access to these DevHub tools:

1. **get-bundle-context** - Get complete project context including:
   - Project type and frameworks
   - File analysis with importance scoring
   - Tech stack detection
   - Development patterns
   - Git history and recent commits

2. **get-current-branch-context** - Auto-detect context from current git branch

3. **get-jira-issue** - Retrieve Jira ticket details (if configured)

4. **get-pr-details** - Get GitHub/GitLab PR information

5. **get-pr-comments** - Fetch unresolved PR review comments

## Project Intelligence

DevHub automatically analyzes:
- Project type (web app, library, CLI tool, etc.)
- Frameworks (React, Django, FastAPI, etc.)
- Tech stack (languages, databases, tools)
- Code organization patterns
- Testing approach
- Development stage

## Usage

You don't need to ask the user about project structure - use DevHub tools to understand it automatically.

Example: Instead of asking "What framework does this project use?", call get-bundle-context to know instantly.
