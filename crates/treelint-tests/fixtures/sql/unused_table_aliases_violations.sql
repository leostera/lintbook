-- Test cases for SQL019: Unused table aliases violations

-- Violations: Alias defined but never used
SELECT name, email FROM users u WHERE name IS NOT NULL;

-- Violations: Multiple aliases, some unused
SELECT u.name, email, phone 
FROM users u, posts p 
WHERE u.active = 1;

-- Violations: JOIN with unused alias
SELECT users.name, posts.title 
FROM users 
JOIN posts p ON users.id = posts.user_id;

-- OK: All aliases used
SELECT u.name, p.title 
FROM users u 
JOIN posts p ON u.id = p.user_id;

-- OK: No aliases (valid)
SELECT name, email FROM users WHERE active = 1;

-- OK: Single table with used alias
SELECT u.name, u.email FROM users u WHERE u.active = 1;