use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use treelint_config::{LlmLintConfig, LlmProviderConfig, TreelintConfig};
use treelint_lang::Grammar;

pub mod cache;
pub mod providers;
pub mod query_generator;

pub use cache::LlmCache;
pub use providers::{LlmProvider, LlmResponse};
pub use query_generator::QueryGenerator;

#[derive(Debug)]
pub struct LlmLinter {
    config: TreelintConfig,
    cache: LlmCache,
    provider: Box<dyn LlmProvider>,
    query_generator: QueryGenerator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmLintResult {
    pub file_path: PathBuf,
    pub lint_name: String,
    pub error_message: String,
    pub line: usize,
    pub column: usize,
}

impl LlmLinter {
    pub fn new(config: TreelintConfig, cache_dir: PathBuf) -> Result<Self> {
        let provider_config = config
            .treelint
            .llm
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No LLM provider configured"))?;

        let provider = providers::create_provider(provider_config)?;
        let cache = LlmCache::new(cache_dir)?;

        Ok(Self {
            config,
            cache,
            provider,
            query_generator: QueryGenerator::new(),
        })
    }

    pub async fn run_llm_lints(&mut self, files: &[PathBuf]) -> Result<Vec<LlmLintResult>> {
        let mut results = Vec::new();

        for (lint_name, lint_config) in &self.config.llm {
            let matched_files = self.filter_files_for_lint(files, lint_config)?;

            for file_path in matched_files {
                if let Some(lint_result) = self
                    .run_lint_on_file(lint_name, lint_config, &file_path)
                    .await?
                {
                    results.push(lint_result);
                }
            }
        }

        Ok(results)
    }

    fn filter_files_for_lint(
        &self,
        files: &[PathBuf],
        lint_config: &LlmLintConfig,
    ) -> Result<Vec<PathBuf>> {
        let mut gitignore_builder = GitignoreBuilder::new(".");
        gitignore_builder.add_line(None, &lint_config.files)?;
        let gitignore = gitignore_builder.build()?;

        let filtered_files: Vec<PathBuf> = files
            .iter()
            .filter(|file| {
                // Check language match
                if let Some(extension) = file.extension().and_then(|e| e.to_str()) {
                    let language_matches = match lint_config.lang.as_str() {
                        "python" => extension == "py",
                        "rust" => extension == "rs",
                        "javascript" => extension == "js",
                        "typescript" => extension == "ts",
                        _ => false,
                    };

                    language_matches && gitignore.matched(file, false).is_whitelist()
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(filtered_files)
    }

    async fn run_lint_on_file(
        &mut self,
        lint_name: &str,
        lint_config: &LlmLintConfig,
        file_path: &Path,
    ) -> Result<Option<LlmLintResult>> {
        // Read file content
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        // Generate tree-sitter query from match prompt (with caching)
        let query_prompt = format!(
            "Generate a tree-sitter query for {} that matches: {}",
            lint_config.lang, lint_config.match_prompt
        );

        let raw_query = self
            .cache
            .get_or_compute(&query_prompt, || async {
                self.provider
                    .generate_query(&lint_config.lang, &lint_config.match_prompt)
                    .await
            })
            .await?;

        let query_str = self.query_generator.extract_query_from_response(&raw_query);
        
        // Validate the query before using it
        self.query_generator.validate_query(&lint_config.lang, &query_str)?;

        // Parse the file with tree-sitter
        let grammar = Grammar::from_name(&lint_config.lang)?;
        let language = grammar.to_tree_sitter_language()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language)?;

        let tree = parser
            .parse(&content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse file"))?;

        // Run the query
        let query = tree_sitter::Query::new(language, &query_str)
            .map_err(|e| anyhow::anyhow!("Invalid tree-sitter query: {}", e))?;

        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        // If we have matches, validate with LLM
        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let node_text = &content[node.byte_range()];

                // Check with LLM if this is actually a problem
                let validation_prompt = format!(
                    "Is this code problematic? Context: {}\n\nCode:\n{}\n\nError to check for: {}",
                    lint_config.match_prompt, node_text, lint_config.error
                );

                let is_error = self
                    .cache
                    .get_or_compute(&validation_prompt, || async {
                        self.provider.validate_match(node_text, &lint_config.error).await
                    })
                    .await?;

                if is_error.contains("yes") || is_error.contains("true") {
                    let start_point = node.start_position();
                    return Ok(Some(LlmLintResult {
                        file_path: file_path.to_path_buf(),
                        lint_name: lint_name.to_string(),
                        error_message: lint_config.error.clone(),
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                    }));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use treelint_config::{LlmProviderConfig, TreelintSection};
    use std::collections::HashMap;

    fn create_test_config() -> TreelintConfig {
        let mut llm_lints = HashMap::new();
        llm_lints.insert(
            "test-lint".to_string(),
            LlmLintConfig {
                lang: "python".to_string(),
                files: "**/*.py".to_string(),
                match_prompt: "global variables".to_string(),
                error: "Avoid global variables".to_string(),
            },
        );

        TreelintConfig {
            treelint: TreelintSection {
                languages: vec!["python".to_string()],
                autofix: false,
                ignore: vec![],
                llm: Some(LlmProviderConfig {
                    provider: "mock".to_string(),
                    model: "test-model".to_string(),
                    api_key_env: None,
                }),
            },
            lints: HashMap::new(),
            llm: llm_lints,
        }
    }

    #[tokio::test]
    async fn test_llm_linter_creation() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config();
        
        let linter = LlmLinter::new(config, temp_dir.path().to_path_buf());
        assert!(linter.is_ok());
    }

    #[tokio::test]
    async fn test_filter_files_for_lint() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config();
        let linter = LlmLinter::new(config, temp_dir.path().to_path_buf()).unwrap();

        let lint_config = LlmLintConfig {
            lang: "python".to_string(),
            files: "**/*.py".to_string(),
            match_prompt: "test".to_string(),
            error: "test error".to_string(),
        };

        let files = vec![
            PathBuf::from("test.py"),
            PathBuf::from("test.rs"),
            PathBuf::from("script.py"),
        ];

        let filtered = linter.filter_files_for_lint(&files, &lint_config).unwrap();
        
        // Should only include .py files
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&PathBuf::from("test.py")));
        assert!(filtered.contains(&PathBuf::from("script.py")));
    }
}