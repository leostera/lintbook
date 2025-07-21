-- SQL033: Nested CASE statements violations
-- Avoid deeply nested CASE statements (max 2 levels)

-- VIOLATION: 3 levels of nesting
SELECT 
  CASE 
    WHEN category = 'A' THEN
      CASE 
        WHEN subcategory = '1' THEN
          CASE 
            WHEN status = 'active' THEN 'A1-Active'
            WHEN status = 'inactive' THEN 'A1-Inactive'
            ELSE 'A1-Unknown'
          END
        WHEN subcategory = '2' THEN 'A2'
        ELSE 'A-Other'
      END
    WHEN category = 'B' THEN 'B'
    ELSE 'Other'
  END as category_label
FROM products;

-- VIOLATION: 4 levels of nesting
SELECT 
  CASE region
    WHEN 'US' THEN
      CASE state
        WHEN 'CA' THEN
          CASE city
            WHEN 'LA' THEN
              CASE district
                WHEN 'Downtown' THEN 'US-CA-LA-DT'
                ELSE 'US-CA-LA-Other'
              END
            ELSE 'US-CA-Other'
          END
        ELSE 'US-Other'
      END
    ELSE 'International'
  END as location_code
FROM addresses;

-- GOOD: Maximum 2 levels of nesting
SELECT 
  CASE 
    WHEN category = 'A' THEN
      CASE 
        WHEN subcategory = '1' THEN 'A1'
        WHEN subcategory = '2' THEN 'A2'
        ELSE 'A-Other'
      END
    WHEN category = 'B' THEN 'B'
    ELSE 'Other'
  END as category_label
FROM products;

-- GOOD: Refactored using CTEs instead of deep nesting
WITH categorized_products AS (
  SELECT 
    *,
    CASE 
      WHEN category = 'A' AND subcategory = '1' THEN 'A1'
      WHEN category = 'A' AND subcategory = '2' THEN 'A2'
      WHEN category = 'A' THEN 'A-Other'
      WHEN category = 'B' THEN 'B'
      ELSE 'Other'
    END as base_category
  FROM products
)
SELECT 
  *,
  CASE 
    WHEN base_category = 'A1' AND status = 'active' THEN 'A1-Active'
    WHEN base_category = 'A1' AND status = 'inactive' THEN 'A1-Inactive'
    WHEN base_category = 'A1' THEN 'A1-Unknown'
    ELSE base_category
  END as category_label
FROM categorized_products;