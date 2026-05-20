-- SQL037: Join condition order violations
-- Join conditions should reference tables in consistent order

-- VIOLATION: Wrong table order in join condition
SELECT *
FROM orders o
JOIN customers c ON c.id = o.customer_id;  -- Should be o.customer_id = c.id

-- VIOLATION: Multiple joins with wrong order
SELECT *
FROM products p
JOIN categories cat ON cat.id = p.category_id  -- Should be p.category_id = cat.id
JOIN suppliers s ON s.id = p.supplier_id;  -- Should be p.supplier_id = s.id

-- VIOLATION: Complex joins with wrong order
SELECT 
    o.order_id,
    c.name,
    p.product_name
FROM orders o
JOIN customers c ON c.customer_id = o.customer_id  -- Wrong order
JOIN order_items oi ON oi.order_id = o.order_id
JOIN products p ON p.product_id = oi.product_id;  -- Wrong order

-- GOOD: Correct table order in join conditions
SELECT *
FROM orders o
JOIN customers c ON o.customer_id = c.id;

-- GOOD: Multiple joins with correct order  
SELECT *
FROM products p
JOIN categories cat ON p.category_id = cat.id
JOIN suppliers s ON p.supplier_id = s.id;

-- GOOD: Complex joins with correct order
SELECT 
    o.order_id,
    c.name,
    p.product_name
FROM orders o
JOIN customers c ON o.customer_id = c.customer_id
JOIN order_items oi ON o.order_id = oi.order_id
JOIN products p ON oi.product_id = p.product_id;