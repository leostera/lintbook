-- SQL031: Leading whitespace violations
-- SQL032: Trailing whitespace violations

-- VIOLATION: Tab indentation
	SELECT * FROM users;

-- VIOLATION: Inconsistent indentation
SELECT 
  id,
   name,  -- 3 spaces instead of 2
    email  -- 4 spaces
FROM users;

-- VIOLATION: Trailing spaces (note: spaces after semicolon)
SELECT * FROM users;   
SELECT name FROM products;  

-- VIOLATION: Mixed tabs and spaces
SELECT 
	id,     -- tab
  name    -- spaces
FROM users;

-- VIOLATION: Odd number of spaces
SELECT 
   id,      -- 3 spaces
     name   -- 5 spaces  
FROM users;

-- GOOD: Consistent indentation
SELECT 
  id,
  name,
  email
FROM users
WHERE active = true;

-- GOOD: No trailing whitespace
SELECT * FROM users;
SELECT name FROM products;