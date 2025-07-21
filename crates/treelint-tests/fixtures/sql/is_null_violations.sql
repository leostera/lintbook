-- Test cases for SQL007: IS NULL vs = NULL violations

-- Violations: Using = NULL
SELECT * FROM users WHERE name = NULL;
SELECT * FROM posts WHERE content = NULL AND author_id = 1;

-- Violations: Using != NULL
SELECT * FROM users WHERE email != NULL;
UPDATE users SET active = 1 WHERE deleted_at != NULL;

-- Violations: Using <> NULL
SELECT * FROM products WHERE description <> NULL;
DELETE FROM orders WHERE shipped_at <> NULL;

-- OK: Proper IS NULL usage
SELECT * FROM users WHERE name IS NULL;
SELECT * FROM posts WHERE content IS NULL AND author_id = 1;

-- OK: Proper IS NOT NULL usage
SELECT * FROM users WHERE email IS NOT NULL;
UPDATE users SET active = 1 WHERE deleted_at IS NOT NULL;
SELECT * FROM products WHERE description IS NOT NULL;
DELETE FROM orders WHERE shipped_at IS NOT NULL;