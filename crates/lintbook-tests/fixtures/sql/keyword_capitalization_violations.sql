-- Test cases for SQL003: Keyword capitalization violations

-- Violations: All lowercase keywords
select name, email from users where id = 1;

-- Violations: Mixed case keywords
Select name, Email From users Where id = 1;

-- Violations: Complex query with mixed cases
select u.name, count(p.id) as post_count
from users u
join posts p on u.id = p.user_id
where u.active = true
group by u.name
having count(p.id) > 5
order by post_count desc;

-- OK: All uppercase keywords
SELECT name, email FROM users WHERE id = 1;

-- OK: Complex query with proper capitalization
SELECT u.name, COUNT(p.id) AS post_count
FROM users AS u
JOIN posts AS p ON u.id = p.user_id
WHERE u.active = TRUE
GROUP BY u.name
HAVING COUNT(p.id) > 5
ORDER BY post_count DESC;