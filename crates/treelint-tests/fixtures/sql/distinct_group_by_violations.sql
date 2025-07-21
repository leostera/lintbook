-- Test cases for SQL017: DISTINCT with GROUP BY conflict violations

-- Violations: DISTINCT with GROUP BY (potentially redundant)
SELECT DISTINCT department, COUNT(*) 
FROM employees 
GROUP BY department;

-- Violations: Another case of redundant DISTINCT
SELECT DISTINCT 
    category_id,
    category_name,
    COUNT(product_id) as product_count
FROM products p
JOIN categories c ON p.category_id = c.id
GROUP BY category_id, category_name;

-- Violations: DISTINCT in complex query with GROUP BY
SELECT DISTINCT u.department, AVG(u.salary)
FROM users u
JOIN salaries s ON u.id = s.user_id
GROUP BY u.department
HAVING AVG(u.salary) > 50000;

-- OK: DISTINCT without GROUP BY
SELECT DISTINCT department FROM employees;

-- OK: GROUP BY without DISTINCT
SELECT department, COUNT(*) 
FROM employees 
GROUP BY department;

-- OK: DISTINCT on different columns than GROUP BY (may be intentional)
SELECT department, COUNT(*) 
FROM employees 
GROUP BY department;