-- SQL036: Line length violations
-- Lines should not exceed 120 characters

-- VIOLATION: Very long single-line SELECT
SELECT customer_id, first_name, last_name, email_address, phone_number, street_address, city, state, postal_code, country, registration_date FROM customers;

-- VIOLATION: Long WHERE clause on single line
SELECT * FROM orders WHERE order_date >= '2024-01-01' AND order_date <= '2024-12-31' AND status IN ('pending', 'processing', 'shipped') AND total_amount > 1000;

-- VIOLATION: Long JOIN condition on single line  
SELECT o.order_id, o.order_date, c.customer_name, p.product_name, oi.quantity, oi.unit_price FROM orders o JOIN customers c ON o.customer_id = c.customer_id JOIN order_items oi ON o.order_id = oi.order_id JOIN products p ON oi.product_id = p.product_id;

-- GOOD: Properly formatted multi-line SELECT
SELECT 
    customer_id,
    first_name,
    last_name,
    email_address,
    phone_number,
    street_address,
    city,
    state,
    postal_code,
    country,
    registration_date
FROM customers;

-- GOOD: Multi-line WHERE clause
SELECT * 
FROM orders 
WHERE order_date >= '2024-01-01' 
  AND order_date <= '2024-12-31' 
  AND status IN ('pending', 'processing', 'shipped') 
  AND total_amount > 1000;

-- GOOD: Multi-line JOIN
SELECT 
    o.order_id,
    o.order_date,
    c.customer_name,
    p.product_name,
    oi.quantity,
    oi.unit_price
FROM orders o
JOIN customers c ON o.customer_id = c.customer_id
JOIN order_items oi ON o.order_id = oi.order_id
JOIN products p ON oi.product_id = p.product_id;