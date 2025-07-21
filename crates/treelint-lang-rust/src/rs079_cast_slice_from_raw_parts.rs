use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct CastSliceFromRawParts;

impl Rule for CastSliceFromRawParts {
    fn id(&self) -> &'static str {
        "RS079"
    }

    fn name(&self) -> &'static str {
        "cast-slice-from-raw-parts"
    }

    fn description(&self) -> &'static str {
        "Checks for potentially unsafe slice::from_raw_parts casts"
    }

    fn explanation(&self) -> &'static str {
        "Creating slices from raw parts with incorrect alignment or size can cause undefined behavior. \
         Ensure pointer alignment and size calculations are correct."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl CastSliceFromRawParts {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                let function_text = &source[function_node.byte_range()];
                
                if is_from_raw_parts_call(function_text) {
                    // Check for potentially dangerous patterns
                    if let Some(args_node) = node.child_by_field_name("arguments") {
                        let call_text = &source[node.byte_range()];
                        
                        // Look for casts that might be problematic
                        if contains_suspicious_cast(call_text) {
                            let position = node.start_position();
                            violations.push(LintViolation {
                                line: position.row + 1,
                                column: position.column + 1,
                                message: "Potentially unsafe slice::from_raw_parts - ensure pointer alignment and size are correct".to_string(),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
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

fn is_from_raw_parts_call(function_text: &str) -> bool {
    function_text.contains("from_raw_parts") && (
        function_text.contains("slice::from_raw_parts") ||
        function_text.contains("std::slice::from_raw_parts") ||
        function_text.ends_with("::from_raw_parts")
    )
}

fn contains_suspicious_cast(call_text: &str) -> bool {
    // Look for casts that might be problematic
    call_text.contains(" as *") || 
    call_text.contains("transmute") ||
    call_text.contains("cast()") ||
    call_text.contains("size_of") ||
    call_text.contains("align_of") ||
    // Check for hardcoded numbers that might be sizes
    (call_text.contains("from_raw_parts") && 
     (call_text.contains(", 1)") || 
      call_text.contains(", 2)") || 
      call_text.contains(", 4)") || 
      call_text.contains(", 8)")))
}