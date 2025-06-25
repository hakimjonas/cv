/**
 * Modules for dynamically loading HTML components
 * This prevents reloading these components when switching between pages
 * and ensures a consistent UI across the site
 */

// Function to load HTML content from a URL
async function loadHTML(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            throw new Error(`Failed to load ${url}: ${response.status} ${response.statusText}`);
        }
        return await response.text();
    } catch (error) {
        console.error(`Error loading ${url}:`, error);
        // Provide a more user-friendly error handling
        document.dispatchEvent(new CustomEvent('module-load-error', { 
            detail: { url, error: error.message } 
        }));
        return null;
    }
}

// Function to load and cache HTML modules
const ModuleLoader = (function() {
    // Cache for loaded modules using Map for better performance and functionality
    const moduleCache = new Map();

    // Social links data (will be loaded from CV data if available)
    const socialLinks = [];

    // Load a module and cache it
    async function loadModule(moduleName) {
        // Check if module is already cached
        if (moduleCache.has(moduleName)) {
            return moduleCache.get(moduleName);
        }

        try {
            const html = await loadHTML(`modules/${moduleName}.html`);
            if (html) {
                moduleCache.set(moduleName, html);
            }
            return html;
        } catch (error) {
            console.error(`Failed to load module ${moduleName}:`, error);
            // Dispatch a more specific event for this particular error
            document.dispatchEvent(new CustomEvent('module-load-error', { 
                detail: { 
                    moduleName, 
                    error: error.message,
                    type: 'module' 
                } 
            }));
            return null;
        }
    }

    // Extract author name from page title
    function extractAuthorName() {
        const title = document.title;
        if (title) {
            // Title format is usually "Author Name - Page Title"
            const parts = title.split(' - ');
            if (parts.length > 0) {
                return parts[0];
            }
        }
        return 'Your Name'; // Default fallback
    }

    // Replace placeholders in HTML content
    function replacePlaceholders(html, data) {
        if (!html) return html;

        // Define placeholder mappings with defaults
        const placeholders = {
            '__AUTHOR_NAME__': data.authorName || 'Your Name',
            '__PAGE_TITLE__': data.pageTitle || document.title || 'Personal Website',
            '__PAGE_DESCRIPTION__': data.pageDescription || 'Personal website and portfolio showcasing projects, skills, and blog posts.',
            '__PAGE_KEYWORDS__': data.pageKeywords || 'portfolio, blog, developer, projects, skills, technology',
            '__PAGE_URL__': data.pageUrl || window.location.href,
            '__PAGE_IMAGE__': data.pageImage || `${window.location.origin}/img/profile_picture.jpg`
        };

        // Create a new string by replacing all placeholders
        let result = html;

        // Replace simple placeholders
        Object.entries(placeholders).forEach(([placeholder, value]) => {
            // Only process if the placeholder exists in the HTML
            if (result.includes(placeholder)) {
                result = result.replace(new RegExp(placeholder, 'g'), value);
            }
        });

        // Replace current page indicators for navigation
        if (result.includes('__CURRENT_PAGE_')) {
            // Get the current page from the URL
            const currentPath = window.location.pathname;
            const pageName = currentPath.split('/').pop().split('.')[0] || 'index';

            // Create a map of page names to their current state
            const pageCurrentState = {
                'index': pageName === 'index' ? 'page' : 'false',
                'cv': pageName === 'cv' ? 'page' : 'false',
                'projects': pageName === 'projects' ? 'page' : 'false'
            };

            // Replace all current page indicators
            Object.entries(pageCurrentState).forEach(([page, state]) => {
                result = result.replace(
                    new RegExp(`__CURRENT_PAGE_${page.toUpperCase()}__`, 'g'), 
                    state
                );
            });
        }

        // Replace social links
        if (result.includes('__SOCIAL_LINKS__')) {
            // Use social links from data or defaults
            const links = data.socialLinks || [
                { platform: 'github', url: 'https://github.com/username' },
                { platform: 'linkedin', url: 'https://linkedin.com/in/username' }
            ];

            // Generate social links HTML using template literals and map
            const socialLinksHtml = links.map(link => `
                <a href="${link.url}" class="social-link" target="_blank" rel="noopener" aria-label="${link.platform} profile">
                    <i class="nf nf-${link.platform.toLowerCase()}"></i>
                </a>`
            ).join('');

            result = result.replace('__SOCIAL_LINKS__', socialLinksHtml);
        }

        return result;
    }

    // Extract page description from meta tags or provide a default
    function extractPageDescription() {
        const metaDescription = document.querySelector('meta[name="description"]');
        return metaDescription ? metaDescription.getAttribute('content') : '';
    }

    // Extract page keywords from meta tags or provide a default
    function extractPageKeywords() {
        const metaKeywords = document.querySelector('meta[name="keywords"]');
        return metaKeywords ? metaKeywords.getAttribute('content') : '';
    }

    // Extract page image from meta tags or provide a default
    function extractPageImage() {
        const metaImage = document.querySelector('meta[property="og:image"]');
        return metaImage ? metaImage.getAttribute('content') : '';
    }

    // Load head module and inject into document head
    async function loadHeadModule() {
        const headHTML = await loadModule('head');
        if (headHTML) {
            // Extract metadata from the document
            const authorName = extractAuthorName();
            const pageTitle = document.title;
            const pageDescription = extractPageDescription();
            const pageKeywords = extractPageKeywords();
            const pageUrl = window.location.href;
            const pageImage = extractPageImage();

            // Replace placeholders with actual values
            const processedHTML = replacePlaceholders(headHTML, {
                authorName,
                pageTitle,
                pageDescription,
                pageKeywords,
                pageUrl,
                pageImage
            });

            // Create a temporary container
            const tempContainer = document.createElement('div');
            tempContainer.innerHTML = processedHTML;

            // Get all child nodes
            const nodes = tempContainer.childNodes;

            // Append each node to the head
            for (let i = 0; i < nodes.length; i++) {
                // Skip text nodes (whitespace)
                if (nodes[i].nodeType !== Node.TEXT_NODE) {
                    document.head.appendChild(nodes[i].cloneNode(true));
                }
            }
        }
    }

    // Function to check if CSS files are loaded
    function areCSSFilesLoaded() {
        // Get all stylesheet links
        const linkElements = document.querySelectorAll('head link[rel="stylesheet"]');

        if (linkElements.length === 0) {
            console.warn('No stylesheet links found in the document head');
            return true; // No stylesheets to load, so consider them "loaded"
        }

        // Convert to array for easier processing
        const links = Array.from(linkElements);

        // Critical stylesheets that must be loaded
        const criticalStylesheets = [
            'main.css',
            'nerd-fonts.css',
            'nerd-icons.css'
        ];

        // Check if critical stylesheets exist and are loaded
        const criticalLinks = links.filter(link => 
            criticalStylesheets.some(css => link.href.includes(css))
        );

        if (criticalLinks.length === 0) {
            console.warn('No critical stylesheets found');
            // Continue anyway, but log a warning
        }

        // Check if all stylesheets are loaded
        return links.every(link => {
            // First check if the link has loaded
            if (link.sheet === null) {
                return false;
            }

            try {
                // For Firefox and other browsers that implement cssRules
                // An empty stylesheet is still considered loaded
                return link.sheet !== null;
            } catch (e) {
                // Handle CORS issues
                if (e.name === 'SecurityError') {
                    console.warn(`Cannot access cssRules for ${link.href} due to CORS restrictions`);
                    // For CORS-restricted stylesheets, we'll assume they're loaded
                    // after a certain time has passed since the page started loading
                    const pageLoadTime = performance.now();
                    return pageLoadTime > 2000; // Assume loaded after 2 seconds
                }

                console.error(`Error checking if CSS is loaded: ${e.message}`);
                return false;
            }
        });
    }

    // Initialize modules on the page
    async function initModules() {
        // Get author name from page title
        const authorName = extractAuthorName();

        // Try to load social links from the page if available
        const socialLinksContainer = document.getElementById('social-links');
        if (socialLinksContainer) {
            const links = socialLinksContainer.querySelectorAll('a.social-link');
            links.forEach(link => {
                const icon = link.querySelector('i');
                if (icon) {
                    const classNames = icon.className.split(' ');
                    for (const className of classNames) {
                        if (className.startsWith('nf-')) {
                            const platform = className.replace('nf-', '');
                            socialLinks.push({
                                platform,
                                url: link.href
                            });
                            break;
                        }
                    }
                }
            });
        }

        // Data for placeholder replacement
        const placeholderData = {
            authorName,
            socialLinks
        };

        // Load head module first and ensure it's applied
        await loadHeadModule();

        // Wait for CSS files to be loaded with a timeout
        const maxWaitTime = 5000; // 5 seconds maximum wait time
        const startTime = Date.now();

        while (!areCSSFilesLoaded() && (Date.now() - startTime < maxWaitTime)) {
            // Wait 100ms before checking again
            await new Promise(resolve => setTimeout(resolve, 100));
        }

        // Force a browser reflow to ensure CSS is applied
        document.body.offsetHeight;

        // Add a delay to ensure rendering is complete
        await new Promise(resolve => setTimeout(resolve, 500));

        // Load header
        const headerPlaceholder = document.getElementById('header-placeholder');
        if (headerPlaceholder) {
            const headerHTML = await loadModule('header');
            if (headerHTML) {
                const processedHTML = replacePlaceholders(headerHTML, placeholderData);
                headerPlaceholder.outerHTML = processedHTML;

                // Initialize menu after header is loaded
                if (window.MenuModule) {
                    const currentPath = window.location.pathname;
                    const pageName = currentPath.split('/').pop().split('.')[0] || 'index';
                    window.MenuModule.init({ activeItem: pageName });
                }

                // Initialize theme switcher with improved accessibility and performance
                const themeToggle = document.getElementById('theme-toggle');
                if (themeToggle) {
                    // Create a ThemeManager to handle theme switching
                    const ThemeManager = {
                        // Theme constants
                        THEMES: {
                            DARK: 'theme-dark',
                            LIGHT: 'theme-light'
                        },

                        // Storage key
                        STORAGE_KEY: 'theme',

                        // Get the current theme
                        getCurrentTheme() {
                            return document.documentElement.className;
                        },

                        // Check if current theme is dark
                        isDarkTheme() {
                            return this.getCurrentTheme() === this.THEMES.DARK;
                        },

                        // Get user's preferred theme from localStorage or system preference
                        getPreferredTheme() {
                            const savedTheme = localStorage.getItem(this.STORAGE_KEY);
                            if (savedTheme) {
                                return savedTheme;
                            }

                            return window.matchMedia('(prefers-color-scheme: dark)').matches 
                                ? this.THEMES.DARK 
                                : this.THEMES.LIGHT;
                        },

                        // Set the theme and update UI
                        setTheme(theme) {
                            // Validate theme
                            if (theme !== this.THEMES.DARK && theme !== this.THEMES.LIGHT) {
                                console.error(`Invalid theme: ${theme}`);
                                theme = this.THEMES.LIGHT; // Default to light theme
                            }

                            // Clear any existing theme classes first
                            document.documentElement.classList.remove(this.THEMES.DARK, this.THEMES.LIGHT);

                            // Apply theme to document
                            document.documentElement.classList.add(theme);

                            // Ensure the theme class is applied (for backward compatibility)
                            // Instead of replacing all classes, we'll ensure only theme classes are managed
                            const currentClasses = document.documentElement.className.split(' ');
                            const filteredClasses = currentClasses.filter(cls => 
                                cls !== this.THEMES.DARK && 
                                cls !== this.THEMES.LIGHT
                            );

                            // Add the new theme class to the filtered classes
                            filteredClasses.push(theme);

                            // Apply the updated class list
                            document.documentElement.className = filteredClasses.join(' ');

                            // Force a browser reflow to ensure CSS variables are applied
                            void document.documentElement.offsetHeight;

                            // Save preference
                            localStorage.setItem(this.STORAGE_KEY, theme);

                            // Update button state for accessibility
                            const isDark = theme === this.THEMES.DARK;
                            themeToggle.setAttribute('aria-pressed', isDark.toString());

                            // Update class for styling
                            themeToggle.classList.toggle('active', isDark);

                            // Log theme change for debugging
                            console.log(`Theme set to: ${theme}`);

                            // Dispatch event for other components to react
                            document.dispatchEvent(new CustomEvent('themechange', { 
                                detail: { theme, isDark } 
                            }));

                            return theme;
                        },

                        // Toggle between light and dark themes
                        toggleTheme() {
                            console.log('toggleTheme called');
                            console.log('Current theme:', this.getCurrentTheme());
                            console.log('Is dark theme:', this.isDarkTheme());

                            const newTheme = this.isDarkTheme() 
                                ? this.THEMES.LIGHT 
                                : this.THEMES.DARK;

                            console.log('New theme:', newTheme);

                            // Force a reflow to ensure CSS variables are applied
                            document.documentElement.offsetHeight;

                            return this.setTheme(newTheme);
                        },

                        // Initialize theme based on preferences
                        initialize() {
                            // Check if there's already a theme class on the document
                            const currentClass = document.documentElement.className;
                            const hasDarkTheme = currentClass.includes(this.THEMES.DARK);
                            const hasLightTheme = currentClass.includes(this.THEMES.LIGHT);
                            const hasThemeClass = hasDarkTheme || hasLightTheme;

                            // If a theme is already applied (by the script in head.html), respect it
                            if (hasThemeClass) {
                                console.log('Found existing theme class:', currentClass);

                                // Determine which theme is currently applied
                                const currentTheme = hasDarkTheme ? this.THEMES.DARK : this.THEMES.LIGHT;

                                // Update the button state to match the current theme
                                const isDark = currentTheme === this.THEMES.DARK;
                                if (themeToggle) {
                                    themeToggle.setAttribute('aria-pressed', isDark.toString());
                                    themeToggle.classList.toggle('active', isDark);
                                }

                                // Store the current theme in localStorage to ensure persistence
                                localStorage.setItem(this.STORAGE_KEY, currentTheme);

                                console.log('Respecting existing theme:', currentTheme);
                                return this;
                            }

                            // If no theme is applied yet, apply the preferred theme
                            const preferredTheme = this.getPreferredTheme();
                            console.log('No existing theme found. Initializing with:', preferredTheme);
                            this.setTheme(preferredTheme);

                            // Listen for system preference changes
                            const darkModeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

                            // Use the appropriate event listener method based on browser support
                            const mediaQueryListener = e => {
                                // Only update if user hasn't set a preference
                                if (!localStorage.getItem(this.STORAGE_KEY)) {
                                    this.setTheme(
                                        e.matches ? this.THEMES.DARK : this.THEMES.LIGHT
                                    );
                                }
                            };

                            // Use addEventListener if available, otherwise use the deprecated addListener
                            if (darkModeMediaQuery.addEventListener) {
                                darkModeMediaQuery.addEventListener('change', mediaQueryListener);
                            } else if (darkModeMediaQuery.addListener) {
                                // Fallback for older browsers
                                darkModeMediaQuery.addListener(mediaQueryListener);
                            }

                            return this;
                        }
                    };

                    // Initialize the theme manager
                    const themeManager = Object.create(ThemeManager).initialize();

                    // Add click event listener to toggle theme
                    themeToggle.addEventListener('click', () => {
                        console.log('Theme toggle clicked');
                        themeManager.toggleTheme();
                    });

                    // Log the initial state of the theme toggle button
                    console.log('Theme toggle button:', themeToggle);
                    console.log('Theme toggle button aria-pressed:', themeToggle.getAttribute('aria-pressed'));
                    console.log('Theme toggle button class:', themeToggle.className);

                    // Add keyboard support for accessibility
                    themeToggle.addEventListener('keydown', (e) => {
                        // Toggle on Enter or Space
                        if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            themeManager.toggleTheme();
                        }
                    });
                }
            }
        }

        // Load footer
        const footerPlaceholder = document.getElementById('footer-placeholder');
        if (footerPlaceholder) {
            const footerHTML = await loadModule('footer');
            if (footerHTML) {
                const processedHTML = replacePlaceholders(footerHTML, placeholderData);
                footerPlaceholder.outerHTML = processedHTML;

                // Update current year in footer
                const currentYearElement = document.getElementById('current-year');
                if (currentYearElement) {
                    currentYearElement.textContent = new Date().getFullYear();
                }

                // Initialize accordion in footer
                const accordionHeader = document.querySelector('.accordion-header');
                if (accordionHeader) {
                    accordionHeader.addEventListener('click', function() {
                        this.classList.toggle('active');
                        const content = this.nextElementSibling;
                        content.classList.toggle('active');

                        if (content.classList.contains('active')) {
                            content.style.maxHeight = content.scrollHeight + 'px';
                        } else {
                            content.style.maxHeight = '0';
                        }
                    });
                }
            }
        }
    }

    // Load a project card and populate it with data
    async function loadProjectCard(projectData, container) {
        // Validate inputs
        if (!container) {
            console.error('No container provided for project card');
            return;
        }

        if (!projectData) {
            console.error('No project data provided for project card');
            return;
        }

        try {
            // Load the project card template
            const cardHTML = await loadModule('project-card');
            if (!cardHTML) {
                throw new Error('Failed to load project card template');
            }

            // Create a mapping of placeholders to their values
            const placeholderMap = {
                '__PROJECT_NAME__': projectData.display_name || projectData.name || 'Unnamed Project',
                '__PROJECT_DESCRIPTION__': projectData.description || 'No description available',
                '__LANGUAGE_CLASS__': projectData.language ? ` ${projectData.language}` : '',
                '__LANGUAGE_ICON__': projectData.language_icon ? 
                    `<span class="${projectData.language}-icon">${projectData.language_icon}</span>` : '',
                '__STARS__': projectData.stars ? 
                    `<div class="stars"><i class="nf nf-star"></i> ${projectData.stars}</div>` : '',
                '__OWNER_AVATAR__': projectData.owner_avatar ? 
                    `<img src="${projectData.owner_avatar}" alt="Owner avatar" class="avatar" loading="lazy">` : 
                    '<div class="avatar-placeholder" aria-hidden="true"></div>',
                '__OWNER_USERNAME__': projectData.owner_username ? 
                    `<span class="owner-name">${projectData.owner_username}</span>` : ''
            };

            // Generate highlights HTML if available
            if (projectData.highlights && projectData.highlights.length > 0) {
                placeholderMap['__PROJECT_HIGHLIGHTS__'] = `
                <div class="project-highlights">
                    <h4>Highlights:</h4>
                    <ul>
                        ${projectData.highlights.map(highlight => 
                            `<li>${escapeHTML(highlight)}</li>`).join('')}
                    </ul>
                </div>`;
            } else {
                placeholderMap['__PROJECT_HIGHLIGHTS__'] = '';
            }

            // Generate technologies HTML if available
            if (projectData.technologies && projectData.technologies.length > 0) {
                placeholderMap['__PROJECT_TECHNOLOGIES__'] = projectData.technologies
                    .map(tech => `<span class="tech-tag">${escapeHTML(tech)}</span>`)
                    .join('');
            } else {
                placeholderMap['__PROJECT_TECHNOLOGIES__'] = '';
            }

            // Generate repository link if available
            if (projectData.repository) {
                placeholderMap['__PROJECT_REPOSITORY__'] = `
                <a href="${projectData.repository}" 
                   class="project-link" 
                   target="_blank" 
                   rel="noopener"
                   aria-label="View GitHub repository for ${escapeHTML(projectData.display_name || projectData.name)}">
                    <i class="nf nf-github" aria-hidden="true"></i> GitHub
                </a>`;
            } else {
                placeholderMap['__PROJECT_REPOSITORY__'] = '';
            }

            // Generate project URL if available
            if (projectData.url) {
                placeholderMap['__PROJECT_URL__'] = `
                <a href="${projectData.url}" 
                   class="project-link" 
                   target="_blank" 
                   rel="noopener"
                   aria-label="View live demo of ${escapeHTML(projectData.display_name || projectData.name)}">
                    <i class="nf nf-external-link" aria-hidden="true"></i> Live Demo
                </a>`;
            } else {
                placeholderMap['__PROJECT_URL__'] = '';
            }

            // Replace all placeholders in the template
            let processedHTML = cardHTML;
            Object.entries(placeholderMap).forEach(([placeholder, value]) => {
                processedHTML = processedHTML.replace(placeholder, value);
            });

            // Create a temporary container and add the processed HTML
            const tempContainer = document.createElement('div');
            tempContainer.innerHTML = processedHTML;

            // Get the first child (the card) and add it to the container
            const card = tempContainer.firstChild;

            // Add proper ARIA attributes for accessibility
            card.setAttribute('role', 'article');
            card.setAttribute('aria-label', `Project: ${projectData.display_name || projectData.name}`);

            // Append the card to the container
            container.appendChild(card);

            // Return the created card element
            return card;
        } catch (error) {
            console.error('Error loading project card:', error);
            // Create a fallback card for graceful degradation
            const fallbackCard = document.createElement('div');
            fallbackCard.className = 'github-card error';
            fallbackCard.innerHTML = `
                <div class="card-header">
                    <h3 class="project-title">${projectData.display_name || projectData.name || 'Project'}</h3>
                </div>
                <p class="repo-description">
                    ${projectData.description || 'No description available'}
                    <br><small>(Error loading full project card)</small>
                </p>
            `;
            container.appendChild(fallbackCard);
            return fallbackCard;
        }
    }

    // Helper function to escape HTML to prevent XSS
    function escapeHTML(str) {
        if (!str) return '';
        return str
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#039;');
    }

    // Public API
    return {
        init: initModules,
        loadProjectCard: loadProjectCard,
        loadModule: loadModule,
        replacePlaceholders: replacePlaceholders
    };
})();

// Initialize modules when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    ModuleLoader.init();
});
