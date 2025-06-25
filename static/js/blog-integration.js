/**
 * Blog Integration Script
 * This script adds blog functionality to the index.html page
 * Using modern JavaScript patterns and best practices
 */

// Configuration options
const BLOG_CONFIG = {
  postsToShow: 6,
  dateFormat: { year: 'numeric', month: 'long', day: 'numeric' },
  language: 'en-US',
  loadingMessage: 'Loading posts...',
  errorMessage: 'Error loading posts. Please try again later.',
  noPostsMessage: 'No posts found.'
};

// Function to format date with better error handling
function formatDate(dateString) {
  if (!dateString) return '';

  try {
    const date = new Date(dateString);
    // Check if date is valid
    if (isNaN(date.getTime())) {
      console.warn(`Invalid date: ${dateString}`);
      return '';
    }

    return date.toLocaleDateString(BLOG_CONFIG.language, BLOG_CONFIG.dateFormat);
  } catch (error) {
    console.error(`Error formatting date: ${dateString}`, error);
    return dateString; // Return original string as fallback
  }
}

/**
 * Creates and inserts the blog section into the page
 * @returns {HTMLElement|null} The created blog section element or null if creation failed
 */
function createBlogSection() {
  try {
    // Add blog styles first
    addBlogStyles();

    // Find the introduction section
    const introSection = document.getElementById('introduction');
    if (!introSection) {
      console.warn('Introduction section not found, cannot insert blog section');
      return null;
    }

    // Create blog section with proper ARIA attributes
    const blogSection = document.createElement('section');
    blogSection.className = 'blog-section';
    blogSection.id = 'blog';
    blogSection.setAttribute('aria-labelledby', 'blog-section-title');

    // Create container
    const container = document.createElement('div');
    container.className = 'container';

    // Create title with ID for aria-labelledby
    const title = document.createElement('h2');
    title.className = 'section-title';
    title.id = 'blog-section-title';
    title.textContent = 'Latest Blog Posts';

    // Create posts container with loading state
    const postsContainer = document.createElement('div');
    postsContainer.id = 'blog-posts-container';
    postsContainer.className = 'blog-posts';
    postsContainer.setAttribute('role', 'feed');
    postsContainer.setAttribute('aria-busy', 'true');
    postsContainer.setAttribute('aria-live', 'polite');

    // Add loading indicator with proper accessibility
    const loadingDiv = document.createElement('div');
    loadingDiv.className = 'loading';
    loadingDiv.textContent = BLOG_CONFIG.loadingMessage;
    loadingDiv.setAttribute('role', 'status');
    postsContainer.appendChild(loadingDiv);

    // Assemble the section
    container.appendChild(title);
    container.appendChild(postsContainer);
    blogSection.appendChild(container);

    // Insert after introduction section
    introSection.parentNode.insertBefore(blogSection, introSection.nextSibling);

    // Load blog posts
    loadBlogPosts().catch(error => {
      console.error('Failed to load blog posts:', error);
      showErrorMessage(postsContainer, BLOG_CONFIG.errorMessage);
    });

    return blogSection;
  } catch (error) {
    console.error('Error creating blog section:', error);
    return null;
  }
}

/**
 * Shows an error message in the specified container
 * @param {HTMLElement} container - The container to show the error in
 * @param {string} message - The error message to display
 */
function showErrorMessage(container, message) {
  if (!container) return;

  container.setAttribute('aria-busy', 'false');
  container.innerHTML = `
    <div class="error" role="alert">
      <p>${message}</p>
      <button class="retry-button" onclick="loadBlogPosts()">
        <i class="nf nf-refresh" aria-hidden="true"></i> Retry
      </button>
    </div>
  `;
}

/**
 * Loads and displays blog posts
 * @returns {Promise<Array>} A promise that resolves to the loaded posts
 */
async function loadBlogPosts() {
  const postsContainer = document.getElementById('blog-posts-container');
  if (!postsContainer) {
    console.error('Posts container not found');
    return [];
  }

  // Update container state
  postsContainer.setAttribute('aria-busy', 'true');
  postsContainer.innerHTML = `
    <div class="loading" role="status">
      <span class="visually-hidden">Loading blog posts</span>
      ${BLOG_CONFIG.loadingMessage}
    </div>
  `;

  try {
    // Use our API client to get published posts with timeout
    const posts = await Promise.race([
      BlogAPI.getPublishedPosts(),
      new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Request timed out')), 10000)
      )
    ]);

    // Update container state
    postsContainer.setAttribute('aria-busy', 'false');

    if (!posts || posts.length === 0) {
      postsContainer.innerHTML = `
        <div class="no-posts" role="status">
          ${BLOG_CONFIG.noPostsMessage}
        </div>
      `;
      return [];
    }

    // Clear the container
    postsContainer.innerHTML = '';

    // Display only the most recent posts (up to configured limit)
    const recentPosts = posts.slice(0, BLOG_CONFIG.postsToShow);

    // Create a document fragment for better performance
    const fragment = document.createDocumentFragment();

    // Create post elements
    recentPosts.forEach((post, index) => {
      const postElement = createPostElement(post, index);
      fragment.appendChild(postElement);
    });

    // Add all posts to the container at once (better performance)
    postsContainer.appendChild(fragment);

    // Add event listeners to "Read more" links
    setupReadMoreListeners();

    return recentPosts;
  } catch (error) {
    console.error('Error loading posts:', error);
    showErrorMessage(postsContainer, 
      `${BLOG_CONFIG.errorMessage} ${error.message ? `(${error.message})` : ''}`
    );
    return [];
  }
}

/**
 * Creates a blog post element
 * @param {Object} post - The post data
 * @param {number} index - The index of the post in the list
 * @returns {HTMLElement} The created post element
 */
function createPostElement(post, index) {
  // Create article element with proper attributes
  const postElement = document.createElement('article');
  postElement.className = 'blog-card';
  postElement.dataset.slug = post.slug || '';
  postElement.setAttribute('role', 'article');
  postElement.setAttribute('aria-posinset', index + 1);
  postElement.setAttribute('aria-labelledby', `post-title-${post.slug || index}`);

  // Sanitize post data to prevent XSS
  const safePost = {
    title: sanitizeHTML(post.title || 'Untitled Post'),
    content: sanitizeHTML(post.content || ''),
    excerpt: sanitizeHTML(post.excerpt || (post.content ? post.content.substring(0, 150) + '...' : '')),
    created_at: post.created_at || new Date(),
    featured: post.featured || false,
    tags: post.tags || []
  };

  // Generate tags HTML
  let tagsHtml = '';
  if (safePost.tags.length > 0) {
    tagsHtml = safePost.tags
      .map(tag => `<span class="tag">${sanitizeHTML(tag.name)}</span>`)
      .join('');
  }

  // Generate featured status
  const featuredStatus = safePost.featured ? 
    '<span class="featured-badge" aria-label="Featured post">• Featured</span>' : '';

  // Create the HTML content
  postElement.innerHTML = `
    <div class="blog-card-content">
      <h3 class="blog-title" id="post-title-${post.slug || index}">${safePost.title}</h3>
      <div class="post-meta">
        <time datetime="${new Date(safePost.created_at).toISOString()}">
          ${formatDate(safePost.created_at)}
        </time>
        ${featuredStatus}
      </div>
      <div class="post-tags" aria-label="Tags">${tagsHtml}</div>
      <div class="blog-content">
        <p class="blog-excerpt">${safePost.excerpt}</p>
        <div class="full-content" style="display: none;">${safePost.content}</div>
      </div>
      <button class="read-more" data-action="expand" aria-expanded="false" aria-controls="full-content-${post.slug || index}">
        Read more <i class="nf nf-arrow-right"></i>
      </button>
    </div>
  `;

  // Set ID for the full content for ARIA controls
  const fullContent = postElement.querySelector('.full-content');
  fullContent.id = `full-content-${post.slug || index}`;

  return postElement;
}

/**
 * Sets up event listeners for "Read more" buttons
 */
function setupReadMoreListeners() {
  document.querySelectorAll('.read-more').forEach(button => {
    button.addEventListener('click', (e) => {
      e.preventDefault();

      // Get the parent blog card
      const blogCard = button.closest('.blog-card');

      // Get the excerpt and full content elements
      const excerpt = blogCard.querySelector('.blog-excerpt');
      const fullContent = blogCard.querySelector('.full-content');

      // Toggle between expanded and collapsed states
      const isExpanded = button.getAttribute('aria-expanded') === 'true';

      if (!isExpanded) {
        // Expand the content
        excerpt.style.display = 'none';
        fullContent.style.display = 'block';
        button.innerHTML = 'Show less <i class="nf nf-arrow-up"></i>';
        button.setAttribute('aria-expanded', 'true');

        // Add expanded class for styling
        blogCard.classList.add('expanded');

        // Scroll to the top of the card if it's not fully visible
        if (blogCard.getBoundingClientRect().top < 0) {
          blogCard.scrollIntoView({ 
            behavior: 'smooth',
            block: 'start'
          });
        }
      } else {
        // Collapse the content
        excerpt.style.display = 'block';
        fullContent.style.display = 'none';
        button.innerHTML = 'Read more <i class="nf nf-arrow-right"></i>';
        button.setAttribute('aria-expanded', 'false');

        // Remove expanded class
        blogCard.classList.remove('expanded');
      }
    });
  });
}

/**
 * Sanitizes HTML to prevent XSS attacks
 * @param {string} html - The HTML to sanitize
 * @returns {string} The sanitized HTML
 */
function sanitizeHTML(html) {
  if (!html) return '';

  const temp = document.createElement('div');
  temp.textContent = html;
  return temp.innerHTML;
}

/**
 * Adds styling for blog posts with modern CSS techniques
 * Uses CSS custom properties, grid layout, and accessibility improvements
 */
function addBlogStyles() {
  // Only add styles once
  if (document.querySelector('style#blog-styles')) return;

  const style = document.createElement('style');
  style.id = 'blog-styles';

  // Use CSS custom properties from the main theme for consistent styling
  style.textContent = `
    /* Blog layout using CSS Grid */
    .blog-posts {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
      gap: var(--spacing-lg);
      margin-top: var(--spacing-lg);
    }

    /* Blog card with modern styling */
    .blog-card {
      --card-padding: var(--spacing-md);
      background-color: var(--color-card-background);
      border-radius: var(--border-radius-lg);
      box-shadow: var(--color-card-shadow);
      padding: var(--card-padding);
      transition: transform 0.3s ease, box-shadow 0.3s ease;
      height: auto;
      overflow: hidden;
      width: 100%;
      display: flex;
      flex-direction: column;
    }

    /* Hover and focus states for accessibility */
    .blog-card:hover,
    .blog-card:focus-within {
      transform: translateY(-5px);
      box-shadow: var(--color-card-shadow);
    }

    /* Expanded state styling with grid span */
    .blog-card.expanded {
      grid-column: 1 / -1; /* Span all columns when expanded */
      max-width: 800px; /* Limit width for readability */
      margin: 0 auto;
      transform: none;
    }

    /* Blog title with proper spacing */
    .blog-title {
      margin-top: 0;
      margin-bottom: var(--spacing-sm);
      color: var(--color-text);
      font-size: var(--font-size-lg);
      line-height: 1.3;
    }

    /* Post metadata styling */
    .post-meta {
      color: var(--color-text-light);
      font-size: var(--font-size-sm);
      margin-bottom: var(--spacing-sm);
      display: flex;
      align-items: center;
      flex-wrap: wrap;
      gap: var(--spacing-xs);
    }

    /* Featured badge styling */
    .featured-badge {
      color: var(--color-accent);
      font-weight: 600;
    }

    /* Tags container */
    .post-tags {
      display: flex;
      flex-wrap: wrap;
      gap: var(--spacing-xs);
      margin-bottom: var(--spacing-sm);
    }

    /* Individual tag styling */
    .tag {
      display: inline-block;
      background-color: var(--color-secondary);
      color: white;
      padding: 3px 8px;
      border-radius: var(--border-radius-sm);
      font-size: var(--font-size-xs);
      line-height: 1.4;
    }

    /* Blog content container */
    .blog-content {
      margin-top: var(--spacing-sm);
      line-height: 1.6;
      flex-grow: 1; /* Take up available space */
      color: var(--color-text);
    }

    /* Blog excerpt styling */
    .blog-excerpt {
      margin-bottom: var(--spacing-sm);
    }

    /* Full content animation */
    .full-content {
      animation: fadeIn 0.5s ease;
      line-height: 1.7;
    }

    /* Read more button styling */
    .read-more {
      display: inline-flex;
      align-items: center;
      margin-top: var(--spacing-sm);
      color: var(--color-primary);
      font-weight: 600;
      text-decoration: none;
      padding: 8px 16px;
      border-radius: var(--border-radius-sm);
      background-color: var(--color-background-light);
      border: none;
      cursor: pointer;
      transition: background-color 0.3s ease, color 0.3s ease;
    }

    /* Hover, focus, and active states for accessibility */
    .read-more:hover,
    .read-more:focus {
      background-color: var(--color-primary);
      color: white;
      outline: none;
    }

    .read-more:focus {
      box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.3);
    }

    .read-more:active {
      transform: translateY(1px);
    }

    /* Loading and error states */
    .loading,
    .error,
    .no-posts {
      grid-column: 1 / -1;
      padding: var(--spacing-md);
      text-align: center;
      background-color: var(--color-background-light);
      border-radius: var(--border-radius-md);
      margin: var(--spacing-md) 0;
    }

    .error {
      color: var(--color-error);
      border-left: 4px solid var(--color-error);
    }

    /* Retry button styling */
    .retry-button {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      margin-top: var(--spacing-sm);
      padding: 8px 16px;
      background-color: var(--color-primary);
      color: white;
      border: none;
      border-radius: var(--border-radius-sm);
      cursor: pointer;
      transition: background-color 0.3s ease;
    }

    .retry-button:hover,
    .retry-button:focus {
      background-color: var(--color-secondary);
      outline: none;
    }

    /* Animation keyframes */
    @keyframes fadeIn {
      from { opacity: 0; }
      to { opacity: 1; }
    }

    /* Responsive adjustments */
    @media (max-width: 768px) {
      .blog-posts {
        grid-template-columns: 1fr;
      }

      .blog-card {
        max-width: 100%;
      }
    }
  `;

  // Add the styles to the document head
  document.head.appendChild(style);

  // Listen for theme changes and update styles accordingly
  document.addEventListener('themechange', function() {
    // Force a reflow to ensure CSS variables are applied
    void document.documentElement.offsetHeight;
  });
}

// Initialize when the DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
  // No need to check if nerdfonts are loaded since they're loaded directly in the HTML
  // Check if the API client is available
  if (typeof BlogAPI === 'undefined') {
    // Load the API client script
    const script = document.createElement('script');
    script.src = 'js/api-client.js';
    script.onload = createBlogSection;
    document.head.appendChild(script);
  } else {
    createBlogSection();
  }
});
