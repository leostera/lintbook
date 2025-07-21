-- Test cases for SQL020: Semicolon placement violations

-- Violations: Missing semicolon on multi-line statement
SELECT name, email 
FROM users 
WHERE active = 1

-- Violations: Semicolon in wrong position
SELECT name, email; FROM users WHERE active = 1;

-- Violations: Multiple statements without proper semicolons
UPDATE users SET active = 1 WHERE id = 1
DELETE FROM posts WHERE user_id = 1

-- OK: Proper semicolon placement
SELECT name, email 
FROM users 
WHERE active = 1;

-- OK: Single line (semicolon optional)
SELECT COUNT(*) FROM users;

-- OK: Multiple statements with proper semicolons
UPDATE users SET active = 1 WHERE id = 1;
DELETE FROM posts WHERE user_id = 1;