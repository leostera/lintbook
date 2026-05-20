-- SQL027: Avoid aliases in CTEs violations
-- CTEs already have names and shouldn't be aliased

-- VIOLATION: Aliasing a CTE
WITH user_orders AS (
    SELECT user_id, COUNT(*) as order_count
    FROM orders
    GROUP BY user_id
)
SELECT uo.user_id, uo.order_count
FROM user_orders uo
WHERE uo.order_count > 5;

-- VIOLATION: Multiple CTEs with aliases
WITH 
    active_users AS (
        SELECT * FROM users WHERE active = true
    ),
    recent_orders AS (
        SELECT * FROM orders WHERE order_date > '2024-01-01'
    )
SELECT au.name, ro.order_id
FROM active_users au
JOIN recent_orders ro ON au.id = ro.user_id;

-- GOOD: Using CTE names directly
WITH user_orders AS (
    SELECT user_id, COUNT(*) as order_count
    FROM orders
    GROUP BY user_id
)
SELECT user_orders.user_id, user_orders.order_count
FROM user_orders
WHERE user_orders.order_count > 5;