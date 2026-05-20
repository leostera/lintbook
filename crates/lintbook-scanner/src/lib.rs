use anyhow::Result;
use globset::{Glob, GlobSet};
use ignore::WalkBuilder;
use lintbook_core::*;
use lintbook_lang::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
        config: &lintbook_config::LintbookConfig,
        active_rule_languages: &HashSet<String>,
    ) -> Result<Vec<LintResult<Grammar>>> {
        use std::sync::{Arc, Mutex};

        let results = Arc::new(Mutex::new(Vec::new()));
        let results_for_handler = Arc::clone(&results);
        Self::scan_and_lint_files_with(repo_root, config, active_rule_languages, move |result| {
            results_for_handler.lock().unwrap().push(result);
            Ok(())
        })?;

        let results = Arc::try_unwrap(results)
            .unwrap_or_else(|_| panic!("Failed to unwrap results"))
            .into_inner()
            .unwrap();

        Ok(results)
    }

    pub fn scan_and_lint_targets(
        repo_root: &Path,
        current_dir: &Path,
        targets: &[String],
        config: &lintbook_config::LintbookConfig,
        active_rule_languages: &HashSet<String>,
    ) -> Result<Vec<LintResult<Grammar>>> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let results_for_handler = Arc::clone(&results);
        Self::scan_and_lint_targets_with(
            repo_root,
            current_dir,
            targets,
            config,
            active_rule_languages,
            move |result| {
                results_for_handler.lock().unwrap().push(result);
                Ok(())
            },
        )?;

        let results = Arc::try_unwrap(results)
            .unwrap_or_else(|_| panic!("Failed to unwrap results"))
            .into_inner()
            .unwrap();

        Ok(results)
    }

    pub fn scan_and_lint_targets_with<F>(
        repo_root: &Path,
        current_dir: &Path,
        targets: &[String],
        config: &lintbook_config::LintbookConfig,
        active_rule_languages: &HashSet<String>,
        handler: F,
    ) -> Result<()>
    where
        F: Fn(LintResult<Grammar>) -> Result<()> + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        let seen = Arc::new(Mutex::new(HashSet::new()));
        let mut directory_roots = Vec::new();
        let mut glob_roots = Vec::new();
        let mut glob_builder = globset::GlobSetBuilder::new();
        let mut has_globs = false;

        for target in targets {
            if contains_glob_meta(target) {
                let pattern = resolve_target_pattern(current_dir, target);
                glob_builder.add(Glob::new(&pattern)?);
                glob_roots.push(glob_walk_root(current_dir, target));
                has_globs = true;
                continue;
            }

            let path = resolve_target_path(current_dir, target);
            if !path.exists() {
                anyhow::bail!("Check target not found: {}", target);
            }
            if path.is_dir() {
                directory_roots.push(path);
                continue;
            }

            let normalized = normalize_seen_path(&path);
            if !seen.lock().unwrap().insert(normalized) {
                continue;
            }
            if let Some(result) = Self::lint_file(&path, config, active_rule_languages)? {
                handler(result)?;
            }
        }

        if directory_roots.is_empty() && glob_roots.is_empty() {
            return Ok(());
        }

        if !directory_roots.is_empty() {
            Self::scan_roots_and_lint_files_with(
                repo_root,
                &directory_roots,
                None,
                Some(Arc::clone(&seen)),
                config,
                active_rule_languages,
                Arc::clone(&handler),
            )?;
        }

        if has_globs {
            Self::scan_roots_and_lint_files_with(
                repo_root,
                &glob_roots,
                Some(Arc::new(glob_builder.build()?)),
                Some(seen),
                config,
                active_rule_languages,
                handler,
            )?;
        }

        Ok(())
    }

    pub fn scan_and_lint_files_with<F>(
        repo_root: &Path,
        config: &lintbook_config::LintbookConfig,
        active_rule_languages: &HashSet<String>,
        handler: F,
    ) -> Result<()>
    where
        F: Fn(LintResult<Grammar>) -> Result<()> + Send + Sync + 'static,
    {
        Self::scan_roots_and_lint_files_with(
            repo_root,
            &[repo_root.to_path_buf()],
            None,
            None,
            config,
            active_rule_languages,
            Arc::new(handler),
        )
    }

    pub fn lint_file(
        path: &Path,
        config: &lintbook_config::LintbookConfig,
        active_rule_languages: &HashSet<String>,
    ) -> Result<Option<LintResult<Grammar>>> {
        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            return Ok(None);
        };
        let Some(grammar) = lintbook_lang::get_grammar_for_extension(extension) else {
            return Ok(None);
        };
        if !config
            .lintbook
            .languages
            .contains(&grammar.name().to_string())
            || (!active_rule_languages.contains(grammar.name()) && grammar.lints().is_empty())
        {
            return Ok(None);
        }

        let start_time = std::time::Instant::now();
        if grammar.lints().is_empty() {
            return Ok(Some(LintResult {
                file_path: path.to_path_buf(),
                duration: start_time.elapsed(),
                status: LintStatus::Ok,
                violations: vec![],
                language: Some(grammar),
            }));
        }

        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(_) => return Ok(None),
        };
        Ok(Some(lintbook_lang::parse(
            config, path, &source, grammar, start_time,
        )))
    }

    fn scan_roots_and_lint_files_with<F>(
        repo_root: &Path,
        roots: &[PathBuf],
        include_globs: Option<Arc<GlobSet>>,
        seen: Option<Arc<Mutex<HashSet<PathBuf>>>>,
        config: &lintbook_config::LintbookConfig,
        active_rule_languages: &HashSet<String>,
        handler: Arc<F>,
    ) -> Result<()>
    where
        F: Fn(LintResult<Grammar>) -> Result<()> + Send + Sync + 'static,
    {
        // Collect enabled extensions for quick filtering
        let enabled_extensions: HashSet<String> = config
            .lintbook
            .languages
            .iter()
            .filter_map(|lang| {
                if !active_rule_languages.contains(lang) {
                    return None;
                }
                lintbook_lang::get_supported_grammars()
                    .into_iter()
                    .find(|g| g.name() == lang)
            })
            .flat_map(|g| g.extensions().iter().map(|e| e.to_string()))
            .collect();

        let callback_error = Arc::new(Mutex::new(None));

        for root in roots {
            let mut walker = WalkBuilder::new(root);
            walker
                .hidden(false) // Don't skip hidden files by default
                .ignore(true) // Honor .ignore files
                .git_ignore(true) // Honor .gitignore files
                .git_global(false) // Skip global gitignore for performance
                .git_exclude(false) // Skip .git/info/exclude for performance
                .threads(num_cpus::get()); // Use all available CPU cores

            // Only add custom ignore patterns if needed
            if !config.lintbook.ignore.is_empty() {
                let mut overrides = ignore::overrides::OverrideBuilder::new(repo_root);
                for pattern in &config.lintbook.ignore {
                    // Add patterns as globs to exclude (! prefix means exclude)
                    overrides.add(&format!("!{}", pattern))?;
                }
                walker.overrides(overrides.build()?);
            }

            // Add file type filters to reduce the number of files we need to check
            if !enabled_extensions.is_empty() {
                let mut types_builder = ignore::types::TypesBuilder::new();
                for ext in &enabled_extensions {
                    types_builder.add("lintbook", &format!("*.{}", ext))?;
                }
                types_builder.select("lintbook");
                walker.types(types_builder.build()?);
            }

            // Run the parallel walker
            walker.build_parallel().run(|| {
                let config = config.clone();
                let active_rule_languages = active_rule_languages.clone();
                let handler = Arc::clone(&handler);
                let callback_error = Arc::clone(&callback_error);
                let include_globs = include_globs.clone();
                let seen = seen.clone();

                Box::new(move |entry| {
                    if callback_error.lock().unwrap().is_some() {
                        return ignore::WalkState::Quit;
                    }

                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => return ignore::WalkState::Continue,
                    };

                    let path = entry.path();

                    // Skip directories
                    if path.is_dir() {
                        return ignore::WalkState::Continue;
                    }

                    if let Some(include_globs) = &include_globs {
                        if !include_globs.is_match(path) {
                            return ignore::WalkState::Continue;
                        }
                    }

                    if let Some(seen) = &seen {
                        let normalized = normalize_seen_path(path);
                        if !seen.lock().unwrap().insert(normalized) {
                            return ignore::WalkState::Continue;
                        }
                    }

                    let result = match Self::lint_file(path, &config, &active_rule_languages) {
                        Ok(Some(result)) => result,
                        Ok(None) => return ignore::WalkState::Continue,
                        Err(error) => {
                            *callback_error.lock().unwrap() = Some(error);
                            return ignore::WalkState::Quit;
                        }
                    };

                    if let Err(error) = handler(result) {
                        *callback_error.lock().unwrap() = Some(error);
                        return ignore::WalkState::Quit;
                    };

                    ignore::WalkState::Continue
                })
            });

            if callback_error.lock().unwrap().is_some() {
                break;
            }
        }

        if let Some(error) = Arc::try_unwrap(callback_error)
            .unwrap_or_else(|_| panic!("Failed to unwrap callback error"))
            .into_inner()
            .unwrap()
        {
            return Err(error);
        }

        Ok(())
    }
}

fn resolve_target_path(current_dir: &Path, target: &str) -> PathBuf {
    let path = Path::new(target);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    }
}

fn resolve_target_pattern(current_dir: &Path, target: &str) -> String {
    resolve_target_path(current_dir, target)
        .to_string_lossy()
        .replace('\\', "/")
}

fn contains_glob_meta(target: &str) -> bool {
    target
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'['))
}

fn glob_walk_root(current_dir: &Path, target: &str) -> PathBuf {
    let pattern = resolve_target_pattern(current_dir, target);
    let first_meta = pattern
        .find(|ch| matches!(ch, '*' | '?' | '['))
        .unwrap_or(pattern.len());
    let prefix = pattern[..first_meta].trim_end_matches('/').to_string();
    if prefix.is_empty() {
        return current_dir.to_path_buf();
    }

    let mut root = if pattern[..first_meta].ends_with('/') {
        PathBuf::from(prefix)
    } else {
        PathBuf::from(&prefix)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| current_dir.to_path_buf())
    };

    while !root.exists() {
        let Some(parent) = root.parent() else {
            return current_dir.to_path_buf();
        };
        root = parent.to_path_buf();
    }

    if root.is_file() {
        root.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| current_dir.to_path_buf())
    } else {
        root
    }
}

fn normalize_seen_path(path: &Path) -> PathBuf {
    path.to_path_buf()
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

        assert!(grammars.contains(&lintbook_lang::Grammar::Python));
        assert!(grammars.contains(&lintbook_lang::Grammar::Rust));
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
