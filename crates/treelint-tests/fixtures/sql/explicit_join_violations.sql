-- Test cases for SQL009: Explicit JOIN types violations

-- Violations: Comma-separated tables (implicit joins)
SELECT u.name, p.title 
FROM users u, posts p 
WHERE u.id = p.user_id;

-- Violations: Multiple table implicit join
SELECT u.name, p.title, c.name as category
FROM users u, posts p, categories c
WHERE u.id = p.user_id 
  AND p.category_id = c.id;

-- Violations: Mixed implicit and explicit joins
SELECT u.name, p.title, t.name as tag
FROM users u, posts p
JOIN post_tags pt ON p.id = pt.post_id
JOIN tags t ON pt.tag_id = t.id
WHERE u.id = p.user_id;

-- OK: Explicit INNER JOIN
SELECT u.name, p.title 
FROM users u
INNER JOIN posts p ON u.id = p.user_id;

-- OK: Explicit LEFT JOIN
SELECT u.name, p.title
FROM users u  
LEFT JOIN posts p ON u.id = p.user_id;

-- OK: Multiple explicit joins
SELECT u.name, p.title, c.name as category
FROM users u
INNER JOIN posts p ON u.id = p.user_id
INNER JOIN categories c ON p.category_id = c.id;

-- OK: Function calls with commas (should not trigger)
SELECT CONCAT(first_name, ', ', last_name) as full_name
FROM users;