-- SQL029: Quoted literals violations
-- String literals should use single quotes, not double quotes

-- VIOLATION: Double-quoted string literals
SELECT * FROM users WHERE name = "John Doe";
SELECT * FROM products WHERE category = "Electronics";
INSERT INTO logs (message) VALUES ("User logged in");

-- VIOLATION: Mixed quote styles
SELECT 
    CASE 
        WHEN status = 'active' THEN "Active User"
        WHEN status = 'inactive' THEN "Inactive User"
        ELSE 'Unknown'
    END as status_label
FROM users;

-- GOOD: Single quotes for string literals
SELECT * FROM users WHERE name = 'John Doe';
SELECT * FROM products WHERE category = 'Electronics';
INSERT INTO logs (message) VALUES ('User logged in');

-- GOOD: Double quotes for identifiers (when needed)
SELECT "user", "select", "from" 
FROM "table" 
WHERE "column" = 'value';