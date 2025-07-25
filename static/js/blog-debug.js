/**
 * Blog Debug Script
 * This script provides utilities for debugging the blog API
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