-- Test cases for SQL001: Table aliasing style violations

-- Violation: Missing AS keyword
SELECT * FROM users u WHERE u.id = 1;

-- Violation: Multiple tables without AS
SELECT u.name, p.title 
FROM users u, posts p 
WHERE u.id = p.user_id;

-- OK: Proper AS usage
SELECT * FROM users AS u WHERE u.id = 1;

-- Violation: Complex query with missing AS
SELECT u.name, COUNT(p.id) as post_count
FROM users u
JOIN posts p ON u.id = p.user_id
GROUP BY u.name;

-- OK: All aliases use AS
SELECT u.name, COUNT(p.id) AS post_count
FROM users AS u
JOIN posts AS p ON u.id = p.user_id
GROUP BY u.name;