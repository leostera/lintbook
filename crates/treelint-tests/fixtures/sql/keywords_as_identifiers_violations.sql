-- Test cases for SQL012: Keywords as identifiers violations

-- Violations: SQL keywords as table names
CREATE TABLE user (
    id INT PRIMARY KEY,
    name VARCHAR(100)
);

CREATE TABLE order (
    id INT,
    date DATE,
    user_id INT
);

-- Violations: SQL keywords as column names
CREATE TABLE products (
    id INT PRIMARY KEY,
    name VARCHAR(100),
    date DATE,
    user VARCHAR(50),
    table_id INT
);

-- Violations: SQL keywords as aliases
SELECT u.name AS user, o.date AS date 
FROM users AS u 
JOIN orders AS o ON u.id = o.user_id;

-- OK: Non-keyword identifiers
CREATE TABLE customer (
    id INT PRIMARY KEY,
    full_name VARCHAR(100)
);

-- OK: Quoted keywords (acceptable)
CREATE TABLE "user" (
    id INT PRIMARY KEY,
    "name" VARCHAR(100)
);

-- OK: Proper identifiers
SELECT u.full_name AS customer_name, o.order_date AS purchase_date
FROM customers AS u 
JOIN orders AS o ON u.id = o.customer_id;