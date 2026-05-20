-- Test cases for SQL006: COALESCE over IFNULL/NVL violations

-- Violations: Using IFNULL (MySQL specific)
SELECT IFNULL(name, 'Unknown') as display_name FROM users;
SELECT id, IFNULL(email, 'no-email@example.com') FROM users;

-- Violations: Using NVL (Oracle specific)  
SELECT NVL(description, 'No description') as desc FROM products;
UPDATE users SET bio = NVL(bio, 'No bio available');

-- Violations: Using ISNULL (SQL Server specific)
SELECT ISNULL(phone, 'N/A') as contact_phone FROM users;
SELECT customer_id, ISNULL(notes, '') as customer_notes FROM orders;

-- Mixed violations
SELECT 
    IFNULL(first_name, '') + ' ' + NVL(last_name, '') as full_name,
    ISNULL(email, 'unknown@example.com') as email
FROM users;

-- OK: Using COALESCE (SQL standard)
SELECT COALESCE(name, 'Unknown') as display_name FROM users;
SELECT id, COALESCE(email, 'no-email@example.com') FROM users;
SELECT COALESCE(description, 'No description') as desc FROM products;
UPDATE users SET bio = COALESCE(bio, 'No bio available');
SELECT COALESCE(phone, 'N/A') as contact_phone FROM users;