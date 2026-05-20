pub mod py033_unused_import;
pub mod py034_late_future_import;

use lintbook_core::Rule;

/// Returns all Python lint rules
pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(py033_unused_import::UnusedImport),
        Box::new(py034_late_future_import::LateFutureImport),
    ]
}
