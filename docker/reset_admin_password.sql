-- Reset admin password to 'admin123'
-- This is a pre-computed bcrypt hash for 'admin123'
UPDATE users SET 
    password_hash = '$2b$12$szjUBOGbvVaRDM1yQRRXm.t9FeI9UG9HyTXLOqsB8QMvZ6x5XY5Uu',
    updated_at = datetime('now')
WHERE username = 'admin';

-- If admin user doesn't exist, create it
INSERT OR IGNORE INTO users 
    (username, display_name, email, password_hash, role, created_at, updated_at)
VALUES 
    ('admin', 'Administrator', 'admin@example.com', 
     '$2b$12$szjUBOGbvVaRDM1yQRRXm.t9FeI9UG9HyTXLOqsB8QMvZ6x5XY5Uu', 
     'Admin', datetime('now'), datetime('now'));

-- Verify the admin user exists
SELECT id, username, role FROM users WHERE username = 'admin';