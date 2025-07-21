-- Test cases for SQL011: References must exist in FROM clause violations

-- Violations: Table reference not in FROM clause
SELECT u.name, p.title FROM users WHERE u.id = 1;  -- p is not declared

-- Violations: Typo in table alias
SELECT user.name, posts.title 
FROM users AS u, posts AS p 
WHERE user.id = p.user_id;  -- should be u.name

-- Violations: Multiple missing references
SELECT u.name, p.title, c.name 
FROM users AS u 
WHERE u.id = p.user_id AND p.category_id = c.id;  -- p and c not declared

-- OK: All references properly declared
SELECT u.name, p.title 
FROM users AS u, posts AS p 
WHERE u.id = p.user_id;

-- OK: JOIN syntax with proper references
SELECT u.name, p.title, c.name
FROM users AS u
INNER JOIN posts AS p ON u.id = p.user_id
INNER JOIN categories AS c ON p.category_id = c.id;

-- OK: No table prefixes (valid)
SELECT name, title FROM users, posts WHERE users.id = posts.user_id;