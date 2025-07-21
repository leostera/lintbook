use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct ConstIsEmpty;

impl Rule for ConstIsEmpty {
    fn id(&self) -> &'static str {
        "RS081"
    }

    fn name(&self) -> &'static str {
        "const-is-empty"
    }

    fn description(&self) -> &'static str {
        "Checks for checking if constant-sized collections are empty"
    }

    fn explanation(&self) -> &'static str {
        "Checking if a constant-sized collection is empty when the size is known at compile time \
         is redundant. Use the known size directly in conditional logic."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl ConstIsEmpty {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = &source[field_node.byte_range()];
                        
                        if method_name == "is_empty" {
                            // Check if this is being called on a constant-sized collection
                            if let Some(object_node) = function_node.child_by_field_name("value") {
                                let object_text = &source[object_node.byte_range()];
                                
                                if is_constant_sized_collection(object_text) {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: "Checking if constant-sized collection is empty is redundant - the size is known at compile time".to_string(),
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

fn is_constant_sized_collection(text: &str) -> bool {
    // Check for array literals
    text.starts_with("[") && text.ends_with("]") && !text.contains("..") ||
    // Check for string literals
    text.starts_with("\"") && text.ends_with("\"") ||
    // Check for specific constant collections
    text == "[]" ||
    text == "\"\"" ||
    text == "()" ||
    // Check for array references with known size
    text.contains("&[") && text.contains("]") ||
    // Check for const identifiers (ALL_CAPS typically indicates constants)
    (text.chars().all(|c| c.is_uppercase() || c == '_') && text.len() > 1)
}