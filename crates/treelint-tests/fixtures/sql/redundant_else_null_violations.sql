-- Test cases for SQL010: Redundant ELSE NULL violations

-- Violations: Explicit ELSE NULL (redundant)
SELECT 
    name,
    CASE 
        WHEN age >= 18 THEN 'Adult'
        WHEN age >= 13 THEN 'Teen'
        ELSE NULL
    END as age_category
FROM users;

-- Violations: Multiple CASE statements with redundant ELSE NULL
SELECT 
    CASE status 
        WHEN 'active' THEN 'Active User'
        WHEN 'pending' THEN 'Pending Approval'
        ELSE NULL
    END as status_desc,
    CASE priority
        WHEN 1 THEN 'High'
        WHEN 2 THEN 'Medium'
        ELSE NULL 
    END as priority_desc
FROM tasks;

-- OK: CASE without ELSE (implicit NULL)
SELECT 
    name,
    CASE 
        WHEN age >= 18 THEN 'Adult'
        WHEN age >= 13 THEN 'Teen'
    END as age_category
FROM users;

-- OK: CASE with meaningful ELSE value
SELECT 
    CASE status 
        WHEN 'active' THEN 'Active User'
        WHEN 'pending' THEN 'Pending Approval'
        ELSE 'Unknown Status'
    END as status_desc
FROM users;