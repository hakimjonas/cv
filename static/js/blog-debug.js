// Blog Debug Script
// This script helps debug issues with the blog admin page
/**
 * Blog API debug utilities
 */

async function testApiConnection() {
    const results = { success: false };

    // Get the current host
    const currentHost = window.location.host;
    const baseUrl = window.location.origin;

    try {
        console.log(`Testing API connection to ${baseUrl}/api/blog`);
        const response = await fetch(`${baseUrl}/api/blog`);
// Blog API Debug utilities

/**
 * Test connection to the API server
 * @returns {Promise<Object>} Result object with success status and data
 */
async function testApiConnection() {
    try {
        // Try ports in sequence
        for (let port = 3000; port <= 3010; port++) {
            try {
                const url = `http://127.0.0.1:${port}/api/blog`;
                console.log(`Testing connection to ${url}`);

                const response = await fetch(url, {
                    method: 'GET',
                    headers: {
                        'Accept': 'application/json'
                    },
                    // Set a timeout using AbortController
                    signal: AbortSignal.timeout(5000) // 5 second timeout
                });

                if (response.ok) {
                    const posts = await response.json();
                    console.log(`Connected successfully to ${url}`);
                    return {
                        success: true,
                        source: url,
                        posts: posts || []
                    };
                }
            } catch (err) {
                // Just log and continue to the next port
                console.log(`Error connecting to port ${port}: ${err.message}`);
            }
        }

        // If we get here, no connections succeeded
        return {
            success: false,
            error: "Could not connect to any API endpoint"
        };
    } catch (error) {
        console.error('API connection test failed:', error);
        return {
            success: false,
            error: error.message
        };
    }
}
/**
 * Blog API debug utilities
 */

/**
 * Test connection to the API server
 * @returns {Promise<Object>} Result object with success status and data
 */
async function testApiConnection() {
    try {
        // Try the current domain first
        const currentOrigin = window.location.origin;

        // If we're on a server, try the API directly
        if (currentOrigin.includes('http')) {
            try {
                const url = `${currentOrigin}/api/blog`;
                console.log(`Testing connection to ${url}`);

                const response = await fetch(url, {
                    method: 'GET',
                    headers: {
                        'Accept': 'application/json'
                    },
                    // Set a timeout using AbortController
                    signal: AbortSignal.timeout(5000) // 5 second timeout
                });

                if (response.ok) {
                    const posts = await response.json();
                    console.log(`Connected successfully to ${url}`);
                    return {
                        success: true,
                        source: url,
                        posts: posts || []
                    };
                }
            } catch (err) {
                console.log(`Error connecting to current domain: ${err.message}`);
            }
        }

        // Try different ports on localhost
        for (let port = 3000; port <= 3010; port++) {
            try {
                const url = `http://127.0.0.1:${port}/api/blog`;
                console.log(`Testing connection to ${url}`);

                const response = await fetch(url, {
                    method: 'GET',
                    headers: {
                        'Accept': 'application/json'
                    },
                    // Set a timeout using AbortController
                    signal: AbortSignal.timeout(5000) // 5 second timeout
                });

                if (response.ok) {
                    const posts = await response.json();
                    console.log(`Connected successfully to ${url}`);
                    return {
                        success: true,
                        source: url,
                        posts: posts || []
                    };
                }
            } catch (err) {
                // Just log and continue to the next port
                console.log(`Error connecting to port ${port}: ${err.message}`);
            }
        }

        // If we get here, no connections succeeded
        return {
            success: false,
            error: "Could not connect to any API endpoint"
        };
    } catch (error) {
        console.error('API connection test failed:', error);
        return {
            success: false,
            error: error.message
        };
    }
}

/**
 * Safely get JSON from a response
 * @param {Response} response - Fetch API Response object
 * @returns {Promise<Object>} - Parsed JSON or error object
 */
async function safeGetJson(response) {
    try {
        return await response.json();
    } catch (error) {
        console.error('Error parsing JSON:', error);
        return { error: 'Invalid JSON response' };
    }
}

/**
 * Create a test post
 * @returns {Promise<Object>} Result object
 */
async function createTestPost() {
    try {
        const testPost = {
            title: "Test Post " + new Date().toISOString(),
            slug: "test-post-" + Date.now(),
            date: new Date().toISOString().split('T')[0],
            author: "Test Author",
            excerpt: "This is a test post created by the debug script",
            content: "# Test Post\n\nThis is a test post created by the debug script.",
            published: true,
            featured: false,
            image: null,
            tags: [],
            metadata: {}
        };

        // Get the API endpoint
        const connectionResult = await testApiConnection();
        if (!connectionResult.success) {
            return { success: false, error: "No API connection available" };
        }

        const apiUrl = connectionResult.source;
        console.log(`Creating test post at ${apiUrl}`);

        const response = await fetch(apiUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            },
            body: JSON.stringify(testPost),
            signal: AbortSignal.timeout(10000) // 10 second timeout
        });

        if (response.ok) {
            const result = await safeGetJson(response);
            console.log('Test post created successfully:', result);
            return { success: true, data: result };
        } else {
            const errorText = await response.text();
            console.error(`Failed to create test post: ${response.status} ${errorText}`);
            return { success: false, error: `API returned ${response.status}: ${errorText}` };
        }

    } catch (error) {
        console.error('Error creating test post:', error);
        return { success: false, error: error.message };
    }
}

// Add global error handlers to catch unhandled promise rejections
window.addEventListener('unhandledrejection', function(event) {
    console.error('Unhandled promise rejection:', event.reason);
    event.preventDefault(); // Prevent the default browser behavior
});

// Add global error handler
window.addEventListener('error', function(event) {
    console.error('Global error:', event.error);
});

// Initialize debug tools when the page loads
document.addEventListener('DOMContentLoaded', function() {
    console.log('Blog debug tools initialized');
});

// Make functions available globally
window.testApiConnection = testApiConnection;
window.createTestPost = createTestPost;
window.safeGetJson = safeGetJson;

console.log("Blog Debug Script loaded");
/**
 * Safely get JSON from a response
 * @param {Response} response - Fetch API Response object
 * @returns {Promise<Object>} - Parsed JSON or error object
 */
async function safeGetJson(response) {
    try {
        return await response.json();
    } catch (error) {
        console.error('Error parsing JSON:', error);
        return { error: 'Invalid JSON response' };
    }
}

// Add global error handler to catch unhandled promise rejections
window.addEventListener('unhandledrejection', function(event) {
    console.error('Unhandled promise rejection:', event.reason);
});

// Add global error handler
window.addEventListener('error', function(event) {
    console.error('Global error:', event.error);
});

// Initialize debug tools when the page loads
document.addEventListener('DOMContentLoaded', function() {
    console.log('Blog debug tools initialized');
});
        if (response.ok) {
            const posts = await response.json();
            results.success = true;
            results.source = baseUrl;
            results.posts = posts;
            console.log(`API connection successful! Found ${posts.length} posts.`);
        }
    } catch (error) {
        console.error('API connection error:', error);
    }

    return results;
}
console.log("Blog Debug Script loaded");

// Function to test API connection
async function testApiConnection() {
  console.log("Testing API connection...");
  
  try {
    // Try to fetch from API
    const apiUrl = window.location.protocol === 'file:' 
      ? '../api/blog' // When accessed via file://
      : '/api/blog';  // When accessed via http://
    
    console.log(`Trying to fetch from API: ${apiUrl}`);
    const response = await fetch(apiUrl);
    
    if (response.ok) {
      const posts = await response.json();
      console.log(`API connection successful! Found ${posts.length} posts:`, posts);
      return { success: true, source: 'api', posts };
    } else {
      console.log(`API returned status: ${response.status}`);
    }
  } catch (apiError) {
    console.log('API connection failed:', apiError);
  }
  
  // If API fails, try JSON file
  try {
    const jsonUrl = '../data/blog_posts.json';
    console.log(`Trying to fetch from JSON file: ${jsonUrl}`);
    const response = await fetch(jsonUrl);
    
    if (response.ok) {
      const posts = await response.json();
      console.log(`JSON file connection successful! Found ${posts.length} posts:`, posts);
      return { success: true, source: 'json', posts };
    } else {
      console.log(`JSON file returned status: ${response.status}`);
    }
  } catch (jsonError) {
    console.log('JSON file connection failed:', jsonError);
  }
  
  // If both fail, try localStorage
  try {
    const storedPosts = localStorage.getItem('blog_posts');
    if (storedPosts) {
      const posts = JSON.parse(storedPosts);
      console.log(`localStorage connection successful! Found ${posts.length} posts:`, posts);
      return { success: true, source: 'localStorage', posts };
    } else {
      console.log('No posts found in localStorage');
    }
  } catch (localStorageError) {
    console.log('localStorage connection failed:', localStorageError);
  }
  
  return { success: false };
}

// Function to test saving a post
async function testSavePost() {
  console.log("Testing save post functionality...");
  
  const testPost = {
    title: "Test Post " + new Date().toISOString(),
    slug: "test-post-" + Date.now(),
    date: new Date().toISOString().split('T')[0],
    category: "Test",
    excerpt: "This is a test post created by the debug script",
    content: "# Test Post\n\nThis is a test post created by the debug script.",
    featured: false,
    image: null
  };
  
  try {
    // Try to save to API
    const apiBaseUrl = window.location.protocol === 'file:' 
      ? '../api/blog' // When accessed via file://
      : '/api/blog';  // When accessed via http://
    
    console.log(`Trying to save to API: ${apiBaseUrl}`);
    const response = await fetch(apiBaseUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(testPost)
    });
    
    if (response.ok) {
      console.log('API save successful!');
      return { success: true, source: 'api' };
    } else {
      console.log(`API returned status: ${response.status}`);
    }
  } catch (apiError) {
    console.log('API save failed:', apiError);
  }
  
  // If API fails, try localStorage
  try {
    let posts = [];
    const storedPosts = localStorage.getItem('blog_posts');
    
    if (storedPosts) {
      posts = JSON.parse(storedPosts);
    } else {
      // Try to fetch from JSON file
      try {
        const jsonResponse = await fetch('../data/blog_posts.json');
        if (jsonResponse.ok) {
          posts = await jsonResponse.json();
        }
      } catch (jsonError) {
        console.log('JSON file fetch failed:', jsonError);
      }
    }
    
    // Add new post
    posts.push(testPost);
    
    // Save to localStorage
    localStorage.setItem('blog_posts', JSON.stringify(posts));
    console.log('localStorage save successful!');
    
    return { success: true, source: 'localStorage' };
  } catch (localStorageError) {
    console.log('localStorage save failed:', localStorageError);
  }
  
  return { success: false };
}

// Add debug UI
function addDebugUI() {
  const debugDiv = document.createElement('div');
  debugDiv.className = 'debug-panel';
  debugDiv.style.position = 'fixed';
  debugDiv.style.bottom = '10px';
  debugDiv.style.right = '10px';
  debugDiv.style.backgroundColor = 'rgba(0, 0, 0, 0.8)';
  debugDiv.style.color = 'white';
  debugDiv.style.padding = '10px';
  debugDiv.style.borderRadius = '5px';
  debugDiv.style.zIndex = '9999';
  debugDiv.style.maxWidth = '300px';
  
  const heading = document.createElement('h3');
  heading.textContent = 'Blog Debug Panel';
  heading.style.margin = '0 0 10px 0';
  debugDiv.appendChild(heading);
  
  const testApiButton = document.createElement('button');
  testApiButton.textContent = 'Test API Connection';
  testApiButton.style.display = 'block';
  testApiButton.style.margin = '5px 0';
  testApiButton.style.padding = '5px 10px';
  testApiButton.addEventListener('click', async () => {
    const result = await testApiConnection();
    alert(`API Test Result: ${result.success ? 'Success' : 'Failed'}\nSource: ${result.source || 'None'}\nPosts: ${result.posts ? result.posts.length : 0}`);
  });
  debugDiv.appendChild(testApiButton);
  
  const testSaveButton = document.createElement('button');
  testSaveButton.textContent = 'Test Save Post';
  testSaveButton.style.display = 'block';
  testSaveButton.style.margin = '5px 0';
  testSaveButton.style.padding = '5px 10px';
  testSaveButton.addEventListener('click', async () => {
    const result = await testSavePost();
    alert(`Save Test Result: ${result.success ? 'Success' : 'Failed'}\nSource: ${result.source || 'None'}`);
    // Reload posts after saving
    if (result.success && typeof loadBlogPosts === 'function') {
      loadBlogPosts();
    }
  });
  debugDiv.appendChild(testSaveButton);
  
  const clearStorageButton = document.createElement('button');
  clearStorageButton.textContent = 'Clear localStorage';
  clearStorageButton.style.display = 'block';
  clearStorageButton.style.margin = '5px 0';
  clearStorageButton.style.padding = '5px 10px';
  clearStorageButton.addEventListener('click', () => {
    localStorage.removeItem('blog_posts');
    alert('localStorage cleared');
    // Reload posts after clearing
    if (typeof loadBlogPosts === 'function') {
      loadBlogPosts();
    }
  });
  debugDiv.appendChild(clearStorageButton);
  
  document.body.appendChild(debugDiv);
}

// Initialize debug tools when the page is loaded
document.addEventListener('DOMContentLoaded', function() {
  console.log('Blog Debug: DOM loaded');
  
  // Add debug UI after a short delay to ensure the page is fully loaded
  setTimeout(addDebugUI, 1000);
  
  // Test API connection
  testApiConnection();
});

// Also initialize when the window is fully loaded (as a fallback)
window.addEventListener('load', function() {
  console.log('Blog Debug: Window loaded');
});