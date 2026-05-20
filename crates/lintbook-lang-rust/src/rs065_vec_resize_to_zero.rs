use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct VecResizeToZero;

impl Rule for VecResizeToZero {
    fn id(&self) -> &'static str {
        "RS065"
    }

    fn name(&self) -> &'static str {
        "vec-resize-to-zero"
    }

    fn description(&self) -> &'static str {
        "Checks for Vec::resize with size 0"
    }

    fn explanation(&self) -> &'static str {
        "Using Vec::resize(0, _) is equivalent to Vec::clear() but less clear. Use clear() instead."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl VecResizeToZero {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = &source[field_node.byte_range()];

                        if method_name == "resize" {
                            // Check if first argument is 0
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                let mut cursor = args_node.walk();
                                for child in args_node.children(&mut cursor) {
                                    let arg_text = source[child.byte_range()].trim();
                                    if arg_text == "0" {
                                        let position = node.start_position();
                                        violations.push(LintViolation {
                                            line: position.row + 1,
                                            column: position.column + 1,
                                            message: "Vec::resize(0, _) is equivalent to Vec::clear() - use clear() for better clarity".to_string(),
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

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}
