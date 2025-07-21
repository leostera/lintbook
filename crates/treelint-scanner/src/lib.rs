use anyhow::Result;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use treelint_core::*;
use treelint_lang::*;

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
    ) -> Result<Vec<LintResult<Grammar>>> {
        use std::sync::{Arc, Mutex};

        // Collect enabled extensions for quick filtering
        let enabled_extensions: HashSet<String> = config
            .treelint
            .languages
            .iter()
            .filter_map(|lang| {
                treelint_lang::get_supported_grammars()
                    .into_iter()
                    .find(|g| g.name() == lang)
            })
            .flat_map(|g| g.extensions().iter().map(|e| e.to_string()))
            .collect();

        // Build walker with minimal features for performance
        let mut walker = WalkBuilder::new(repo_root);
        walker
            .hidden(false) // Don't skip hidden files by default
            .ignore(true) // Honor .ignore files
            .git_ignore(true) // Honor .gitignore files
            .git_global(false) // Skip global gitignore for performance
            .git_exclude(false) // Skip .git/info/exclude for performance
            .threads(num_cpus::get()); // Use all available CPU cores

        // Only add custom ignore patterns if needed
        if !config.treelint.ignore.is_empty() {
            let mut overrides = ignore::overrides::OverrideBuilder::new(repo_root);
            for pattern in &config.treelint.ignore {
                // Add patterns as globs to exclude (! prefix means exclude)
                overrides.add(&format!("!{}", pattern))?;
            }
            walker.overrides(overrides.build()?);
        }

        // Add file type filters to reduce the number of files we need to check
        if !enabled_extensions.is_empty() {
            let mut types_builder = ignore::types::TypesBuilder::new();
            for ext in &enabled_extensions {
                types_builder.add("treelint", &format!("*.{}", ext))?;
            }
            types_builder.select("treelint");
            walker.types(types_builder.build()?);
        }

        // Use a thread-safe vector to collect results
        let results = Arc::new(Mutex::new(Vec::new()));

        // Run the parallel walker
        walker.build_parallel().run(|| {
            let config = config.clone();
            let results = Arc::clone(&results);

            Box::new(move |entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return ignore::WalkState::Continue,
                };

                let path = entry.path();

                // Skip directories
                if path.is_dir() {
                    return ignore::WalkState::Continue;
                }

                // Check file extension
                if let Some(extension) = path.extension() {
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
                            let source = std::fs::read_to_string(&path).unwrap();
                            let mut results_lock = results.lock().unwrap();
                            results_lock.push(treelint_lang::parse(&config, path, &source, grammar, start_time));
                        }
                    }
                }

                ignore::WalkState::Continue
            })
        });

        // Extract results from Arc<Mutex<Vec<_>>>
        let results = Arc::try_unwrap(results)
            .unwrap_or_else(|_| panic!("Failed to unwrap results"))
            .into_inner()
            .unwrap();

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
