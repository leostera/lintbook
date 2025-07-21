pub mod ex1001_exception_names;
pub mod ex1002_line_endings;
pub mod ex3001_iex_pry;
pub mod ex3002_io_inspect;
pub mod ex4001_unsafe_to_atom;
pub mod ex5001_function_names;
pub mod ex5002_module_names;

use treelint_core::Rule;

/// Returns all Elixir lint rules
pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ex1001_exception_names::ExceptionNames),
        Box::new(ex1002_line_endings::LineEndings),
        Box::new(ex3001_iex_pry::IExPry),
        Box::new(ex3002_io_inspect::IoInspect),
        Box::new(ex4001_unsafe_to_atom::UnsafeToAtom),
        Box::new(ex5001_function_names::FunctionNames),
        Box::new(ex5002_module_names::ModuleNames),
    ]
}
