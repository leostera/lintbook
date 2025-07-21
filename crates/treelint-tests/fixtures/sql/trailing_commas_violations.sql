-- Test cases for SQL018: Trailing commas violations

-- Violations: Mixed comma styles (inconsistent)
SELECT 
    name,
    email
    , phone
    , address,
FROM users;

-- Violations: Another mixed style example
SELECT 
    u.name,
    u.email
    , o.total,
    o.date
FROM users u
JOIN orders o ON u.id = o.user_id;

-- OK: Consistent trailing commas
SELECT 
    name,
    email,
    phone,
    address,
FROM users;

-- OK: Consistent leading commas
SELECT 
    name
    , email
    , phone
    , address
FROM users;

-- OK: Single line (no comma style issue)
SELECT name, email, phone FROM users;

-- OK: Consistent traditional style (no trailing commas)
SELECT 
    name,
    email,
    phone,
    address
FROM users;