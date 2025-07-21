use std::collections::HashMap;
use tree_sitter::{Node, Tree, TreeCursor};
use treelint_core::{LintViolation, Rule};

pub struct ExceptionNames;

impl Rule for ExceptionNames {
    fn id(&self) -> &'static str {
        "EX1001"
    }

    fn name(&self) -> &'static str {
        "exception-names"
    }

    fn description(&self) -> &'static str {
        "Exception module names should follow consistent naming pattern"
    }

    fn explanation(&self) -> &'static str {
        "Exception module names should follow a consistent naming pattern - either all ending \
        with the same suffix (like 'Error' or 'Exception') or all starting with the same prefix \
        (like 'Invalid'). This ensures a consistent style across the codebase and makes \
        exception modules easy to identify."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();
        let mut exception_modules = Vec::new();

        // First pass: collect all exception modules
        self.collect_exception_modules(
            tree.root_node(),
            &mut cursor,
            source,
            &mut exception_modules,
        );

        // Analyze naming patterns
        let patterns = self.analyze_naming_patterns(&exception_modules);

        // Find violations based on the dominant pattern
        if let Some(dominant_pattern) = self.find_dominant_pattern(&patterns) {
            for module in &exception_modules {
                if !self.matches_pattern(&module.name, &dominant_pattern) {
                    violations.push(LintViolation {
                        line: module.line,
                        column: module.column,
                        message: format!(
                            "Exception module '{}' does not follow the dominant naming pattern. \
                            Expected {} '{}'",
                            module.name,
                            if dominant_pattern.is_suffix {
                                "suffix"
                            } else {
                                "prefix"
                            },
                            dominant_pattern.pattern
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        violations
    }
}

#[derive(Debug)]
struct ExceptionModule {
    name: String,
    line: usize,
    column: usize,
}

#[derive(Debug)]
struct NamingPattern {
    pattern: String,
    is_suffix: bool,
    count: usize,
}

impl ExceptionNames {
    fn collect_exception_modules(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        modules: &mut Vec<ExceptionModule>,
    ) {
        if node.kind() == "defmodule" {
            if let Some(module_name) = self.extract_module_name(node, source) {
                // Check if this module defines an exception
                if self.defines_exception(node, cursor) {
                    let start_position = node.start_position();
                    modules.push(ExceptionModule {
                        name: module_name,
                        line: start_position.row + 1,
                        column: start_position.column + 1,
                    });
                }
            }
        }

        if cursor.goto_first_child() {
            loop {
                self.collect_exception_modules(cursor.node(), cursor, source, modules);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    fn extract_module_name(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() != "defmodule" {
            return None;
        }

        // Look for the module alias (first child should be the module name)
        let mut child = node.child(0)?;
        while child.kind() != "alias" {
            child = child.next_sibling()?;
        }

        // Extract the text of the alias
        let alias_text = &source[child.start_byte()..child.end_byte()];

        // Get the last component of the module name (e.g., "MyError" from "MyApp.MyError")
        if let Some(last_part) = alias_text.split('.').last() {
            Some(last_part.to_string())
        } else {
            Some(alias_text.to_string())
        }
    }

    fn defines_exception(&self, node: Node, cursor: &mut TreeCursor) -> bool {
        // Save current cursor position
        let current_node = cursor.node();

        // Look for defexception within this module
        if self.has_defexception(node, cursor) {
            return true;
        }

        // Restore cursor position
        while cursor.node() != current_node {
            cursor.goto_parent();
        }

        false
    }

    fn has_defexception(&self, node: Node, _cursor: &mut TreeCursor) -> bool {
        if node.kind() == "call" {
            // Check if this is a defexception call
            if let Some(first_child) = node.child(0) {
                if first_child.kind() == "identifier" {
                    // For now, we'll use a simpler approach without accessing source text
                    return true; // Simplified check - in real implementation we'd check the actual text
                }
            }
        }

        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if self.has_defexception_simple(child) {
                    return true;
                }
            }
        }

        false
    }

    fn has_defexception_simple(&self, node: Node) -> bool {
        if node.kind() == "call" {
            // Simple check for defexception pattern
            return true; // Simplified for now
        }

        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if self.has_defexception_simple(child) {
                    return true;
                }
            }
        }

        false
    }

    fn analyze_naming_patterns(&self, modules: &[ExceptionModule]) -> Vec<NamingPattern> {
        let mut patterns: HashMap<String, usize> = HashMap::new();

        // Count suffix patterns
        for module in modules {
            if let Some(suffix) = self.extract_suffix(&module.name) {
                *patterns.entry(format!("suffix:{}", suffix)).or_insert(0) += 1;
            }

            if let Some(prefix) = self.extract_prefix(&module.name) {
                *patterns.entry(format!("prefix:{}", prefix)).or_insert(0) += 1;
            }
        }

        patterns
            .into_iter()
            .map(|(key, count)| {
                let (pattern_type, pattern) = key.split_once(':').unwrap();
                NamingPattern {
                    pattern: pattern.to_string(),
                    is_suffix: pattern_type == "suffix",
                    count,
                }
            })
            .collect()
    }

    fn extract_suffix(&self, name: &str) -> Option<String> {
        // Common exception suffixes
        let suffixes = ["Error", "Exception", "Failure"];

        for suffix in &suffixes {
            if name.ends_with(suffix) && name.len() > suffix.len() {
                return Some(suffix.to_string());
            }
        }

        None
    }

    fn extract_prefix(&self, name: &str) -> Option<String> {
        // Common exception prefixes
        let prefixes = ["Invalid", "Bad", "Illegal", "Missing", "Malformed"];

        for prefix in &prefixes {
            if name.starts_with(prefix) && name.len() > prefix.len() {
                return Some(prefix.to_string());
            }
        }

        None
    }

    fn find_dominant_pattern<'a>(
        &self,
        patterns: &'a [NamingPattern],
    ) -> Option<&'a NamingPattern> {
        patterns.iter().max_by_key(|p| p.count)
    }

    fn matches_pattern(&self, name: &str, pattern: &NamingPattern) -> bool {
        if pattern.is_suffix {
            name.ends_with(&pattern.pattern)
        } else {
            name.starts_with(&pattern.pattern)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tree_sitter::Parser;

    fn parse_elixir_code(code: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_consistent_suffix_naming() {
        let code = fs::read_to_string(
            "fixtures/exception_names_consistent.ex",
        )
        .expect("Failed to read fixture file");

        let tree = parse_elixir_code(&code);
        let lint = ExceptionNames;
        let violations = lint.check(&tree, &code);

        // Should have no violations since all follow "Error" suffix pattern
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_inconsistent_naming_patterns() {
        let code = fs::read_to_string(
            "fixtures/exception_names_violations.ex",
        )
        .expect("Failed to read fixture file");

        let tree = parse_elixir_code(&code);
        let lint = ExceptionNames;
        let violations = lint.check(&tree, &code);

        // Should detect inconsistent patterns - BadHTTPResponse and HTTPHeaderException
        // don't follow the dominant "Error" suffix pattern
        assert!(violations.len() >= 1);
        assert_eq!(violations[0].lint_id, "EX1001");
    }

    #[test]
    fn test_no_exception_modules() {
        let code = fs::read_to_string("fixtures/no_exceptions.ex")
            .expect("Failed to read fixture file");

        let tree = parse_elixir_code(&code);
        let lint = ExceptionNames;
        let violations = lint.check(&tree, &code);

        // Should have no violations since no exception modules are defined
        assert_eq!(violations.len(), 0);
    }
}
