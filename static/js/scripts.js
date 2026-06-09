/**
 * Main JavaScript file for the CV/Portfolio site
 */

// Menu Module - Persistent menu with configurable items
const MenuModule = (function() {
    // Store active menu item in localStorage
    const storeActiveMenuItem = (item) => {
        localStorage.setItem('activeMenuItem', item);
    };

    // Get active menu item from localStorage
    const getActiveMenuItem = () => {
        return localStorage.getItem('activeMenuItem') || 'home';
    };

    // Initialize menu with active item
    const initMenu = (config = {}) => {
        const menuItems = document.querySelectorAll('nav ul li a');
        const activeItem = config.activeItem || getActiveMenuItem();

        // Set active class on the current page's menu item
        menuItems.forEach(item => {
            const href = item.getAttribute('href');
            const itemId = href.includes('.html') ? href.split('.')[0] : href.replace('#', '');

            if (itemId === activeItem) {
                item.classList.add('active');
                storeActiveMenuItem(activeItem);
            }

            // Add click handler to store the clicked item
            item.addEventListener('click', function() {
                const clickedItemId = href.includes('.html') ? href.split('.')[0] : href.replace('#', '');
                storeActiveMenuItem(clickedItemId);
            });
        });

        // Apply custom menu configuration if provided
        if (config.visibleItems) {
            menuItems.forEach(item => {
                const href = item.getAttribute('href');
                const itemId = href.includes('.html') ? href.split('.')[0] : href.replace('#', '');

                if (!config.visibleItems.includes(itemId)) {
                    item.parentElement.style.display = 'none';
                } else {
                    item.parentElement.style.display = '';
                }
            });
        }
    };

    // Public API
    return {
        init: initMenu
    };
})();

// Theme Module - Light/Dark mode switcher with localStorage persistence
const ThemeModule = (function() {
    // Function to set a theme
    function setTheme(theme) {
        document.documentElement.className = theme;

        // Safe localStorage access with error handling
        try {
            localStorage.setItem("theme", theme);
        } catch (error) {
            console.warn('localStorage not available:', error);
        }

        const themeToggle = document.querySelector('.theme-switch');
        if (themeToggle) {
            if (theme === "theme-dark") {
                themeToggle.setAttribute('aria-pressed', 'true');
                themeToggle.classList.add('active');
                // Update tooltip to indicate what will happen on next click
                const lightIcon = themeToggle.querySelector('.light-icon');
                if (lightIcon) {
                    lightIcon.setAttribute('title', 'Switch to light theme');
                }
            } else {
                themeToggle.setAttribute('aria-pressed', 'false');
                themeToggle.classList.remove('active');
                // Update tooltip to indicate what will happen on next click
                const darkIcon = themeToggle.querySelector('.dark-icon');
                if (darkIcon) {
                    darkIcon.setAttribute('title', 'Switch to dark theme');
                }
            }
        }
    }

    // Initialize theme system
    function initTheme() {
        const themeToggle = document.querySelector('.theme-switch');
        if (!themeToggle) {
            return;
        }

        // Check for saved theme preference or use system preference
        let savedTheme = null;
        try {
            savedTheme = localStorage.getItem("theme");
        } catch (error) {
            console.warn('localStorage not available:', error);
        }

        const currentTheme = savedTheme ||
            (window.matchMedia("(prefers-color-scheme: dark)").matches ? "theme-dark" : "theme-light");

        // Apply the theme
        setTheme(currentTheme);

        // Listen for theme toggle
        themeToggle.addEventListener("click", function () {
            if (this.getAttribute('aria-pressed') === 'false') {
                setTheme("theme-dark");
            } else {
                setTheme("theme-light");
            }
        });

        // Add keyboard support for better accessibility
        themeToggle.addEventListener("keydown", function(e) {
            // Toggle on Space or Enter key
            if (e.key === ' ' || e.key === 'Enter') {
                e.preventDefault();
                this.click();
            }
        });
    }

    // Public API
    return {
        init: initTheme,
        setTheme: setTheme
    };
})();

// Note: project-card language classes and icon glyphs are rendered
// server-side from the authoritative GitHub primary language. No
// client-side language detection runs here — re-detecting from titles
// and tech tags appended spurious classes (e.g. substring hits like
// "c" inside "calculus-of-constructions") that fought the correct
// server class and shifted card-header colors.

// Main initialization function
document.addEventListener('DOMContentLoaded', function() {
    // Initialize the menu module
    const currentPath = window.location.pathname;
    const pageName = currentPath.split('/').pop().split('.')[0] || 'index';

    // Configure menu based on current page
    let menuConfig = { activeItem: pageName };

    // Example of page-specific menu configuration
    if (pageName === 'cv') {
        menuConfig.visibleItems = ['cv', 'projects', 'blog', 'index'];
    } else if (pageName === 'projects') {
        menuConfig.visibleItems = ['cv', 'projects', 'blog', 'index'];
    }

    // Initialize the menu with the configuration
    MenuModule.init(menuConfig);

    // Initialize the theme switcher
    ThemeModule.init();

    // Update the current year in the footer
    const currentYearElement = document.getElementById('current-year');
    if (currentYearElement) {
        currentYearElement.textContent = new Date().getFullYear();
    }

    // Format blog dates
    document.querySelectorAll('.blog-date[data-date]').forEach(dateElement => {
        const dateStr = dateElement.getAttribute('data-date');
        const date = new Date(dateStr);
        const options = { year: 'numeric', month: 'long', day: 'numeric' };
        dateElement.textContent = date.toLocaleDateString('en-US', options);
    });

    // GitHub repository links are now correctly formatted in the data source
    // No JavaScript fix needed

    // Initialize accordion functionality
    const accordionHeader = document.querySelector('.accordion-header');
    if (accordionHeader) {
        // Toggle accordion on click
        accordionHeader.addEventListener('click', function() {
            this.classList.toggle('active');
            const content = this.nextElementSibling;
            content.classList.toggle('active');

            if (content.classList.contains('active')) {
                // Open the accordion
                content.style.maxHeight = content.scrollHeight + 'px';
            } else {
                // Close the accordion
                content.style.maxHeight = '0';
            }
        });
    }

    // Mobile menu toggle
    const mobileMenuToggle = document.querySelector('.mobile-menu-toggle');
    const navigation = document.querySelector('#main-navigation');

    if (mobileMenuToggle && navigation) {
        mobileMenuToggle.addEventListener('click', function() {
            navigation.classList.toggle('active');
            mobileMenuToggle.setAttribute('aria-expanded',
                navigation.classList.contains('active') ? 'true' : 'false'
            );
        });
    }
});

// Service Worker Registration
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/service-worker.js')
            .then(registration => {
                // Service Worker registered successfully
            })
            .catch(error => {
                console.error('Service Worker registration failed:', error);
            });
    });
}
