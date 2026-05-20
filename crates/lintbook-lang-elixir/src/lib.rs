pub mod ex1004_parameter_pattern_matching;

use lintbook_core::Rule;

/// Returns all Elixir lint rules
pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![Box::new(
        ex1004_parameter_pattern_matching::ParameterPatternMatching,
    )]
}
