use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct UnitHash;

impl Rule for UnitHash {
    fn id(&self) -> &'static str {
        "RS060"
    }

    fn name(&self) -> &'static str {
        "unit-hash"
    }

    fn description(&self) -> &'static str {
        "Checks for hashing unit type ()"
    }

    fn explanation(&self) -> &'static str {
        "Hashing the unit type `()` is meaningless since all unit values are identical. \
         This usually indicates a logic error or unnecessary computation."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl UnitHash {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for method calls that involve hashing
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = &source[field_node.byte_range()];
                        
                        // Check if this is a hash-related method
                        if is_hash_method(method_name) {
                            // Check if the object being hashed is unit type
                            if let Some(object_node) = function_node.child_by_field_name("value") {
                                let object_text = &source[object_node.byte_range()].trim();
                                if is_unit_value(object_text) {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: format!(
                                            "Hashing unit type `{}` is meaningless - all unit values are identical",
                                            object_text
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

        // Also check for Hash trait implementations or hash function calls with unit values
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                let function_text = &source[function_node.byte_range()];
                if function_text.contains("hash") {
                    // Check arguments for unit values
                    if let Some(args_node) = node.child_by_field_name("arguments") {
                        let mut cursor = args_node.walk();
                        for child in args_node.children(&mut cursor) {
                            let arg_text = &source[child.byte_range()].trim();
                            if is_unit_value(arg_text) {
                                let position = node.start_position();
                                violations.push(LintViolation {
                                    line: position.row + 1,
                                    column: position.column + 1,
                                    message: format!(
                                        "Hashing unit type `{}` is meaningless - all unit values are identical",
                                        arg_text
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

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_hash_method(method_name: &str) -> bool {
    matches!(method_name,
        "hash" | "hasher" | "finish" | "write" |
        "hash_one" | "hash_slice"
    )
}

fn is_unit_value(text: &str) -> bool {
    // Check for unit literal "()"
    text == "()" ||
    // Check for function calls that return unit (common patterns)
    text.ends_with("()") && (
        text.ends_with("println!()") ||
        text.ends_with("print!()") ||
        text.ends_with("panic!()") ||
        text.ends_with("unreachable!()") ||
        text.ends_with("todo!()")
    )
}