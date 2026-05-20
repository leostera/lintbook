pub mod ex1001_exception_names;
pub mod ex1003_multi_alias_import_require_use;
pub mod ex1004_parameter_pattern_matching;
pub mod ex1005_space_around_operators;
pub mod ex1006_space_in_parentheses;
pub mod ex1008_unused_variable_names;
pub mod ex3003_variable_names;

use lintbook_core::Rule;

/// Returns all Elixir lint rules
pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ex1001_exception_names::ExceptionNames),
        Box::new(ex1003_multi_alias_import_require_use::MultiAliasImportRequireUse),
        Box::new(ex1004_parameter_pattern_matching::ParameterPatternMatching),
        Box::new(ex1005_space_around_operators::SpaceAroundOperators),
        Box::new(ex1006_space_in_parentheses::SpaceInParentheses),
        Box::new(ex1008_unused_variable_names::UnusedVariableNames),
        Box::new(ex3003_variable_names::VariableNames),
    ]
}
