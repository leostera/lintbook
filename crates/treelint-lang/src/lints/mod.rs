pub mod python;

use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintViolation {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub lint_name: String,
    pub lint_id: String,
}

/// Trait for lint rule implementations
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn explanation(&self) -> &'static str;
    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation>;
}
