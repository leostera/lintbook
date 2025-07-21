use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use treelint_lang::{get_grammars_for_extensions, lints::LintViolation, Grammar};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub file_path: PathBuf,
    #[serde(with = "serde_millis")]
    pub duration: Duration,
    pub status: LintStatus,
    pub violations: Vec<LintViolation>,
    pub language: Option<Grammar>,
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

pub struct Scanner;

impl Scanner {
    pub fn find_repo_root(start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path.to_path_buf();

        loop {
            if current.join(".git").exists() {
                return Ok(current);
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Err(anyhow::anyhow!("No git repository found")),
            }
        }
    }

    pub fn scan_files(repo_root: &Path) -> Result<Vec<String>> {
        let mut extensions = Vec::new();

        fn visit_dir(dir: &Path, extensions: &mut Vec<String>) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    let dir_name = path.file_name().unwrap().to_string_lossy();
                    if !dir_name.starts_with('.')
                        && dir_name != "target"
                        && dir_name != "node_modules"
                    {
                        visit_dir(&path, extensions)?;
                    }
                } else if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_string();
                    if !extensions.contains(&ext) {
                        extensions.push(ext);
                    }
                }
            }
            Ok(())
        }

        visit_dir(repo_root, &mut extensions)?;
        Ok(extensions)
    }

    pub fn detect_grammars(repo_root: &Path) -> Result<Vec<Grammar>> {
        let extensions = Self::scan_files(repo_root)?;
        let grammars = get_grammars_for_extensions(&extensions);
        Ok(grammars)
    }

    pub fn scan_and_lint_files(
        repo_root: &Path,
        config: &treelint_config::TreelintConfig,
    ) -> Result<Vec<LintResult>> {
        let mut results = Vec::new();

        fn visit_dir(
            dir: &Path,
            results: &mut Vec<LintResult>,
            config: &treelint_config::TreelintConfig,
        ) -> Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    let dir_name = path.file_name().unwrap().to_string_lossy();
                    if !dir_name.starts_with('.')
                        && dir_name != "target"
                        && dir_name != "node_modules"
                    {
                        visit_dir(&path, results, config)?;
                    }
                } else if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy();

                    // Check if we support this language
                    if let Some(grammar) = treelint_lang::get_grammar_for_extension(&ext) {
                        // Check if this language is enabled in config
                        if config
                            .treelint
                            .languages
                            .contains(&grammar.name().to_string())
                        {
                            let start_time = std::time::Instant::now();

                            // Read file
                            let source = match std::fs::read_to_string(&path) {
                                Ok(content) => content,
                                Err(_) => {
                                    results.push(LintResult {
                                        file_path: path.clone(),
                                        duration: start_time.elapsed(),
                                        status: LintStatus::Skipped,
                                        violations: vec![],
                                        language: None,
                                    });
                                    continue;
                                }
                            };

                            // Parse with tree-sitter
                            let mut parser = tree_sitter::Parser::new();
                            let language = match grammar {
                                Grammar::Python => tree_sitter_python::LANGUAGE.into(),
                                _ => {
                                    // Skip unsupported parsers for now
                                    results.push(LintResult {
                                        file_path: path.clone(),
                                        duration: start_time.elapsed(),
                                        status: LintStatus::Skipped,
                                        violations: vec![],
                                        language: None,
                                    });
                                    continue;
                                }
                            };

                            parser.set_language(&language).unwrap();

                            if let Some(tree) = parser.parse(&source, None) {
                                // Run lints
                                let mut all_violations = Vec::new();
                                let lints = grammar.get_lints();

                                for lint in lints {
                                    if config.is_lint_enabled(grammar.name(), lint.name()) {
                                        let mut violations = lint.check(&tree, &source);
                                        // Ensure violations have the correct lint metadata
                                        for v in &mut violations {
                                            v.lint_id = lint.id().to_string();
                                            v.lint_name = lint.name().to_string();
                                        }
                                        all_violations.extend(violations);
                                    }
                                }

                                let status = if all_violations.is_empty() {
                                    LintStatus::Ok
                                } else {
                                    LintStatus::Error
                                };

                                results.push(LintResult {
                                    file_path: path.clone(),
                                    duration: start_time.elapsed(),
                                    status,
                                    violations: all_violations,
                                    language: Some(grammar),
                                });
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dir(repo_root, &mut results, config)?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_files_finds_extensions() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("test.py"), "print('hello')").unwrap();
        fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("config.toml"), "[test]").unwrap();

        let extensions = Scanner::scan_files(temp_path).unwrap();

        assert!(extensions.contains(&"py".to_string()));
        assert!(extensions.contains(&"rs".to_string()));
        assert!(extensions.contains(&"toml".to_string()));
    }

    #[test]
    fn test_detect_grammars() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("test.py"), "print('hello')").unwrap();
        fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();

        let grammars = Scanner::detect_grammars(temp_path).unwrap();

        assert!(grammars.contains(&treelint_lang::Grammar::Python));
        assert!(grammars.contains(&treelint_lang::Grammar::Rust));
    }

    #[test]
    fn test_scan_files_ignores_hidden_directories() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create hidden directory with file
        let hidden_dir = temp_path.join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(hidden_dir.join("secret.py"), "print('secret')").unwrap();

        // Create normal file
        fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();

        let extensions = Scanner::scan_files(temp_path).unwrap();

        assert!(extensions.contains(&"rs".to_string()));
        // Should not find .py from hidden directory
        assert!(!extensions.contains(&"py".to_string()) || extensions.len() == 1);
    }
}
