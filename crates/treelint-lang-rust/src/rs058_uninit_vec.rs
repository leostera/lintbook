use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct UninitVec;

impl Rule for UninitVec {
    fn id(&self) -> &'static str {
        "RS058"
    }

    fn name(&self) -> &'static str {
        "uninit-vec"
    }

    fn description(&self) -> &'static str {
        "Checks for Vec with uninitialized memory"
    }

    fn explanation(&self) -> &'static str {
        "Creating a Vec with uninitialized memory can lead to undefined behavior. \
         Use vec![T::default(); n] or Vec::with_capacity() followed by proper initialization."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl UninitVec {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                let call_text = &source[node.byte_range()];
                
                // Check for dangerous patterns involving Vec and uninitialized memory
                if is_uninit_vec_pattern(call_text) {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "Creating Vec with uninitialized memory is undefined behavior. Use vec![T::default(); n] or Vec::with_capacity() with proper initialization".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Also check for Vec::from_raw_parts with potentially uninitialized data
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                let function_text = &source[function_node.byte_range()];
                if function_text.contains("from_raw_parts") && function_text.contains("Vec") {
                    let call_text = &source[node.byte_range()];
                    if call_text.contains("uninit") || call_text.contains("uninitialized") {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: "Vec::from_raw_parts with uninitialized data is undefined behavior".to_string(),
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

fn is_uninit_vec_pattern(text: &str) -> bool {
    // Look for patterns like:
    // Vec::from_raw_parts(...uninit...)
    // vec![MaybeUninit::uninit(); n]
    // Vec with uninitialized allocation
    
    (text.contains("Vec") && text.contains("uninit")) ||
    (text.contains("vec!") && text.contains("uninit")) ||
    (text.contains("Vec::from_raw_parts") && text.contains("uninit")) ||
    // Check for Vec created with uninitialized memory allocation
    (text.contains("Vec::with_capacity") && text.contains("set_len") && 
     !text.contains("write") && !text.contains("init"))
}