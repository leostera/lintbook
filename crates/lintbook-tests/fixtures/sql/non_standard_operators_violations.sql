-- SQL035: Comparison operators violations
-- Use standard comparison operators

-- VIOLATION: Non-standard operators
SELECT * FROM users WHERE age !< 18;  -- Should use >= 18
SELECT * FROM products WHERE price !> 100;  -- Should use <= 100
SELECT * FROM orders WHERE status ~= 'pending';  -- Should use <> or !=

-- VIOLATION: Database-specific operators
SELECT * FROM logs WHERE message ~< 'error';
SELECT * FROM data WHERE value @@ 'pattern';
SELECT * FROM items WHERE score <=> NULL;  -- Should use IS NOT DISTINCT FROM

-- VIOLATION: ISNULL function instead of IS NULL
SELECT * FROM users WHERE ISNULL(email);
SELECT name, ISNULL(phone) as has_no_phone FROM contacts;

-- VIOLATION: Complex non-standard operators
SELECT * FROM records WHERE data !!< 'value';
SELECT * FROM metrics WHERE score !~ pattern;

-- GOOD: Standard operators
SELECT * FROM users WHERE age >= 18;
SELECT * FROM products WHERE price <= 100;
SELECT * FROM orders WHERE status <> 'pending';
SELECT * FROM orders WHERE status != 'pending';

-- GOOD: Standard NULL checks
SELECT * FROM users WHERE email IS NULL;
SELECT * FROM users WHERE phone IS NOT NULL;

-- GOOD: Standard comparison operators
SELECT * FROM data WHERE value = 'test';
SELECT * FROM numbers WHERE score < 50;
SELECT * FROM ranges WHERE num BETWEEN 1 AND 100;
SELECT * FROM lists WHERE id IN (1, 2, 3);
SELECT * FROM patterns WHERE name LIKE '%test%';