use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct LateFutureImport;

impl Rule for LateFutureImport {
    fn id(&self) -> &'static str {
        "PY034"
    }

    fn name(&self) -> &'static str {
        "late-future-import"
    }

    fn description(&self) -> &'static str {
        "Future import not at the beginning of the file"
    }

    fn explanation(&self) -> &'static str {
        "__future__ imports must appear at the beginning of the file, after module docstrings and comments but before any other code."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        let mut has_non_future_statement = false;
        let mut first_non_future_line = 0;
        
        self.visit_module_statements(root_node, source, &mut violations, &mut has_non_future_statement, &mut first_non_future_line);
        violations
    }
}

impl LateFutureImport {
    fn visit_module_statements(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>, has_non_future_statement: &mut bool, first_non_future_line: &mut usize) {
        // Only check top-level statements in the module
        if node.kind() != "module" {
            return;
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "expression_statement" => {
                        // Skip docstrings (string literals at module level)
                        if !self.is_module_docstring(child, i) {
                            *has_non_future_statement = true;
                            if *first_non_future_line == 0 {
                                *first_non_future_line = child.start_position().row + 1;
                            }
                        }
                    },
                    "import_statement" => {
                        *has_non_future_statement = true;
                        if *first_non_future_line == 0 {
                            *first_non_future_line = child.start_position().row + 1;
                        }
                    },
                    "import_from_statement" => {
                        if self.is_future_import(child, source) {
                            if *has_non_future_statement {
                                self.report_violation(child, *first_non_future_line, violations);
                            }
                        } else {
                            *has_non_future_statement = true;
                            if *first_non_future_line == 0 {
                                *first_non_future_line = child.start_position().row + 1;
                            }
                        }
                    },
                    "function_definition" | "async_function_definition" | "class_definition" |
                    "assignment" | "augmented_assignment" | "if_statement" | "for_statement" |
                    "while_statement" | "try_statement" | "with_statement" | "assert_statement" |
                    "global_statement" | "nonlocal_statement" | "del_statement" | "pass_statement" |
                    "break_statement" | "continue_statement" | "return_statement" | "raise_statement" => {
                        *has_non_future_statement = true;
                        if *first_non_future_line == 0 {
                            *first_non_future_line = child.start_position().row + 1;
                        }
                        
                        // Also check for future imports inside these constructs
                        self.check_nested_future_imports(child, source, violations);
                    },
                    // Comments and blank lines are allowed before future imports
                    "comment" => {},
                    _ => {
                        // Any other statement type should also count as non-future
                        *has_non_future_statement = true;
                        if *first_non_future_line == 0 {
                            *first_non_future_line = child.start_position().row + 1;
                        }
                        
                        // Check for nested future imports
                        self.check_nested_future_imports(child, source, violations);
                    }
                }
            }
        }
    }

    fn is_module_docstring(&self, node: Node, position: usize) -> bool {
        // A module docstring is a string literal as the first statement (position 0)
        // or after comments/blank lines
        if position > 2 { // Allow some flexibility for comments
            return false;
        }
        
        // Check if this expression statement contains just a string literal
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "string" {
                    return true;
                }
            }
        }
        false
    }

    fn is_future_import(&self, node: Node, source: &str) -> bool {
        // Check if this is "from __future__ import ..."
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            return text.trim().starts_with("from __future__");
        }
        false
    }

    fn check_nested_future_imports(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Recursively check for future imports in nested scopes
        if node.kind() == "import_from_statement" && self.is_future_import(node, source) {
            // Future import found in nested scope - always invalid
            self.report_violation(node, 0, violations);
            return;
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_nested_future_imports(child, source, violations);
            }
        }
    }

    fn report_violation(&self, node: Node, first_non_future_line: usize, violations: &mut Vec<LintViolation>) {
        let start_point = node.start_position();
        let message = if first_non_future_line > 0 {
            format!("__future__ import must occur at the beginning of the file (before line {})", first_non_future_line)
        } else {
            "__future__ import must occur at module level".to_string()
        };
        
        violations.push(LintViolation {
            line: start_point.row + 1,
            column: start_point.column + 1,
            message,
            lint_id: self.id().to_string(),
            lint_name: self.name().to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_python(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_future_import_at_beginning() {
        let source = r#"
from __future__ import annotations
from __future__ import unicode_literals

import os
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // Should be valid
    }

    #[test]
    fn test_future_import_after_regular_import() {
        let source = r#"
import os
from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("beginning of the file"));
    }

    #[test]
    fn test_future_import_after_function() {
        let source = r#"
def my_function():
    pass

from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("beginning of the file"));
    }

    #[test]
    fn test_future_import_after_variable() {
        let source = r#"
x = 1
from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("beginning of the file"));
    }

    #[test]
    fn test_future_import_inside_function() {
        let source = r#"
def bad_function():
    from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_future_import_inside_class() {
        let source = r#"
class BadClass:
    from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_future_import_after_docstring() {
        let source = r#"
"""Module docstring."""

from __future__ import annotations
import os
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // Should be valid - docstring is allowed before
    }

    #[test]
    fn test_future_import_in_if_statement() {
        let source = r#"
if True:
    from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_future_import_in_try_except() {
        let source = r#"
try:
    from __future__ import annotations
except ImportError:
    pass
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_multiple_future_imports_correct_order() {
        let source = r#"
from __future__ import annotations
from __future__ import unicode_literals
from __future__ import division

import os
import sys
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // All future imports at beginning
    }

    #[test]
    fn test_mixed_future_and_regular_imports() {
        let source = r#"
from __future__ import annotations
import os
from __future__ import unicode_literals
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Second future import is late
        assert!(violations[0].message.contains("unicode_literals"));
    }

    #[test]
    fn test_future_import_after_assignment() {
        let source = r#"
MODULE_CONSTANT = "value"
from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("beginning of the file"));
    }

    #[test]
    fn test_no_future_imports() {
        let source = r#"
import os
import sys

def function():
    pass
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // No future imports to check
    }

    #[test]
    fn test_future_import_after_class() {
        let source = r#"
class MyClass:
    pass

from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("beginning of the file"));
    }

    #[test]
    fn test_future_import_in_nested_function() {
        let source = r#"
def outer():
    def inner():
        from __future__ import annotations
"#;
        let tree = parse_python(source);
        let rule = LateFutureImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }
}