/**
 * Blog Admin JavaScript
 */

document.addEventListener('DOMContentLoaded', function() {
  // Initialize any components that might need JavaScript
  initializeDropdowns();
  setupFormValidation();

  // Add click handler for mobile menu toggle
  const mobileMenuToggle = document.getElementById('mobile-menu-toggle');
  if (mobileMenuToggle) {
    mobileMenuToggle.addEventListener('click', function() {
      const adminNav = document.querySelector('.admin-nav ul');
      adminNav.classList.toggle('mobile-visible');
    });
  }
});

/**
 * Initialize dropdown menus
 */
function initializeDropdowns() {
  const dropdowns = document.querySelectorAll('.dropdown');

  dropdowns.forEach(dropdown => {
    const trigger = dropdown.querySelector('.dropdown-trigger');
    const menu = dropdown.querySelector('.dropdown-menu');

    if (trigger && menu) {
      trigger.addEventListener('click', function(e) {
        e.preventDefault();
        e.stopPropagation();
        menu.classList.toggle('visible');
      });

      // Close when clicking outside
      document.addEventListener('click', function() {
        menu.classList.remove('visible');
      });

      // Prevent menu from closing when clicking inside it
      menu.addEventListener('click', function(e) {
        e.stopPropagation();
      });
    }
  });
}

/**
 * Setup form validation for common forms
 */
function setupFormValidation() {
  const forms = document.querySelectorAll('form[data-validate="true"]');

  forms.forEach(form => {
    form.addEventListener('submit', function(e) {
      let isValid = true;

      // Check required fields
      const requiredFields = form.querySelectorAll('[required]');
      requiredFields.forEach(field => {
        if (!field.value.trim()) {
          isValid = false;
          highlightInvalidField(field);
        } else {
          removeInvalidHighlight(field);
        }
      });

      // Check email fields
      const emailFields = form.querySelectorAll('input[type="email"]');
      emailFields.forEach(field => {
        if (field.value && !isValidEmail(field.value)) {
          isValid = false;
          highlightInvalidField(field);
        }
      });

      if (!isValid) {
        e.preventDefault();
        showFormError(form, 'Please fill in all required fields correctly.');
      }
    });
  });
}

/**
 * Highlight invalid form field
 */
function highlightInvalidField(field) {
  field.classList.add('invalid');

  // Add error message if not already present
  const parent = field.parentElement;
  if (!parent.querySelector('.error-message')) {
    const errorMsg = document.createElement('div');
    errorMsg.className = 'error-message';
    errorMsg.textContent = field.dataset.errorMsg || 'This field is required';
    parent.appendChild(errorMsg);
  }

  // Remove invalid class when user fixes the input
  field.addEventListener('input', function() {
    removeInvalidHighlight(field);
  }, { once: true });
}

/**
 * Remove invalid highlight from form field
 */
function removeInvalidHighlight(field) {
  field.classList.remove('invalid');

  // Remove error message if present
  const parent = field.parentElement;
  const errorMsg = parent.querySelector('.error-message');
  if (errorMsg) {
    parent.removeChild(errorMsg);
  }
}

/**
 * Show form error message
 */
function showFormError(form, message) {
  // Remove any existing error message
  const existingError = form.querySelector('.form-error');
  if (existingError) {
    existingError.remove();
  }

  // Create and add new error message
  const errorDiv = document.createElement('div');
  errorDiv.className = 'form-error';
  errorDiv.textContent = message;

  form.insertBefore(errorDiv, form.firstChild);

  // Scroll to error message
  errorDiv.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

/**
 * Check if an email address is valid
 */
function isValidEmail(email) {
  const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return re.test(email);
}

/**
 * Format a date to a human-readable string
 */
function formatDate(dateString) {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
}
