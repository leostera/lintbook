use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct RepeatVecWithCapacity;

impl Rule for RepeatVecWithCapacity {
    fn id(&self) -> &'static str {
        "RS125"
    }

    fn name(&self) -> &'static str {
        "repeat-vec-with-capacity"
    }

    fn description(&self) -> &'static str {
        "Checks for Vec creation with capacity that uses repeat"
    }

    fn explanation(&self) -> &'static str {
        "Using vec![x; n] with Vec::with_capacity() creates unnecessary allocations. \
         Consider using Vec::new() or direct initialization instead."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl RepeatVecWithCapacity {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            let call_text = source[node.byte_range()].trim();

            // Check for Vec::with_capacity followed by vec! macro with repeat
            if call_text.contains("Vec::with_capacity")
                && call_text.contains("vec!")
                && call_text.contains(";")
            {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message:
                        "Using vec![x; n] with Vec::with_capacity creates unnecessary allocations"
                            .to_string(),
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
