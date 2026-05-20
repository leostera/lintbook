-- Test cases for SQL016: Consistent COUNT syntax violations

-- Mixed COUNT syntax (inconsistent)
SELECT 
    department,
    COUNT(*) as total_employees,
    COUNT(1) as active_count,
    COUNT(0) as another_count
FROM employees 
GROUP BY department;

-- More mixed usage
SELECT COUNT(*) FROM users UNION ALL SELECT COUNT(1) FROM orders;

-- Different query with different pattern
SELECT COUNT(1) as user_count FROM users WHERE active = 1;
SELECT COUNT(0) as post_count FROM posts WHERE published = 1;

-- OK: Consistent COUNT(*) usage
SELECT 
    department,
    COUNT(*) as total_employees,
    COUNT(*) as active_count
FROM employees 
GROUP BY department;

-- OK: Consistent COUNT(1) usage  
SELECT 
    status,
    COUNT(1) as total_orders,
    COUNT(1) as completed_orders
FROM orders
GROUP BY status;