-- SQL038: Nested subqueries violations
-- Avoid deeply nested subqueries (max 2 levels)

-- VIOLATION: 3 levels of nesting
SELECT *
FROM users
WHERE id IN (
    SELECT user_id
    FROM orders
    WHERE product_id IN (
        SELECT product_id
        FROM products
        WHERE category_id IN (
            SELECT category_id
            FROM categories
            WHERE name = 'Electronics'
        )
    )
);

-- VIOLATION: 4 levels of nesting
SELECT name
FROM employees
WHERE department_id = (
    SELECT department_id
    FROM departments
    WHERE manager_id = (
        SELECT employee_id
        FROM employees
        WHERE team_id = (
            SELECT team_id
            FROM teams
            WHERE location_id = (
                SELECT location_id
                FROM locations
                WHERE city = 'New York'
            )
        )
    )
);

-- GOOD: Maximum 2 levels of nesting
SELECT *
FROM users
WHERE id IN (
    SELECT user_id
    FROM orders
    WHERE total > (
        SELECT AVG(total)
        FROM orders
    )
);

-- GOOD: Refactored using CTEs instead of deep nesting
WITH electronic_categories AS (
    SELECT category_id
    FROM categories
    WHERE name = 'Electronics'
),
electronic_products AS (
    SELECT product_id
    FROM products
    WHERE category_id IN (SELECT category_id FROM electronic_categories)
),
electronic_orders AS (
    SELECT user_id
    FROM orders
    WHERE product_id IN (SELECT product_id FROM electronic_products)
)
SELECT *
FROM users
WHERE id IN (SELECT user_id FROM electronic_orders);