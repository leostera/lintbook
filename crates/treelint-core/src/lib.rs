use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult<T> {
    pub file_path: PathBuf,
    #[serde(with = "serde_millis")]
    pub duration: Duration,
    pub status: LintStatus,
    pub violations: Vec<LintViolation>,
    pub language: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LintStatus {
    Ok,      // No violations found
    Error,   // Violations found
    Skipped, // File skipped (unsupported language, etc)
}

mod serde_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = duration.as_micros() as f64 / 1000.0;
        millis.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = f64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis as u64))
    }
}
