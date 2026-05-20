pub mod py020_break_outside_loop;
pub mod py021_continue_outside_loop;
pub mod py022_yield_outside_function;
pub mod py023_return_outside_function;
pub mod py027_nonlocal_and_global;
pub mod py032_misplaced_bare_raise;
pub mod py033_unused_import;
pub mod py034_late_future_import;

use lintbook_core::Rule;

/// Returns all Python lint rules
pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(py020_break_outside_loop::BreakOutsideLoop),
        Box::new(py021_continue_outside_loop::ContinueOutsideLoop),
        Box::new(py022_yield_outside_function::YieldOutsideFunction),
        Box::new(py023_return_outside_function::ReturnOutsideFunction),
        Box::new(py027_nonlocal_and_global::NonlocalAndGlobal),
        Box::new(py032_misplaced_bare_raise::MisplacedBareRaise),
        Box::new(py033_unused_import::UnusedImport),
        Box::new(py034_late_future_import::LateFutureImport),
    ]
}
