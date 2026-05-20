-- SQL028: Column names in GROUP BY/ORDER BY violations
-- Avoid positional references (1, 2, 3) in GROUP BY and ORDER BY

-- VIOLATION: Positional references in GROUP BY
SELECT 
    department,
    COUNT(*) as employee_count,
    AVG(salary) as avg_salary
FROM employees
GROUP BY 1;

-- VIOLATION: Positional references in ORDER BY
SELECT 
    name,
    age,
    salary
FROM employees
ORDER BY 2 DESC, 3;

-- VIOLATION: Mixed positional references
SELECT 
    department,
    job_title,
    COUNT(*) as count
FROM employees
GROUP BY 1, 2
ORDER BY 3 DESC;

-- GOOD: Using column names
SELECT 
    department,
    COUNT(*) as employee_count,
    AVG(salary) as avg_salary
FROM employees
GROUP BY department;

-- GOOD: Using aliases in ORDER BY
SELECT 
    name,
    age,
    salary
FROM employees
ORDER BY age DESC, salary;