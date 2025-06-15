/**
 * Blog Integration Script
 * This script adds blog functionality to the index.html page
 */

// Function to format date
function formatDate(dateString) {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  });
}

// Function to create blog section
function createBlogSection() {
  // Find the introduction section
  const introSection = document.getElementById('introduction');
  if (!introSection) return;

  // Create blog section
  const blogSection = document.createElement('section');
  blogSection.className = 'blog-section';
  blogSection.id = 'blog';

  // Create container
  const container = document.createElement('div');
  container.className = 'container';

  // Create title
  const title = document.createElement('h2');
  title.className = 'section-title';
  title.textContent = 'Latest Blog Posts';

  // Create posts container
  const postsContainer = document.createElement('div');
  postsContainer.id = 'blog-posts-container';
  postsContainer.className = 'blog-posts';
  postsContainer.innerHTML = '<div class="loading">Loading posts...</div>';

  // Assemble the section
  container.appendChild(title);
  container.appendChild(postsContainer);
  blogSection.appendChild(container);

  // Insert after introduction section
  introSection.parentNode.insertBefore(blogSection, introSection.nextSibling);

  // Load blog posts
  loadBlogPosts();
}

// Function to load blog posts
async function loadBlogPosts() {
  const postsContainer = document.getElementById('blog-posts-container');
  if (!postsContainer) return;
  
  postsContainer.innerHTML = '<div class="loading">Loading posts...</div>';

  try {
    // Use our API client to get published posts
    const posts = await BlogAPI.getPublishedPosts();

    if (posts.length === 0) {
      postsContainer.innerHTML = '<div class="loading">No posts found.</div>';
      return;
    }

    postsContainer.innerHTML = '';

    // Display only the most recent posts (up to 6)
    const recentPosts = posts.slice(0, 6);

    recentPosts.forEach(post => {
      const postElement = document.createElement('article');
      postElement.className = 'blog-card';

      let tagsHtml = '';
      if (post.tags && post.tags.length > 0) {
        tagsHtml = post.tags.map(tag => 
          `<span class="tag">${tag.name}</span>`
        ).join('');
      }

      const featuredStatus = post.featured ? ' • Featured' : '';

      postElement.innerHTML = `
        <div class="blog-card-content">
          <h3 class="blog-title">${post.title}</h3>
          <div class="post-meta">${formatDate(post.created_at || new Date())}${featuredStatus}</div>
          <div class="post-tags">${tagsHtml}</div>
          <p class="blog-excerpt">${post.excerpt || post.content.substring(0, 150)}...</p>
          <a href="#" class="read-more" data-slug="${post.slug}">Read more</a>
        </div>
      `;

      postsContainer.appendChild(postElement);
    });

    // Add event listeners to "Read more" links
    document.querySelectorAll('.read-more').forEach(link => {
      link.addEventListener('click', async (e) => {
        e.preventDefault();
        const slug = link.dataset.slug;
        try {
          const post = await BlogAPI.getPost(slug);
          showPostDetails(post);
        } catch (error) {
          console.error('Error fetching post details:', error);
          alert(`Error: ${error.message}`);
        }
      });
    });

  } catch (error) {
    console.error('Error loading posts:', error);
    postsContainer.innerHTML = `<div class="error">Error loading posts: ${error.message}</div>`;
  }
}

// Function to show post details in a modal
function showPostDetails(post) {
  // Create modal container
  const modal = document.createElement('div');
  modal.className = 'modal';
  
  let tagsHtml = '';
  if (post.tags && post.tags.length > 0) {
    tagsHtml = post.tags.map(tag => 
      `<span class="tag">${tag.name}</span>`
    ).join('');
  }

  const featuredStatus = post.featured ? ' • Featured' : '';

  modal.innerHTML = `
    <div class="modal-content">
      <span class="close-modal">&times;</span>
      <h2>${post.title}</h2>
      <div class="post-meta">${formatDate(post.created_at || new Date())}${featuredStatus}</div>
      <div class="post-tags">${tagsHtml}</div>
      <div class="post-content">${post.content}</div>
    </div>
  `;

  document.body.appendChild(modal);

  // Add event listener to close button
  modal.querySelector('.close-modal').addEventListener('click', () => {
    document.body.removeChild(modal);
  });

  // Close modal when clicking outside the content
  modal.addEventListener('click', (e) => {
    if (e.target === modal) {
      document.body.removeChild(modal);
    }
  });

  // Add some basic modal styling if not already in CSS
  if (!document.querySelector('style#modal-styles')) {
    const style = document.createElement('style');
    style.id = 'modal-styles';
    style.textContent = `
      .modal {
        display: block;
        position: fixed;
        z-index: 1000;
        left: 0;
        top: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0,0,0,0.7);
        overflow: auto;
      }
      .modal-content {
        background-color: var(--card-background, #fff);
        margin: 10% auto;
        padding: 20px;
        border-radius: 8px;
        width: 80%;
        max-width: 800px;
        box-shadow: var(--card-shadow, 0 4px 8px rgba(0,0,0,0.1));
      }
      .close-modal {
        color: var(--text-color, #333);
        float: right;
        font-size: 28px;
        font-weight: bold;
        cursor: pointer;
      }
      .post-content {
        margin-top: 20px;
        line-height: 1.6;
      }
      .blog-posts {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 20px;
        margin-top: 20px;
      }
      .blog-card {
        background-color: var(--card-background, #fff);
        border-radius: 8px;
        box-shadow: var(--card-shadow, 0 4px 8px rgba(0,0,0,0.1));
        padding: 20px;
        transition: transform 0.3s ease;
      }
      .blog-card:hover {
        transform: translateY(-5px);
      }
      .blog-title {
        margin-top: 0;
        color: var(--primary-color, #2c3e50);
      }
      .post-meta {
        color: var(--light-text, #7f8c8d);
        font-size: 0.9rem;
        margin-bottom: 15px;
      }
      .tag {
        display: inline-block;
        background-color: var(--secondary-color, #3498db);
        color: white;
        padding: 3px 8px;
        border-radius: 4px;
        margin-right: 5px;
        font-size: 0.8rem;
      }
      .read-more {
        display: inline-block;
        margin-top: 10px;
        color: var(--primary-color, #2c3e50);
        font-weight: bold;
      }
    `;
    document.head.appendChild(style);
  }
}

// Initialize when the DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
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