/**
 * Simple Blog API Client
 * A shared library for interacting with the blog API using Git-based authentication
 */

const BlogAPI = (function() {
    // Determine base URL from current page
    const baseUrl = window.location.origin;
    
    // Authentication token storage key
    const TOKEN_STORAGE_KEY = 'blog_auth_token';
    const USER_STORAGE_KEY = 'blog_auth_user';
    
    // Get stored authentication token
    function getAuthToken() {
        return localStorage.getItem(TOKEN_STORAGE_KEY);
    }
    
    // Store authentication token
    function setAuthToken(token) {
        if (token) {
            localStorage.setItem(TOKEN_STORAGE_KEY, token);
        } else {
            localStorage.removeItem(TOKEN_STORAGE_KEY);
        }
    }
    
    // Get stored user info
    function getUser() {
        const userJson = localStorage.getItem(USER_STORAGE_KEY);
        return userJson ? JSON.parse(userJson) : null;
    }
    
    // Store user info
    function setUser(user) {
        if (user) {
            localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(user));
        } else {
            localStorage.removeItem(USER_STORAGE_KEY);
        }
    }
    
    // Check if user is authenticated
    function isAuthenticated() {
        return !!getAuthToken();
    }

    /**
     * Make an API request
     * @param {string} endpoint - API endpoint path
     * @param {string} method - HTTP method
     * @param {object} data - Optional data to send
     * @returns {Promise} - Response promise
     */
    async function apiRequest(endpoint, method = 'GET', data = null, retryCount = 0) {
        const url = `${baseUrl}${endpoint}`;
        console.log(`Making ${method} request to ${url}`);

        const maxRetries = 3;
        const retryDelay = 1000; // 1 second initial delay

        const options = {
            method,
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            }
        };
        
        // Add authentication token if available
        const token = getAuthToken();
        if (token) {
            options.headers['Authorization'] = `Bearer ${token}`;
        }

        if (data && (method === 'POST' || method === 'PUT')) {
            // Ensure all required fields are present for posts
            if (endpoint.includes('/api/posts') && 
                (method === 'POST' || method === 'PUT')) {
                if (!data.title || !data.slug || !data.content) {
                    throw new Error('Missing required fields: title, slug, content must be non-empty');
                }

                // Ensure tags is always an array
                if (!data.tags) {
                    data.tags = [];
                }
            }

            console.log('Sending data:', data);
            options.body = JSON.stringify(data);
        }

        try {
            const response = await fetch(url, options);

            // Log response details for debugging
            console.log(`Response status: ${response.status}`);
            console.log('Response headers:', Object.fromEntries([...response.headers]));

            // Get response as text first
            const responseText = await response.text();
            console.log('Raw response:', responseText);

            // Try to parse as JSON if possible
            let responseData;
            try {
                responseData = JSON.parse(responseText);
            } catch (e) {
                console.warn('Response is not valid JSON');
                responseData = responseText;
            }

            // For 2xx responses, return the data
            if (response.status >= 200 && response.status < 300) {
                return responseData;
            }

            // For database locking errors (500 with specific messages), retry
            if (response.status === 500 && 
                (typeof responseData === 'string' && 
                 (responseData.includes('database is locked') || 
                  responseData.includes('database is busy') || 
                  responseData.includes('Failed to create post')))) {

                if (retryCount < maxRetries) {
                    console.log(`Database lock detected, retrying (${retryCount + 1}/${maxRetries})...`);
                    // Exponential backoff with jitter
                    const delay = retryDelay * Math.pow(2, retryCount) + Math.random() * 1000;
                    await new Promise(resolve => setTimeout(resolve, delay));
                    return apiRequest(endpoint, method, data, retryCount + 1);
                }
            }

            // For error responses, create an error with the response data
            const error = new Error(
                typeof responseData === 'string' 
                    ? responseData 
                    : JSON.stringify(responseData)
            );
            error.status = response.status;
            error.response = responseData;
            throw error;
        } catch (error) {
            // For network errors, retry
            if (error.name === 'TypeError' && error.message.includes('fetch') && retryCount < maxRetries) {
                console.log(`Network error, retrying (${retryCount + 1}/${maxRetries})...`);
                const delay = retryDelay * Math.pow(2, retryCount) + Math.random() * 1000;
                await new Promise(resolve => setTimeout(resolve, delay));
                return apiRequest(endpoint, method, data, retryCount + 1);
            }

            console.error('API request failed:', error);
            throw error;
        }
    }

    // Automatically create a session when the API client is initialized
    async function createSession() {
        try {
            if (!isAuthenticated()) {
                console.log('Creating session based on Git identity...');
                const response = await apiRequest('/api/session', 'POST');
                
                // Store the token and user info
                setAuthToken(response.token);
                setUser({
                    id: response.user.id,
                    username: response.user.username,
                    displayName: response.user.display_name,
                    role: response.user.role
                });
                
                console.log('Session created successfully:', response);
                return response;
            } else {
                console.log('Already authenticated');
                return { user: getUser() };
            }
        } catch (error) {
            console.error('Failed to create session:', error);
            throw error;
        }
    }

    return {
        // Authentication methods
        createSession: createSession,
        
        logout: () => {
            // Clear the stored token and user info
            setAuthToken(null);
            setUser(null);
        },
        
        isAuthenticated: isAuthenticated,
        
        getCurrentUser: getUser,
        
        // Blog post methods
        getPosts: () => apiRequest('/api/posts'),
        getPost: (slug) => apiRequest(`/api/posts/${slug}`),
        createPost: (postData) => apiRequest('/api/posts', 'POST', postData),
        updatePost: (slug, postData) => apiRequest(`/api/posts/${slug}`, 'PUT', postData),
        deletePost: (slug) => apiRequest(`/api/posts/${slug}`, 'DELETE'),

        // Special collections
        getPublishedPosts: () => apiRequest('/api/published'),
        getFeaturedPosts: () => apiRequest('/api/featured'),
        getTags: () => apiRequest('/api/tags'),
        getPostsByTag: (tagSlug) => apiRequest(`/api/tags/${tagSlug}`),

        // Utility method to get current API URL
        getBaseUrl: () => baseUrl
    };
})();

// Automatically create a session when the page loads
document.addEventListener('DOMContentLoaded', async () => {
    try {
        await BlogAPI.createSession();
        // If there's an updateAuthStatus function defined, call it
        if (typeof updateAuthStatus === 'function') {
            updateAuthStatus();
        }
    } catch (error) {
        console.error('Failed to create session on page load:', error);
    }
});