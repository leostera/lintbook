pub mod ex1001_exception_names;
pub mod ex1002_line_endings;
pub mod ex1003_multi_alias_import_require_use;
pub mod ex1004_parameter_pattern_matching;
pub mod ex1005_space_around_operators;
pub mod ex1006_space_in_parentheses;
pub mod ex1007_tabs_or_spaces;
pub mod ex1008_unused_variable_names;
pub mod ex3001_iex_pry;
pub mod ex3002_io_inspect;
pub mod ex3003_variable_names;
pub mod ex3010_trailing_whitespace;
pub mod ex3011_semicolons;
pub mod ex4001_unsafe_to_atom;
pub mod ex5001_function_names;
pub mod ex5002_module_names;
pub mod ex5003_unsafe_exec;
pub mod ex5006_dbg;

use lintbook_core::Rule;

/// Returns all Elixir lint rules
pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ex1001_exception_names::ExceptionNames),
        Box::new(ex1002_line_endings::LineEndings),
        Box::new(ex1003_multi_alias_import_require_use::MultiAliasImportRequireUse),
        Box::new(ex1004_parameter_pattern_matching::ParameterPatternMatching),
        Box::new(ex1005_space_around_operators::SpaceAroundOperators),
        Box::new(ex1006_space_in_parentheses::SpaceInParentheses),
        Box::new(ex1007_tabs_or_spaces::TabsOrSpaces),
        Box::new(ex1008_unused_variable_names::UnusedVariableNames),
        Box::new(ex3001_iex_pry::IExPry),
        Box::new(ex3002_io_inspect::IoInspect),
        Box::new(ex3003_variable_names::VariableNames),
        Box::new(ex3010_trailing_whitespace::TrailingWhitespace),
        Box::new(ex3011_semicolons::Semicolons),
        Box::new(ex4001_unsafe_to_atom::UnsafeToAtom),
        Box::new(ex5001_function_names::FunctionNames),
        Box::new(ex5002_module_names::ModuleNames),
        Box::new(ex5003_unsafe_exec::UnsafeExec),
        Box::new(ex5006_dbg::Dbg),
    ]
}
