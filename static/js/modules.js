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
        return null;
    }
}

// Function to load and cache HTML modules
const ModuleLoader = (function() {
    // Cache for loaded modules
    const moduleCache = {};

    // Social links data (will be loaded from CV data if available)
    let socialLinks = [];

    // Load a module and cache it
    async function loadModule(moduleName) {
        if (moduleCache[moduleName]) {
            return moduleCache[moduleName];
        }

        const html = await loadHTML(`modules/${moduleName}.html`);
        if (html) {
            moduleCache[moduleName] = html;
        }
        return html;
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

        // Replace author name
        html = html.replace(/__AUTHOR_NAME__/g, data.authorName || 'Your Name');

        // Replace social links
        if (html.includes('__SOCIAL_LINKS__')) {
            let socialLinksHtml = '';

            // Use social links from data or defaults
            const links = data.socialLinks || [
                { platform: 'github', url: 'https://github.com/username' },
                { platform: 'linkedin', url: 'https://linkedin.com/in/username' }
            ];

            links.forEach(link => {
                socialLinksHtml += `
                <a href="${link.url}" class="social-link" target="_blank" rel="noopener" aria-label="${link.platform} profile">
                    <i class="nf nf-${link.platform.toLowerCase()}"></i>
                </a>`;
            });

            html = html.replace('__SOCIAL_LINKS__', socialLinksHtml);
        }

        return html;
    }

    // Load head module and inject into document head
    async function loadHeadModule() {
        const headHTML = await loadModule('head');
        if (headHTML) {
            const authorName = extractAuthorName();
            const processedHTML = replacePlaceholders(headHTML, { authorName });

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

        // Ensure CSS files are loaded before rendering header and footer
        await new Promise(resolve => setTimeout(resolve, 100));

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

                // Initialize theme switcher
                const toggleSwitch = document.querySelector('.theme-switch input[type="checkbox"]');
                if (toggleSwitch) {
                    // Function to set a theme
                    function setTheme(theme) {
                        document.documentElement.className = theme;
                        localStorage.setItem('theme', theme);

                        if (theme === 'theme-dark') {
                            toggleSwitch.checked = true;
                        } else {
                            toggleSwitch.checked = false;
                        }
                    }

                    // Check for saved theme preference or use system preference
                    const savedTheme = localStorage.getItem('theme');
                    const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'theme-dark' : 'theme-light';

                    // Apply the theme (prioritize user's saved preference)
                    if (savedTheme) {
                        setTheme(savedTheme);
                    } else {
                        setTheme(systemPreference);
                    }

                    // Listen for theme toggle
                    toggleSwitch.addEventListener('change', function(e) {
                        if (e.target.checked) {
                            setTheme('theme-dark');
                        } else {
                            setTheme('theme-light');
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
        if (!container) return;

        const cardHTML = await loadModule('project-card');
        if (!cardHTML) return;

        let processedHTML = cardHTML;

        // Replace project name
        const projectName = projectData.display_name || projectData.name;
        processedHTML = processedHTML.replace('__PROJECT_NAME__', projectName);

        // Replace project description
        processedHTML = processedHTML.replace('__PROJECT_DESCRIPTION__', projectData.description || '');

        // Replace language class
        const languageClass = projectData.language ? ` ${projectData.language}` : '';
        processedHTML = processedHTML.replace('__LANGUAGE_CLASS__', languageClass);

        // Replace language icon
        const languageIcon = projectData.language_icon ? 
            `<span class="${projectData.language}-icon">${projectData.language_icon}</span>` : '';
        processedHTML = processedHTML.replace('__LANGUAGE_ICON__', languageIcon);

        // Replace stars
        const stars = projectData.stars ? 
            `<div class="stars"><i class="nf nf-star"></i> ${projectData.stars}</div>` : '';
        processedHTML = processedHTML.replace('__STARS__', stars);

        // Replace project highlights
        let highlightsHTML = '';
        if (projectData.highlights && projectData.highlights.length > 0) {
            highlightsHTML = `
            <div class="project-highlights">
                <h4>Highlights:</h4>
                <ul>
                    ${projectData.highlights.map(highlight => `<li>${highlight}</li>`).join('')}
                </ul>
            </div>`;
        }
        processedHTML = processedHTML.replace('__PROJECT_HIGHLIGHTS__', highlightsHTML);

        // Replace technologies
        let technologiesHTML = '';
        if (projectData.technologies && projectData.technologies.length > 0) {
            technologiesHTML = projectData.technologies.map(tech => 
                `<span class="tech-tag">${tech}</span>`).join('');
        }
        processedHTML = processedHTML.replace('__PROJECT_TECHNOLOGIES__', technologiesHTML);

        // Replace owner avatar
        let avatarHTML = '';
        if (projectData.owner_avatar) {
            avatarHTML = `<img src="${projectData.owner_avatar}" alt="Owner avatar" class="avatar">`;
        } else {
            avatarHTML = '<div class="avatar-placeholder"></div>';
        }
        processedHTML = processedHTML.replace('__OWNER_AVATAR__', avatarHTML);

        // Replace owner username
        const usernameHTML = projectData.owner_username ? 
            `<span class="owner-name">${projectData.owner_username}</span>` : '';
        processedHTML = processedHTML.replace('__OWNER_USERNAME__', usernameHTML);

        // Replace repository link
        const repoHTML = projectData.repository ? 
            `<a href="${projectData.repository}" class="project-link" target="_blank" rel="noopener">
                <i class="nf nf-github"></i> GitHub
            </a>` : '';
        processedHTML = processedHTML.replace('__PROJECT_REPOSITORY__', repoHTML);

        // Replace project URL
        const urlHTML = projectData.url ? 
            `<a href="${projectData.url}" class="project-link" target="_blank" rel="noopener">
                <i class="nf nf-external-link"></i> Live Demo
            </a>` : '';
        processedHTML = processedHTML.replace('__PROJECT_URL__', urlHTML);

        // Create a temporary container
        const tempContainer = document.createElement('div');
        tempContainer.innerHTML = processedHTML;

        // Append the card to the container
        container.appendChild(tempContainer.firstChild);
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
