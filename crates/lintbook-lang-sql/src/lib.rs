pub mod sql001_table_aliasing_style;
pub mod sql002_column_aliasing_style;
pub mod sql003_keyword_capitalization;
pub mod sql004_identifier_capitalization;
pub mod sql005_not_equal_operator_consistency;
pub mod sql006_coalesce_over_ifnull;
pub mod sql007_is_null_vs_equal_null;
pub mod sql008_explicit_union_all;
pub mod sql009_explicit_join_types;
pub mod sql010_remove_redundant_else_null;
pub mod sql011_references_in_from;
pub mod sql012_keywords_as_identifiers;
pub mod sql013_no_sp_prefix;
pub mod sql014_complex_expressions_need_aliases;
pub mod sql015_unique_table_aliases;
pub mod sql016_consistent_count_syntax;
pub mod sql017_distinct_group_by_conflict;
pub mod sql018_trailing_commas;
pub mod sql019_unused_table_aliases;
pub mod sql020_semicolon_placement;
pub mod sql021_consistent_order_by_directions;
pub mod sql022_quote_style_consistency;
pub mod sql023_type_casting_style;
pub mod sql024_wildcard_column_ambiguity;
pub mod sql025_unnecessary_quoted_identifiers;
pub mod sql026_table_alias_length;
pub mod sql027_avoid_aliases_in_ctes;
pub mod sql028_column_names_in_group_order_by;
pub mod sql029_quoted_literals;
pub mod sql030_distinct_values_in_clause;
pub mod sql031_leading_whitespace;
pub mod sql032_trailing_whitespace;
pub mod sql033_nested_case_statements;
pub mod sql034_boolean_value_expressions;
pub mod sql035_comparison_operators;
pub mod sql036_line_length;
pub mod sql037_join_condition_order;
pub mod sql038_nested_subqueries;
pub mod sql039_qualification_consistency;
pub mod sql040_consistent_table_references;
pub mod sql041_commas_placement;
pub mod sql042_line_breaks_in_clauses;
pub mod sql043_cte_vs_subquery;
pub mod sql044_wildcard_in_count;
pub mod sql045_operators_spacing;
pub mod sql046_unnecessary_distinct;
pub mod sql047_undefined_functions;
pub mod sql048_performance_anti_patterns;
pub mod sql049_security_patterns;
pub mod sql050_data_type_best_practices;
pub mod sql051_transaction_patterns;
pub mod sql052_index_usage_hints;
pub mod sql053_schema_validation;
pub mod sql054_query_complexity;
pub mod sql055_temporal_patterns;
pub mod sql056_error_handling;
pub mod sql057_stored_procedure_patterns;
pub mod sql058_view_patterns;
pub mod sql059_trigger_patterns;
pub mod sql060_cursor_patterns;
pub mod sql061_backup_restore_patterns;
pub mod sql062_partition_patterns;
pub mod sql063_replication_patterns;
pub mod sql064_deprecated_features;
pub mod sql065_compatibility_patterns;

use lintbook_core::Rule;

pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        // Temporarily disabled - missing implementations
        Box::new(sql001_table_aliasing_style::TableAliasingStyle),
        Box::new(sql002_column_aliasing_style::ColumnAliasingStyle),
        Box::new(sql003_keyword_capitalization::KeywordCapitalization),
        Box::new(sql004_identifier_capitalization::IdentifierCapitalization),
        Box::new(sql005_not_equal_operator_consistency::NotEqualOperatorConsistency),
        Box::new(sql006_coalesce_over_ifnull::CoalesceOverIfnull),
        Box::new(sql007_is_null_vs_equal_null::IsNullVsEqualNull),
        Box::new(sql008_explicit_union_all::ExplicitUnionAll),
        Box::new(sql009_explicit_join_types::ExplicitJoinTypes),
        Box::new(sql010_remove_redundant_else_null::RemoveRedundantElseNull),
        Box::new(sql011_references_in_from::ReferencesInFrom),
        Box::new(sql012_keywords_as_identifiers::KeywordsAsIdentifiers),
        Box::new(sql013_no_sp_prefix::NoSpPrefix),
        Box::new(sql014_complex_expressions_need_aliases::ComplexExpressionsNeedAliases),
        Box::new(sql015_unique_table_aliases::UniqueTableAliases),
        Box::new(sql016_consistent_count_syntax::ConsistentCountSyntax),
        Box::new(sql017_distinct_group_by_conflict::DistinctGroupByConflict),
        Box::new(sql018_trailing_commas::TrailingCommas),
        Box::new(sql019_unused_table_aliases::UnusedTableAliases),
        Box::new(sql020_semicolon_placement::SemicolonPlacement),
        Box::new(sql021_consistent_order_by_directions::ConsistentOrderByDirections),
        Box::new(sql022_quote_style_consistency::QuoteStyleConsistency),
        Box::new(sql023_type_casting_style::TypeCastingStyle),
        Box::new(sql024_wildcard_column_ambiguity::WildcardColumnAmbiguity),
        Box::new(sql025_unnecessary_quoted_identifiers::UnnecessaryQuotedIdentifiers),
        Box::new(sql026_table_alias_length::TableAliasLength),
        Box::new(sql027_avoid_aliases_in_ctes::AvoidAliasesInCtes),
        Box::new(sql028_column_names_in_group_order_by::ColumnNamesInGroupOrderBy),
        Box::new(sql029_quoted_literals::QuotedLiterals),
        Box::new(sql030_distinct_values_in_clause::DistinctValuesInClause),
        Box::new(sql031_leading_whitespace::LeadingWhitespace),
        Box::new(sql032_trailing_whitespace::TrailingWhitespace),
        Box::new(sql033_nested_case_statements::NestedCaseStatements),
        Box::new(sql034_boolean_value_expressions::BooleanValueExpressions),
        Box::new(sql035_comparison_operators::ComparisonOperators),
        Box::new(sql036_line_length::LineLength),
        Box::new(sql037_join_condition_order::JoinConditionOrder),
        Box::new(sql038_nested_subqueries::NestedSubqueries),
        Box::new(sql039_qualification_consistency::QualificationConsistency),
        Box::new(sql040_consistent_table_references::ConsistentTableReferences),
        Box::new(sql041_commas_placement::CommasPlacement),
        Box::new(sql042_line_breaks_in_clauses::LineBreaksInClauses),
        Box::new(sql043_cte_vs_subquery::CteVsSubquery),
        Box::new(sql044_wildcard_in_count::WildcardInCount),
        Box::new(sql045_operators_spacing::OperatorsSpacing),
        Box::new(sql046_unnecessary_distinct::UnnecessaryDistinct),
        Box::new(sql047_undefined_functions::UndefinedFunctions),
        Box::new(sql048_performance_anti_patterns::PerformanceAntiPatterns),
        Box::new(sql049_security_patterns::SecurityPatterns),
        Box::new(sql050_data_type_best_practices::DataTypeBestPractices),
        Box::new(sql051_transaction_patterns::TransactionPatterns),
        Box::new(sql052_index_usage_hints::IndexUsageHints),
        Box::new(sql053_schema_validation::SchemaValidation),
        Box::new(sql054_query_complexity::QueryComplexity),
        Box::new(sql055_temporal_patterns::TemporalPatterns),
        Box::new(sql056_error_handling::ErrorHandling),
        Box::new(sql057_stored_procedure_patterns::StoredProcedurePatterns),
        Box::new(sql058_view_patterns::ViewPatterns),
        Box::new(sql059_trigger_patterns::TriggerPatterns),
        Box::new(sql060_cursor_patterns::CursorPatterns),
        Box::new(sql061_backup_restore_patterns::BackupRestorePatterns),
        Box::new(sql062_partition_patterns::PartitionPatterns),
        Box::new(sql063_replication_patterns::ReplicationPatterns),
        Box::new(sql064_deprecated_features::DeprecatedFeatures),
        Box::new(sql065_compatibility_patterns::CompatibilityPatterns),
    ]
}
