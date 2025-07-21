use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct CastAbsToUnsigned;

impl Rule for CastAbsToUnsigned {
    fn id(&self) -> &'static str {
        "RS075"
    }

    fn name(&self) -> &'static str {
        "cast-abs-to-unsigned"
    }

    fn description(&self) -> &'static str {
        "Checks for casting the result of abs() to unsigned types"
    }

    fn explanation(&self) -> &'static str {
        "Casting the result of abs() to an unsigned type can be dangerous. For the minimum value \
         of signed types (e.g., i32::MIN), abs() will panic in debug mode or wrap in release mode. \
         Consider using unsigned_abs() or checking for the minimum value first."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl CastAbsToUnsigned {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "as_expression" {
            if let (Some(expr_node), Some(type_node)) = (
                node.child_by_field_name("value"),
                node.child_by_field_name("type"),
            ) {
                let type_text = &source[type_node.byte_range()];
                
                // Check if casting to unsigned type
                if is_unsigned_type(type_text) {
                    // Check if the expression is a call to abs()
                    if is_abs_call(expr_node, source) {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Casting abs() result to unsigned type `{}` can panic on minimum values. Consider using unsigned_abs()",
                                type_text
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
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

fn is_unsigned_type(type_text: &str) -> bool {
    matches!(type_text.trim(), 
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
        "std::u8" | "std::u16" | "std::u32" | "std::u64" | "std::u128" | "std::usize"
    )
}

fn is_abs_call(node: Node, source: &str) -> bool {
    if node.kind() == "call_expression" {
        if let Some(function_node) = node.child_by_field_name("function") {
            // Check for method call like x.abs()
            if function_node.kind() == "field_expression" {
                if let Some(field_node) = function_node.child_by_field_name("field") {
                    let method_name = &source[field_node.byte_range()];
                    return method_name == "abs";
                }
            }
            // Check for function call like abs(x) or i32::abs(x)
            else {
                let function_text = &source[function_node.byte_range()];
                return function_text == "abs" || 
                       function_text.ends_with("::abs") ||
                       function_text.contains("abs");
            }
        }
    }
    
    false
}