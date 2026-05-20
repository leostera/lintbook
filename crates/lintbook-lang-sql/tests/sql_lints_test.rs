use lintbook_core::Rule;

fn check_violations(rule: &dyn Rule, source: &str) -> Vec<String> {
    // For now, we'll use a dummy tree since we're doing pattern matching
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_sequel::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();

    let violations = rule.check(&tree, source);
    violations.into_iter().map(|v| v.message).collect()
}

#[test]
fn test_sql001_table_aliasing_style() {
    let rule = lintbook_lang_sql::sql001_table_aliasing_style::TableAliasingStyle;

    // Should find violations for implicit aliases
    let violations = check_violations(&rule, "SELECT * FROM users u");
    assert!(!violations.is_empty());

    // Should not find violations for explicit AS
    let violations = check_violations(&rule, "SELECT * FROM users AS u");
    assert!(violations.is_empty());
}

#[test]
fn test_sql002_column_aliasing_style() {
    let rule = lintbook_lang_sql::sql002_column_aliasing_style::ColumnAliasingStyle;

    // Should find violations for implicit aliases
    let violations = check_violations(&rule, "SELECT name n FROM users");
    assert!(!violations.is_empty());

    // Should not find violations for explicit AS
    let violations = check_violations(&rule, "SELECT name AS n FROM users");
    assert!(violations.is_empty());
}

#[test]
fn test_sql003_keyword_capitalization() {
    let rule = lintbook_lang_sql::sql003_keyword_capitalization::KeywordCapitalization;

    // Should find violations for lowercase keywords
    let violations = check_violations(&rule, "select * from users where id = 1");
    assert!(!violations.is_empty());

    // Should not find violations for uppercase keywords
    let violations = check_violations(&rule, "SELECT * FROM users WHERE id = 1");
    assert!(violations.is_empty());
}

#[test]
fn test_sql005_not_equal_consistency() {
    let rule =
        lintbook_lang_sql::sql005_not_equal_operator_consistency::NotEqualOperatorConsistency;

    // Should find violations for mixed operators
    let source = "SELECT * FROM users WHERE a != 1 AND b <> 2";
    let violations = check_violations(&rule, source);
    assert!(!violations.is_empty());
}

#[test]
fn test_sql007_is_null_vs_equal_null() {
    let rule = lintbook_lang_sql::sql007_is_null_vs_equal_null::IsNullVsEqualNull;

    // Should find violations for = NULL
    let violations = check_violations(&rule, "SELECT * FROM users WHERE name = NULL");
    assert!(!violations.is_empty());

    // Should not find violations for IS NULL
    let violations = check_violations(&rule, "SELECT * FROM users WHERE name IS NULL");
    assert!(violations.is_empty());
}

#[test]
fn test_sql008_explicit_union_all() {
    let rule = lintbook_lang_sql::sql008_explicit_union_all::ExplicitUnionAll;

    // Should find violations for plain UNION
    let violations = check_violations(&rule, "SELECT 1 UNION SELECT 2");
    assert!(!violations.is_empty());

    // Should not find violations for UNION ALL
    let violations = check_violations(&rule, "SELECT 1 UNION ALL SELECT 2");
    assert!(violations.is_empty());
}

#[test]
fn test_sql009_explicit_join_types() {
    let rule = lintbook_lang_sql::sql009_explicit_join_types::ExplicitJoinTypes;

    // Should find violations for plain JOIN
    let violations = check_violations(&rule, "SELECT * FROM a JOIN b ON a.id = b.id");
    assert!(!violations.is_empty());

    // Should not find violations for INNER JOIN
    let violations = check_violations(&rule, "SELECT * FROM a INNER JOIN b ON a.id = b.id");
    assert!(violations.is_empty());
}

#[test]
fn test_sql010_redundant_else_null() {
    let rule = lintbook_lang_sql::sql010_remove_redundant_else_null::RemoveRedundantElseNull;

    // Should find violations for ELSE NULL
    let violations = check_violations(&rule, "SELECT CASE WHEN x = 1 THEN 'one' ELSE NULL END");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql012_keywords_as_identifiers() {
    let rule = lintbook_lang_sql::sql012_keywords_as_identifiers::KeywordsAsIdentifiers;

    // Should find violations for unquoted keywords
    let violations = check_violations(&rule, "SELECT user, select, from FROM table");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql018_trailing_commas() {
    let rule = lintbook_lang_sql::sql018_trailing_commas::TrailingCommas;

    // Should find violations for trailing commas
    let violations = check_violations(&rule, "SELECT a, b, FROM users");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql022_quote_style_consistency() {
    let rule = lintbook_lang_sql::sql022_quote_style_consistency::QuoteStyleConsistency;

    // Should find violations for mixed quotes
    let violations = check_violations(&rule, "SELECT 'hello' AS a, \"world\" AS b");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql024_wildcard_ambiguity() {
    let rule = lintbook_lang_sql::sql024_wildcard_column_ambiguity::WildcardColumnAmbiguity;

    // Should find violations for SELECT * in joins
    let violations = check_violations(
        &rule,
        "SELECT * FROM users JOIN orders ON users.id = orders.user_id",
    );
    assert!(!violations.is_empty());
}

#[test]
fn test_sql026_table_alias_length() {
    let rule = lintbook_lang_sql::sql026_table_alias_length::TableAliasLength;

    // Should find violations for single-letter aliases in complex queries
    let violations = check_violations(
        &rule,
        "SELECT * FROM users u JOIN orders o ON u.id = o.user_id",
    );
    assert!(!violations.is_empty());

    // Should not find violations for meaningful aliases
    let violations = check_violations(
        &rule,
        "SELECT * FROM users usr JOIN orders ord ON usr.id = ord.user_id",
    );
    assert!(violations.is_empty());
}

#[test]
fn test_sql027_avoid_aliases_in_ctes() {
    let rule = lintbook_lang_sql::sql027_avoid_aliases_in_ctes::AvoidAliasesInCtes;

    // Should find violations for aliased CTEs
    let violations = check_violations(
        &rule,
        "WITH user_data AS (SELECT * FROM users) SELECT * FROM user_data ud",
    );
    assert!(!violations.is_empty());
}

#[test]
fn test_sql028_column_names_in_group_order_by() {
    let rule = lintbook_lang_sql::sql028_column_names_in_group_order_by::ColumnNamesInGroupOrderBy;

    // Should find violations for positional references
    let violations = check_violations(
        &rule,
        "SELECT name, COUNT(*) FROM users GROUP BY 1 ORDER BY 2",
    );
    assert!(!violations.is_empty());
}

#[test]
fn test_sql029_quoted_literals() {
    let rule = lintbook_lang_sql::sql029_quoted_literals::QuotedLiterals;

    // Should find violations for double-quoted string literals
    let violations = check_violations(&rule, "SELECT * FROM users WHERE name = \"John\"");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql030_distinct_values_in_clause() {
    let rule = lintbook_lang_sql::sql030_distinct_values_in_clause::DistinctValuesInClause;

    // Should find violations for duplicate values in IN clause
    let violations = check_violations(&rule, "SELECT * FROM users WHERE id IN (1, 2, 1, 3)");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql034_boolean_value_expressions() {
    let rule = lintbook_lang_sql::sql034_boolean_value_expressions::BooleanValueExpressions;

    // Should find violations for = TRUE
    let violations = check_violations(&rule, "SELECT * FROM users WHERE is_active = TRUE");
    assert!(!violations.is_empty());

    // Should find violations for = FALSE
    let violations = check_violations(&rule, "SELECT * FROM users WHERE is_deleted = FALSE");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql035_comparison_operators() {
    let rule = lintbook_lang_sql::sql035_comparison_operators::ComparisonOperators;

    // Should find violations for non-standard operators
    let violations = check_violations(&rule, "SELECT * FROM users WHERE age !< 18");
    assert!(!violations.is_empty());
}

#[test]
fn test_sql036_line_length() {
    let rule = lintbook_lang_sql::sql036_line_length::LineLength;

    // Should find violations for lines over 120 characters
    let long_line = "SELECT customer_id, first_name, last_name, email_address, phone_number, street_address, city, state, postal_code, country, registration_date FROM customers WHERE active = true";
    let violations = check_violations(&rule, long_line);
    assert!(!violations.is_empty());
}
