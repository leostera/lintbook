use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct UnusedIoAmount;

impl Rule for UnusedIoAmount {
    fn id(&self) -> &'static str {
        "RS063"
    }

    fn name(&self) -> &'static str {
        "unused-io-amount"
    }

    fn description(&self) -> &'static str {
        "Checks for unused return values from I/O operations"
    }

    fn explanation(&self) -> &'static str {
        "I/O operations like read() and write() return the number of bytes processed. \
         Ignoring this value can lead to bugs when partial reads/writes occur."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl UnusedIoAmount {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for expression statements that ignore return values
        if node.kind() == "expression_statement" {
            if let Some(expr_node) = node.named_child(0) {
                if expr_node.kind() == "call_expression" {
                    if let Some(function_node) = expr_node.child_by_field_name("function") {
                        if function_node.kind() == "field_expression" {
                            if let Some(field_node) = function_node.child_by_field_name("field") {
                                let method_name = &source[field_node.byte_range()];

                                if is_io_method_with_amount(method_name) {
                                    let position = expr_node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: format!(
                                            "Unused return value from `{}()` - this method returns the number of bytes processed",
                                            method_name
                                        ),
                                        lint_name: self.name().to_string(),
                                        lint_id: self.id().to_string(),
                                    });
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

fn is_io_method_with_amount(method_name: &str) -> bool {
    matches!(
        method_name,
        "read"
            | "write"
            | "read_exact"
            | "write_all"
            | "read_to_end"
            | "read_to_string"
            | "read_vectored"
            | "write_vectored"
            | "read_buf"
            | "write_buf"
            | "copy"
            | "copy_buf"
    )
}
