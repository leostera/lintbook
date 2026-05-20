-- Test cases for SQL015: Unique table aliases violations

-- Violations: Duplicate table aliases
SELECT u.name, u.email 
FROM users u, user_profiles u 
WHERE u.id = u.user_id;

-- Violations: Multiple JOINs with same alias
SELECT u.name, p.title
FROM users u
JOIN posts p ON u.id = p.user_id
JOIN user_preferences u ON u.id = u.user_id;

-- Violations: Complex query with duplicate aliases
SELECT u.name, o.total, o.date
FROM users u
JOIN orders o ON u.id = o.user_id
JOIN order_items o ON o.id = o.order_id
WHERE u.active = 1;

-- OK: All unique aliases
SELECT u.name, p.title, c.name as category
FROM users u
JOIN posts p ON u.id = p.user_id
JOIN categories c ON p.category_id = c.id;

-- OK: No aliases (valid)
SELECT users.name, posts.title
FROM users, posts 
WHERE users.id = posts.user_id;

-- OK: Single table with alias
SELECT u.name, u.email FROM users u WHERE u.active = 1;