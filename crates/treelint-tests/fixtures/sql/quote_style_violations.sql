-- Test cases for SQL022: Quote style consistency violations

-- Violations: Mixed quote styles
SELECT 'John' as first_name, "Doe" as last_name FROM users;

-- Violations: More mixed usage
INSERT INTO users (name, email) VALUES ('Alice', "alice@example.com");

-- Violations: Different queries with different styles
SELECT * FROM users WHERE status = 'active';
UPDATE users SET role = "admin" WHERE id = 1;

-- OK: Consistent single quotes
SELECT 'John' as first_name, 'Doe' as last_name FROM users;
INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');

-- OK: Consistent double quotes
SELECT "John" as first_name, "Doe" as last_name FROM users;
INSERT INTO users (name, email) VALUES ("Alice", "alice@example.com");