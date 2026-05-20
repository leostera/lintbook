pub mod py020_break_outside_loop;
pub mod py021_continue_outside_loop;
pub mod py022_yield_outside_function;
pub mod py023_return_outside_function;
pub mod py026_return_in_init;
pub mod py027_nonlocal_and_global;
pub mod py028_continue_in_finally;
pub mod py029_duplicate_bases;
pub mod py030_invalid_all_object;
pub mod py031_invalid_all_format;
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
        Box::new(py026_return_in_init::ReturnInInit),
        Box::new(py027_nonlocal_and_global::NonlocalAndGlobal),
        Box::new(py028_continue_in_finally::ContinueInFinally),
        Box::new(py029_duplicate_bases::DuplicateBases),
        Box::new(py030_invalid_all_object::InvalidAllObject),
        Box::new(py031_invalid_all_format::InvalidAllFormat),
        Box::new(py032_misplaced_bare_raise::MisplacedBareRaise),
        Box::new(py033_unused_import::UnusedImport),
        Box::new(py034_late_future_import::LateFutureImport),
    ]
}
