use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct FromRawWithVoidPtr;

impl Rule for FromRawWithVoidPtr {
    fn id(&self) -> &'static str {
        "RS096"
    }

    fn name(&self) -> &'static str {
        "from-raw-with-void-ptr"
    }

    fn description(&self) -> &'static str {
        "Checks for from_raw calls with void pointers"
    }

    fn explanation(&self) -> &'static str {
        "Creating types from void pointers (*const c_void) can be dangerous. \
         Ensure the pointer is properly aligned and points to valid data."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl FromRawWithVoidPtr {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                let function_text = &source[function_node.byte_range()];
                
                if function_text.contains("from_raw") {
                    let call_text = &source[node.byte_range()];
                    if contains_void_ptr(call_text) {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: "Creating types from void pointers is dangerous - ensure proper alignment and validity".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
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

fn contains_void_ptr(text: &str) -> bool {
    text.contains("*const c_void") || 
    text.contains("*mut c_void") ||
    text.contains("*const std::ffi::c_void") ||
    text.contains("*mut std::ffi::c_void") ||
    text.contains("void_ptr") ||
    text.contains("as *const c_void") ||
    text.contains("as *mut c_void")
}