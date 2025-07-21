-- Test cases for SQL004: Identifier capitalization violations

-- Violations: Mixed case table names
CREATE TABLE UserProfiles (
    id INT PRIMARY KEY,
    firstName VARCHAR(50),
    lastName VARCHAR(50)
);

-- Violations: Mixed case in SELECT
SELECT firstName, lastName, EmailAddress FROM userProfiles;

-- Violations: Mixed case in FROM
SELECT name FROM UserAccounts WHERE active = 1;

-- OK: snake_case (consistent)
CREATE TABLE user_profiles (
    id INT PRIMARY KEY,
    first_name VARCHAR(50),
    last_name VARCHAR(50)
);

SELECT first_name, last_name, email_address FROM user_profiles;

-- OK: PascalCase (consistent)
CREATE TABLE UserProfiles (
    ID INT PRIMARY KEY,
    FirstName VARCHAR(50),
    LastName VARCHAR(50)
);

SELECT FirstName, LastName FROM UserProfiles;