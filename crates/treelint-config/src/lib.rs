use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct TreelintConfig {
    pub treelint: TreelintSection,
    #[serde(default)]
    pub lints: HashMap<String, HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreelintSection {
    pub languages: Vec<String>,
    pub autofix: bool,
}

impl TreelintConfig {
    pub fn new(grammar_names: Vec<&str>) -> Self {
        let languages = grammar_names
            .into_iter()
            .map(|name| name.to_string())
            .collect();

        Self {
            treelint: TreelintSection {
                languages,
                autofix: true,
            },
            lints: HashMap::new(),
        }
    }

    pub fn is_lint_enabled(&self, language: &str, lint_name: &str) -> bool {
        self.lints
            .get(language)
            .and_then(|lang_lints| lang_lints.get(lint_name))
            .copied()
            .unwrap_or(true) // Default to enabled
    }

    pub fn write_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let toml_content = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_content)?;
        Ok(())
    }

    pub fn read_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_new_config() {
        let grammars = vec!["python", "rust", "javascript"];
        let config = TreelintConfig::new(grammars);

        assert_eq!(config.treelint.languages.len(), 3);
        assert!(config.treelint.languages.contains(&"python".to_string()));
        assert!(config.treelint.languages.contains(&"rust".to_string()));
        assert!(config
            .treelint
            .languages
            .contains(&"javascript".to_string()));
        assert_eq!(config.treelint.autofix, true);
        assert!(config.lints.is_empty());
    }

    #[test]
    fn test_is_lint_enabled_default() {
        let config = TreelintConfig::new(vec!["python"]);

        // Should default to enabled when not specified
        assert_eq!(config.is_lint_enabled("python", "no-try-catch"), true);
        assert_eq!(config.is_lint_enabled("rust", "unknown-lint"), true);
    }

    #[test]
    fn test_is_lint_enabled_explicit() {
        let mut config = TreelintConfig::new(vec!["python"]);

        // Add explicit lint configuration
        let mut python_lints = HashMap::new();
        python_lints.insert("no-try-catch".to_string(), false);
        config.lints.insert("python".to_string(), python_lints);

        assert_eq!(config.is_lint_enabled("python", "no-try-catch"), false);
        assert_eq!(config.is_lint_enabled("python", "other-lint"), true); // Default
    }

    #[test]
    fn test_write_and_read_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let original_config = TreelintConfig::new(vec!["python", "rust"]);
        original_config.write_to_file(&config_path).unwrap();

        let loaded_config = TreelintConfig::read_from_file(&config_path).unwrap();

        assert_eq!(loaded_config.treelint.languages.len(), 2);
        assert!(loaded_config
            .treelint
            .languages
            .contains(&"python".to_string()));
        assert!(loaded_config
            .treelint
            .languages
            .contains(&"rust".to_string()));
        assert_eq!(loaded_config.treelint.autofix, true);
    }

    #[test]
    fn test_config_toml_format() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("format_test.toml");

        let config = TreelintConfig::new(vec!["python", "rust"]);
        config.write_to_file(&config_path).unwrap();

        let toml_content = fs::read_to_string(&config_path).unwrap();

        assert!(toml_content.contains("[treelint]"));
        assert!(toml_content.contains("languages = ["));
        assert!(toml_content.contains("\"python\""));
        assert!(toml_content.contains("\"rust\""));
        assert!(toml_content.contains("autofix = true"));
        assert!(toml_content.contains("[lints]"));
    }
}
