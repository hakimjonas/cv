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

// Load language icons from JSON file
let languageIcons = {};

// Fetch language icons from JSON file
fetch('data/language_icons.json')
    .then(response => response.json())
    .then(data => {
        languageIcons = data;
        console.log('Language icons loaded:', Object.keys(languageIcons).length);

        // Initialize language icons for GitHub cards after icons are loaded
        initializeLanguageIcons();
    })
    .catch(error => {
        console.error('Error loading language icons:', error);
        // Fallback to hardcoded icons
        languageIcons = {
            "scala": "",
            "rust": ""
        };
        initializeLanguageIcons();
    });

// Function to get language icon
function getLanguageIcon(language) {
    const normalizedLang = language.toLowerCase();

    // Direct match
    if (languageIcons[normalizedLang]) {
        return languageIcons[normalizedLang];
    }

    // Check for partial matches (e.g., "scala3" should match "scala")
    for (const [key, value] of Object.entries(languageIcons)) {
        if (normalizedLang.includes(key)) {
            return value;
        }
    }

    // Default icon for unknown languages
    return '';
}

// Update the current year in the footer
document.addEventListener('DOMContentLoaded', function() {
    // Initialize the menu module
    const currentPath = window.location.pathname;
    const pageName = currentPath.split('/').pop().split('.')[0] || 'index';

    // Configure menu based on current page
    let menuConfig = { activeItem: pageName };

    // Example of page-specific menu configuration
    if (pageName === 'cv') {
        menuConfig.visibleItems = ['index', 'cv', 'projects'];
    } else if (pageName === 'projects') {
        menuConfig.visibleItems = ['index', 'cv', 'projects', 'blog'];
    }

    // Initialize the menu with the configuration
    MenuModule.init(menuConfig);

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
});

// Function to initialize language icons for GitHub cards
function initializeLanguageIcons() {
    const githubCards = document.querySelectorAll('.github-card');
    githubCards.forEach(card => {
        const header = card.querySelector('.card-header');
        const title = card.querySelector('.project-title').textContent.toLowerCase();
        let languageDetected = false;

        // Function to add language icon
        function addLanguageIcon(language) {
            const titleElement = card.querySelector('.project-title');
            const iconClass = `${language}-icon`;

            if (titleElement && !titleElement.querySelector('i') && !titleElement.querySelector(`span.${iconClass}`)) {
                const icon = document.createElement('span');
                icon.textContent = getLanguageIcon(language);
                icon.className = iconClass;
                titleElement.prepend(icon);
                titleElement.insertBefore(document.createTextNode(' '), titleElement.childNodes[1]);
            }

            header.classList.add(language);
            languageDetected = true;
        }

        // Check title for language hints
        for (const lang in languageIcons) {
            if (title.includes(lang)) {
                addLanguageIcon(lang);
                break;
            }
        }

        // If no language detected from title, check technologies
        if (!languageDetected) {
            const techTags = card.querySelectorAll('.tech-tag');
            techTags.forEach(tag => {
                const tagText = tag.textContent.toLowerCase();
                for (const lang in languageIcons) {
                    if (tagText === lang || tagText.includes(lang)) {
                        addLanguageIcon(lang);
                        return;
                    }
                }
            });
        }
    });
}

// Theme switcher functionality
document.addEventListener('DOMContentLoaded', function() {
    const toggleSwitch = document.querySelector('.theme-switch input[type="checkbox"]');
    if (!toggleSwitch) return;

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

    // Function to get system preference
    function getSystemPreference() {
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'theme-dark' : 'theme-light';
    }

    // Check for saved theme preference or use system preference
    const savedTheme = localStorage.getItem('theme');
    const systemPreference = getSystemPreference();

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

    // Listen for system preference changes
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function(e) {
        // Only update if user hasn't set a preference
        if (!localStorage.getItem('theme')) {
            setTheme(e.matches ? 'theme-dark' : 'theme-light');
        }
    });
});

// Service Worker Registration
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/service-worker.js')
            .then(registration => {
                console.log('Service Worker registered with scope:', registration.scope);
            })
            .catch(error => {
                console.error('Service Worker registration failed:', error);
            });
    });
}
