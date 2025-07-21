-- SQL034: Boolean value expressions violations
-- Avoid redundant boolean comparisons

-- VIOLATION: Comparing to TRUE
SELECT * FROM users WHERE is_active = TRUE;
SELECT * FROM users WHERE is_admin = true;
SELECT * FROM orders WHERE is_completed=TRUE;

-- VIOLATION: Comparing to FALSE  
SELECT * FROM users WHERE is_deleted = FALSE;
SELECT * FROM products WHERE is_hidden = false;
SELECT * FROM accounts WHERE is_suspended=FALSE;

-- VIOLATION: Using != with boolean values
SELECT * FROM users WHERE is_verified != TRUE;
SELECT * FROM users WHERE is_premium != FALSE;

-- VIOLATION: Using <> with boolean values
SELECT * FROM posts WHERE is_published <> TRUE;
SELECT * FROM comments WHERE is_spam <> FALSE;

-- VIOLATION: Using IS TRUE/IS FALSE
SELECT * FROM users WHERE is_active IS TRUE;
SELECT * FROM users WHERE is_deleted IS FALSE;
SELECT * FROM users WHERE is_admin IS NOT TRUE;
SELECT * FROM users WHERE is_verified IS NOT FALSE;

-- GOOD: Direct boolean usage
SELECT * FROM users WHERE is_active;
SELECT * FROM users WHERE NOT is_deleted;
SELECT * FROM products WHERE is_visible AND NOT is_discontinued;

-- GOOD: Checking for NULL with IS NULL
SELECT * FROM users WHERE is_active IS NULL;
SELECT * FROM users WHERE is_active IS NOT NULL;