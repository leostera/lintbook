use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct TransmutingNull;

impl Rule for TransmutingNull {
    fn id(&self) -> &'static str {
        "RS056"
    }

    fn name(&self) -> &'static str {
        "transmuting-null"
    }

    fn description(&self) -> &'static str {
        "Checks for transmuting null pointers"
    }

    fn explanation(&self) -> &'static str {
        "Transmuting null pointers can lead to undefined behavior. \
         Consider using Option<T> or proper null checks instead."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl TransmutingNull {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                let function_text = &source[function_node.byte_range()];

                // Look for transmute calls
                if is_transmute_call(function_text) {
                    if let Some(args_node) = node.child_by_field_name("arguments") {
                        // Check if any argument is null-like
                        let mut cursor = args_node.walk();
                        for child in args_node.children(&mut cursor) {
                            if is_null_value(child, source) {
                                let position = node.start_position();
                                violations.push(LintViolation {
                                    line: position.row + 1,
                                    column: position.column + 1,
                                    message: "Transmuting null pointer can lead to undefined behavior. Consider using Option<T> instead".to_string(),
                                    lint_name: self.name().to_string(),
                                    lint_id: self.id().to_string(),
                                });
                                break; // Only report once per transmute call
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

fn is_transmute_call(function_text: &str) -> bool {
    function_text.contains("transmute")
        && (function_text == "transmute"
            || function_text.ends_with("::transmute")
            || function_text.contains("mem::transmute")
            || function_text.contains("std::mem::transmute"))
}

fn is_null_value(node: Node, source: &str) -> bool {
    let text = source[node.byte_range()].trim();
    matches!(
        text,
        "0" | "0usize"
            | "0isize"
            | "0u64"
            | "0i64"
            | "0u32"
            | "0i32"
            | "0u16"
            | "0i16"
            | "0u8"
            | "0i8"
            | "null()"
            | "null_mut()"
            | "std::ptr::null()"
            | "std::ptr::null_mut()"
            | "ptr::null()"
            | "ptr::null_mut()"
            | "NonNull::dangling()"
            | "0 as *const _"
            | "0 as *mut _"
            | "0usize as *const _"
            | "0usize as *mut _"
    ) || text.starts_with("0 as *")
        || text.contains("as *") && text.contains("0")
}
