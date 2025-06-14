// Blog Admin Fix Script
// This script ensures the admin page correctly connects to the blog post data

console.log("Blog Admin Fix Script loaded");

// Wait for the page to load
document.addEventListener('DOMContentLoaded', function() {
  console.log("DOM loaded, initializing blog admin fix");

  // Override the loadBlogPosts function to ensure it correctly loads blog posts
  window.originalLoadBlogPosts = window.loadBlogPosts;

  window.loadBlogPosts = async function() {
    console.log("Fixed loadBlogPosts function called");

    try {
      // Try to load directly from the JSON file first
      console.log("Trying to load directly from JSON file");
      const jsonUrl = '../data/blog_posts.json';

      try {
        const response = await fetch(jsonUrl, {
          cache: 'no-cache',
          headers: {
            'Accept': 'application/json'
          }
        });

        if (response.ok) {
          const posts = await response.json();
          console.log(`JSON file loaded successfully! Found ${posts.length} posts:`, posts);

          // Store in localStorage as a backup
          localStorage.setItem('blog_posts', JSON.stringify(posts));

          // Display the posts
          window.displayBlogPosts(posts);
          return;
        } else {
          console.log(`JSON file returned status: ${response.status}`);
        }
      } catch (jsonError) {
        console.log('Error loading from JSON file:', jsonError);
      }

      // If JSON file fails, try API
      console.log("Trying to load from API");
      const apiUrl = window.location.protocol === 'file:' 
        ? '../api/blog' // When accessed via file://
        : '/api/blog';  // When accessed via http://

      try {
        const response = await fetch(apiUrl, {
          cache: 'no-cache',
          headers: {
            'Accept': 'application/json'
          }
        });

        if (response.ok) {
          const posts = await response.json();
          console.log(`API loaded successfully! Found ${posts.length} posts:`, posts);

          // Store in localStorage as a backup
          localStorage.setItem('blog_posts', JSON.stringify(posts));

          // Display the posts
          window.displayBlogPosts(posts);
          return;
        } else {
          console.log(`API returned status: ${response.status}`);
        }
      } catch (apiError) {
        console.log('Error loading from API:', apiError);
      }

      // If both fail, try localStorage
      console.log("Trying to load from localStorage");
      const storedPosts = localStorage.getItem('blog_posts');
      if (storedPosts) {
        try {
          const posts = JSON.parse(storedPosts);
          console.log(`localStorage loaded successfully! Found ${posts.length} posts:`, posts);

          // Display the posts
          window.displayBlogPosts(posts);
          return;
        } catch (localStorageError) {
          console.log('Error parsing localStorage data:', localStorageError);
        }
      } else {
        console.log('No posts found in localStorage');
      }

      // If all else fails, show an error message
      console.log("All loading methods failed");
      document.getElementById('blog-items').innerHTML = '<p>Error loading blog posts. Please try again later.</p>';

    } catch (error) {
      console.error('Error in fixed loadBlogPosts function:', error);

      // Fall back to the original function
      console.log("Falling back to original loadBlogPosts function");
      return window.originalLoadBlogPosts();
    }
  };

  // Override the form submission to ensure it correctly saves blog posts
  const blogForm = document.getElementById('blog-form');
  if (blogForm) {
    const originalSubmitHandler = blogForm.onsubmit;

    blogForm.onsubmit = async function(e) {
      e.preventDefault();
      console.log("Fixed form submission handler called");

      // Get form data
      const postIdInput = document.getElementById('post-id');
      const titleInput = document.getElementById('title');
      const slugInput = document.getElementById('slug');
      const dateInput = document.getElementById('date');
      const categoryInput = document.getElementById('category');
      const excerptInput = document.getElementById('excerpt');
      const contentInput = document.getElementById('content');
      const imageInput = document.getElementById('image');
      const featuredInput = document.getElementById('featured');

      const postData = {
        title: titleInput.value,
        slug: slugInput.value,
        date: dateInput.value,
        category: categoryInput.value,
        excerpt: excerptInput.value,
        content: contentInput.value,
        image: imageInput.value || null,
        featured: featuredInput.checked
      };

      console.log("Post data:", postData);

      try {
        // First try to save to the server API
        try {
          // Use relative path for API endpoint
          const apiBaseUrl = window.location.protocol === 'file:' 
            ? '../api/blog' // When accessed via file://
            : '/api/blog';  // When accessed via http://

          console.log(`Using API endpoint: ${apiBaseUrl}`);

          let response;

          if (postIdInput.value) {
            // Update existing post using PUT request
            const slug = slugInput.value;
            console.log(`Updating post with slug: ${slug}`);

            response = await fetch(`${apiBaseUrl}/${slug}`, {
              method: 'PUT',
              headers: {
                'Content-Type': 'application/json'
              },
              body: JSON.stringify({
                title: postData.title,
                date: postData.date,
                category: postData.category,
                excerpt: postData.excerpt,
                content: postData.content,
                featured: postData.featured,
                image: postData.image
              })
            });

            console.log(`Update response status: ${response.status}`);
            if (!response.ok) {
              throw new Error(`HTTP error! status: ${response.status}`);
            }

            console.log(`Post "${postData.title}" updated successfully on server!`);
            alert(`Post "${postData.title}" updated successfully!`);
          } else {
            // Add new post using POST request
            console.log(`Creating new post: ${postData.title}`);

            response = await fetch(apiBaseUrl, {
              method: 'POST',
              headers: {
                'Content-Type': 'application/json'
              },
              body: JSON.stringify(postData)
            });

            console.log(`Create response status: ${response.status}`);
            if (!response.ok) {
              throw new Error(`HTTP error! status: ${response.status}`);
            }

            console.log(`Post "${postData.title}" created successfully on server!`);
            alert(`Post "${postData.title}" created successfully!`);
          }

          // Reset the form
          if (typeof resetForm === 'function') {
            resetForm();
          } else {
            blogForm.reset();
          }

          // Reload posts
          window.loadBlogPosts();

          return false;
        } catch (apiError) {
          console.error('API not available, falling back to localStorage:', apiError);

          // If API fails, fall back to localStorage
          let posts = [];

          // Try to load existing posts
          try {
            const jsonResponse = await fetch('../data/blog_posts.json', {
              cache: 'no-cache',
              headers: {
                'Accept': 'application/json'
              }
            });

            if (jsonResponse.ok) {
              posts = await jsonResponse.json();
              console.log(`Loaded ${posts.length} existing posts from JSON file`);
            }
          } catch (jsonError) {
            console.log('Error loading from JSON file:', jsonError);

            // Try to load from localStorage
            const storedPosts = localStorage.getItem('blog_posts');
            if (storedPosts) {
              posts = JSON.parse(storedPosts);
              console.log(`Loaded ${posts.length} existing posts from localStorage`);
            }
          }

          if (postIdInput.value) {
            // Update existing post
            const index = parseInt(postIdInput.value);
            console.log(`Updating post at index ${index}`);

            if (index >= 0 && index < posts.length) {
              posts[index] = postData;
              console.log(`Post updated: ${postData.title}`);
            } else {
              console.log(`Invalid index: ${index}, adding as new post`);
              posts.push(postData);
            }
          } else {
            // Add new post
            console.log(`Adding new post: ${postData.title}`);
            posts.push(postData);
          }

          // Save to localStorage
          localStorage.setItem('blog_posts', JSON.stringify(posts));
          console.log(`Saved ${posts.length} posts to localStorage`);

          // Reset the form
          if (typeof resetForm === 'function') {
            resetForm();
          } else {
            blogForm.reset();
          }

          // Reload posts
          window.loadBlogPosts();

          alert(`Post "${postData.title}" saved successfully to localStorage. Note: Changes will not be saved to the server until you restart the application.`);
        }
      } catch (error) {
        console.error('Error in fixed form submission handler:', error);

        // Fall back to the original handler
        console.log("Falling back to original form submission handler");
        return originalSubmitHandler ? originalSubmitHandler(e) : true;
      }

      return false;
    };
  }

  // Call loadBlogPosts to load posts immediately
  console.log("Calling loadBlogPosts from fix script");
  setTimeout(() => {
    if (typeof window.loadBlogPosts === 'function') {
      window.loadBlogPosts();
    }
  }, 500);
});
