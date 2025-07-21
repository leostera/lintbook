-- Test cases for SQL014: Complex expressions need aliases violations

-- Violations: Mathematical operations without aliases
SELECT 
    price * quantity,
    (price * quantity) * 1.08,
    ROUND(price * 0.9, 2)
FROM order_items;

-- Violations: Function calls without aliases
SELECT 
    CONCAT(first_name, ' ', last_name),
    COALESCE(phone, email, 'No contact'),
    UPPER(TRIM(description))
FROM users;

-- Violations: CASE statements without aliases
SELECT 
    name,
    CASE 
        WHEN age >= 65 THEN 'Senior'
        WHEN age >= 18 THEN 'Adult'
        ELSE 'Minor'
    END,
    CASE status WHEN 'A' THEN 'Active' ELSE 'Inactive' END
FROM users;

-- Violations: Subqueries without aliases
SELECT 
    name,
    (SELECT COUNT(*) FROM orders WHERE user_id = users.id)
FROM users;

-- OK: Simple column references (no alias needed)
SELECT name, email, phone FROM users;
SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id;

-- OK: Complex expressions with proper aliases
SELECT 
    price * quantity AS line_total,
    (price * quantity) * 1.08 AS total_with_tax,
    ROUND(price * 0.9, 2) AS discounted_price,
    CONCAT(first_name, ' ', last_name) AS full_name,
    CASE 
        WHEN age >= 65 THEN 'Senior'
        WHEN age >= 18 THEN 'Adult'
        ELSE 'Minor'
    END AS age_category
FROM order_items;