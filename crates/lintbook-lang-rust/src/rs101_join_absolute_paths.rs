use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct JoinAbsolutePaths;

impl Rule for JoinAbsolutePaths {
    fn id(&self) -> &'static str {
        "RS101"
    }

    fn name(&self) -> &'static str {
        "join-absolute-paths"
    }

    fn description(&self) -> &'static str {
        "Checks for joining absolute paths with Path::join"
    }

    fn explanation(&self) -> &'static str {
        "Using Path::join with absolute paths replaces the base path entirely. \
         This might not be the intended behavior."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl JoinAbsolutePaths {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = &source[field_node.byte_range()];

                        if method_name == "join" {
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                let mut cursor = args_node.walk();
                                for child in args_node.children(&mut cursor) {
                                    let arg_text = source[child.byte_range()].trim();
                                    if is_absolute_path(arg_text) {
                                        let position = node.start_position();
                                        violations.push(LintViolation {
                                            line: position.row + 1,
                                            column: position.column + 1,
                                            message: "Joining absolute path replaces the base path entirely".to_string(),
                                            lint_name: self.name().to_string(),
                                            lint_id: self.id().to_string(),
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_absolute_path(path: &str) -> bool {
    // Remove quotes if present
    let cleaned = path.trim_matches('"').trim_matches('\'');

    // Unix absolute paths start with /
    cleaned.starts_with('/') ||
    // Windows absolute paths start with drive letter or UNC
    (cleaned.len() >= 3 && cleaned.chars().nth(1) == Some(':') && cleaned.chars().nth(2) == Some('\\')) ||
    cleaned.starts_with("\\\\")
}
