use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct SuspiciousSplitN;

impl Rule for SuspiciousSplitN {
    fn id(&self) -> &'static str {
        "RS054"
    }

    fn name(&self) -> &'static str {
        "suspicious-splitn"
    }

    fn description(&self) -> &'static str {
        "Checks for suspicious splitn calls with n=0 or n=1"
    }

    fn explanation(&self) -> &'static str {
        "Calling .splitn(0) returns an empty iterator, and .splitn(1) returns the original string \
         as a single element. These are usually not the intended behavior and may indicate a logic error."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SuspiciousSplitN {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            // Check if this is a method call
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = &source[field_node.byte_range()];
                        
                        // Check for splitn, rsplitn methods
                        if method_name == "splitn" || method_name == "rsplitn" {
                            // Check the arguments
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                // Look for the first argument (the n parameter)
                                let mut cursor = args_node.walk();
                                for child in args_node.children(&mut cursor) {
                                    if child.kind() == "integer_literal" {
                                        let arg_text = &source[child.byte_range()];
                                        if let Ok(n) = arg_text.parse::<i32>() {
                                            if n == 0 || n == 1 {
                                                let position = node.start_position();
                                                let message = if n == 0 {
                                                    format!("Suspicious `{}(0)` call - returns empty iterator", method_name)
                                                } else {
                                                    format!("Suspicious `{}(1)` call - returns original string as single element", method_name)
                                                };
                                                
                                                violations.push(LintViolation {
                                                    line: position.row + 1,
                                                    column: position.column + 1,
                                                    message,
                                                    lint_name: self.name().to_string(),
                                                    lint_id: self.id().to_string(),
                                                });
                                            }
                                        }
                                        break; // Only check the first argument
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