use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct TransmuteNullToFn;

impl Rule for TransmuteNullToFn {
    fn id(&self) -> &'static str {
        "RS055"
    }

    fn name(&self) -> &'static str {
        "transmute-null-to-fn"
    }

    fn description(&self) -> &'static str {
        "Checks for transmuting null pointers to function pointers"
    }

    fn explanation(&self) -> &'static str {
        "Transmuting a null pointer to a function pointer is undefined behavior. \
         Use Option<fn()> or check for null before transmuting."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl TransmuteNullToFn {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                let function_text = &source[function_node.byte_range()];

                // Look for transmute calls
                if function_text.contains("transmute") {
                    if let Some(args_node) = node.child_by_field_name("arguments") {
                        // Check if first argument is null-like
                        let mut cursor = args_node.walk();
                        for child in args_node.children(&mut cursor) {
                            if is_null_value(child, source) {
                                // Check if this is likely transmuting to a function pointer
                                if could_be_fn_transmute(node, source) {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: "Transmuting null pointer to function pointer is undefined behavior. Use Option<fn()> instead".to_string(),
                                        lint_name: self.name().to_string(),
                                        lint_id: self.id().to_string(),
                                    });
                                }
                                break;
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

fn is_null_value(node: Node, source: &str) -> bool {
    let text = source[node.byte_range()].trim();
    matches!(
        text,
        "0" | "0usize"
            | "0isize"
            | "0u64"
            | "0i64"
            | "null()"
            | "null_mut()"
            | "std::ptr::null()"
            | "std::ptr::null_mut()"
            | "ptr::null()"
            | "ptr::null_mut()"
    )
}

fn could_be_fn_transmute(node: Node, source: &str) -> bool {
    // Look for type annotations or context that suggests function pointer
    let text = &source[node.byte_range()];
    text.contains("fn(")
        || text.contains("fn ")
        || text.contains("function")
        || text.contains("*const fn")
        || text.contains("*mut fn")
}
