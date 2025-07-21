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

use treelint_core::Rule;

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
    ]
}
