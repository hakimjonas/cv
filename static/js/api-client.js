/**
 * Blog API Client
 * A shared library for interacting with the blog API
 */

const BlogAPI = (function() {
    // Determine base URL from current page
    const baseUrl = window.location.origin;

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

        if (data && (method === 'POST' || method === 'PUT')) {
            // Ensure all required fields are present for posts
            if (endpoint.includes('/api/blog') && 
                (method === 'POST' || method === 'PUT')) {
                if (!data.title || !data.slug || !data.content || !data.date || !data.author) {
                    throw new Error('Missing required fields: title, slug, content, date, author must be non-empty');
                }

                // Ensure tags is always an array
                if (!data.tags) {
                    data.tags = [];
                }

                // Ensure metadata is always an object
                if (!data.metadata) {
                    data.metadata = {};
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

    return {
        // Blog post methods
        getPosts: () => apiRequest('/api/blog'),
        getPost: (slug) => apiRequest(`/api/blog/${slug}`),
        createPost: (postData) => apiRequest('/api/blog', 'POST', postData),
        updatePost: (slug, postData) => apiRequest(`/api/blog/${slug}`, 'PUT', postData),
        deletePost: (slug) => apiRequest(`/api/blog/${slug}`, 'DELETE'),

        // Special collections
        getPublishedPosts: () => apiRequest('/api/blog/published'),
        getFeaturedPosts: () => apiRequest('/api/blog/featured'),
        getTags: () => apiRequest('/api/blog/tags'),
        getPostsByTag: (tagSlug) => apiRequest(`/api/blog/tag/${tagSlug}`),

        // Utility method to get current API URL
        getBaseUrl: () => baseUrl
    };
})();

// If we're in a CommonJS or ES module environment
if (typeof module !== 'undefined' && module.exports) {
    module.exports = BlogAPI;
} else if (typeof define === 'function' && define.amd) {
    define([], function() { return BlogAPI; });
}
