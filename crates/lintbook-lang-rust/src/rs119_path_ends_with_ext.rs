use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct PathEndsWithExt;

impl Rule for PathEndsWithExt {
    fn id(&self) -> &'static str {
        "RS119"
    }

    fn name(&self) -> &'static str {
        "path-ends-with-ext"
    }

    fn description(&self) -> &'static str {
        "Checks for inefficient path extension checks"
    }

    fn explanation(&self) -> &'static str {
        "Using .ends_with(\".ext\") to check file extensions is inefficient. \
         Use .extension() == Some(\"ext\") for proper extension checking."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl PathEndsWithExt {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = source[field_node.byte_range()].trim();

                        if method_name == "ends_with" {
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                let mut cursor = args_node.walk();
                                for child in args_node.children(&mut cursor) {
                                    if matches!(child.kind(), "string_literal") {
                                        let arg_text = source[child.byte_range()].trim();
                                        if is_file_extension(arg_text) {
                                            let position = node.start_position();
                                            violations.push(LintViolation {
                                                line: position.row + 1,
                                                column: position.column + 1,
                                                message: format!(
                                                    "Use .extension() == Some({}) instead of .ends_with({}) for file extension checks",
                                                    &arg_text[1..], arg_text
                                                ),
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
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_file_extension(text: &str) -> bool {
    // Remove quotes and check if it's a file extension pattern
    let cleaned = text.trim_matches('"').trim_matches('\'');
    cleaned.starts_with('.') &&
    cleaned.len() > 1 &&
    cleaned.len() <= 5 &&  // Common extensions are short
    cleaned.chars().skip(1).all(|c| c.is_ascii_alphanumeric())
}
