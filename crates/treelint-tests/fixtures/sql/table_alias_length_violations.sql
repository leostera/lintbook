-- SQL026: Table alias length violations
-- Single-letter aliases in complex queries should be avoided

-- VIOLATION: Single-letter aliases in joins
SELECT 
    u.id,
    u.name,
    o.order_date,
    p.product_name
FROM users u
JOIN orders o ON u.id = o.user_id
JOIN products p ON o.product_id = p.id;

-- VIOLATION: Multiple single-letter aliases
SELECT *
FROM customers c
INNER JOIN addresses a ON c.id = a.customer_id
LEFT JOIN phones p ON c.id = p.customer_id
WHERE c.active = true;

-- GOOD: Meaningful aliases
SELECT 
    usr.id,
    usr.name,
    ord.order_date,
    prod.product_name
FROM users usr
JOIN orders ord ON usr.id = ord.user_id
JOIN products prod ON ord.product_id = prod.id;

-- GOOD: Single table queries can use short aliases
SELECT u.id, u.name
FROM users u
WHERE u.active = true;