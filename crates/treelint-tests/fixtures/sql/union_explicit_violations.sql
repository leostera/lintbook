-- Test cases for SQL008: Explicit UNION ALL/DISTINCT violations

-- Violations: Plain UNION (implicit DISTINCT)
SELECT name FROM users
UNION
SELECT title FROM posts;

-- Violations: Multiple unions
SELECT id, name FROM customers
UNION  
SELECT id, company_name FROM suppliers
UNION
SELECT id, product_name FROM products;

-- Violations: Mixed with proper usage
SELECT email FROM users
UNION ALL
SELECT contact_email FROM customers
UNION  
SELECT support_email FROM vendors;

-- OK: Explicit UNION ALL
SELECT name FROM users
UNION ALL
SELECT title FROM posts;

-- OK: Explicit UNION DISTINCT  
SELECT category FROM products
UNION DISTINCT
SELECT department FROM employees;

-- OK: Multiple explicit unions
SELECT id, name FROM customers
UNION ALL
SELECT id, company_name FROM suppliers
UNION DISTINCT
SELECT id, product_name FROM products;