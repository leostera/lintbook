-- Test cases for SQL002: Column aliasing style violations

-- Violation: Missing AS keyword in column alias
SELECT name username, email user_email FROM users;

-- Violation: Multiple columns without AS
SELECT 
    name username,
    email user_email,
    created_at signup_date
FROM users;

-- OK: Proper AS usage
SELECT name AS username, email AS user_email FROM users;

-- Violation: Mixed usage (some with AS, some without)
SELECT 
    name AS username,
    email user_email,
    COUNT(id) user_count
FROM users;

-- OK: All aliases use AS
SELECT 
    name AS username,
    email AS user_email,
    COUNT(id) AS user_count
FROM users;