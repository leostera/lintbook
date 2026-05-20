-- Test cases for SQL005: Not-equal operator consistency violations

-- Mixed usage: Some != and some <> (should be consistent)
SELECT * FROM users WHERE status != 'inactive' AND role <> 'admin';

-- More mixed usage
SELECT u.name, p.title 
FROM users u 
JOIN posts p ON u.id = p.user_id 
WHERE u.active != 0 
  AND p.status <> 'draft' 
  AND u.email != '';

-- OK: Consistent != usage
SELECT * FROM users WHERE status != 'inactive' AND role != 'admin';
SELECT * FROM posts WHERE author_id != 1 AND status != 'published';

-- OK: Consistent <> usage  
SELECT * FROM users WHERE status <> 'inactive' AND role <> 'admin';
SELECT * FROM products WHERE price <> 0 AND category_id <> 999;