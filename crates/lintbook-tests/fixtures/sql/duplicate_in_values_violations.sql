-- SQL030: Distinct values in IN clause violations
-- IN clauses should not contain duplicate values

-- VIOLATION: Duplicate numeric values
SELECT * FROM users WHERE id IN (1, 2, 3, 2, 4);
SELECT * FROM orders WHERE status_id IN (10, 20, 10, 30);

-- VIOLATION: Duplicate string values
SELECT * FROM products WHERE category IN ('Electronics', 'Books', 'Electronics', 'Toys');
SELECT * FROM users WHERE country IN ('USA', 'UK', 'Canada', 'USA');

-- VIOLATION: Complex duplicates
SELECT * FROM transactions 
WHERE type IN ('CREDIT', 'DEBIT', 'CREDIT', 'REFUND')
  AND amount IN (100, 200, 100, 300);

-- GOOD: Unique values only
SELECT * FROM users WHERE id IN (1, 2, 3, 4);
SELECT * FROM products WHERE category IN ('Electronics', 'Books', 'Toys');
SELECT * FROM users WHERE country IN ('USA', 'UK', 'Canada');