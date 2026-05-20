use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct SwapPtrToRef;

impl Rule for SwapPtrToRef {
    fn id(&self) -> &'static str {
        "RS139"
    }

    fn name(&self) -> &'static str {
        "swap-ptr-to-ref"
    }

    fn description(&self) -> &'static str {
        "Checks for pointer to reference swaps"
    }

    fn explanation(&self) -> &'static str {
        "Swapping between pointers and references might indicate confusion about ownership. \
         Consider using consistent pointer or reference types throughout."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SwapPtrToRef {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            let call_text = source[node.byte_range()].trim();

            // Check for ptr::swap or mem::swap with mixed pointer/reference types
            if (call_text.contains("ptr::swap") || call_text.contains("mem::swap"))
                && has_mixed_ptr_ref_types(call_text)
            {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Swapping between pointer and reference types - consider using consistent types".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn has_mixed_ptr_ref_types(call_text: &str) -> bool {
    // Look for patterns indicating mixed pointer/reference usage
    let has_ptr =
        call_text.contains("*mut") || call_text.contains("*const") || call_text.contains("as_ptr");
    let has_ref =
        call_text.contains("&mut") || call_text.contains("&") && !call_text.contains("&mut");

    has_ptr && has_ref
}
