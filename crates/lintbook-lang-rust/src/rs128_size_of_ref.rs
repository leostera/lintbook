use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct SizeOfRef;

impl Rule for SizeOfRef {
    fn id(&self) -> &'static str {
        "RS128"
    }

    fn name(&self) -> &'static str {
        "size-of-ref"
    }

    fn description(&self) -> &'static str {
        "Checks for size_of on references"
    }

    fn explanation(&self) -> &'static str {
        "Using size_of on a reference returns the size of the pointer, not the referenced value. \
         Use size_of_val() or dereference the value first."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SizeOfRef {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            let call_text = source[node.byte_range()].trim();

            if call_text.contains("size_of") && !call_text.contains("size_of_val") {
                // Check if the argument is a reference
                if is_size_of_reference(call_text) {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "size_of on reference returns pointer size, not value size - use size_of_val() instead".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_size_of_reference(call_text: &str) -> bool {
    // Look for patterns like size_of::<&T> or size_of(ref_var)
    call_text.contains("size_of::<&")
        || call_text.contains("size_of(&")
        || (call_text.contains("size_of(") && call_text.contains("&"))
}
