use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct MisrefactoredAssignOp;

impl Rule for MisrefactoredAssignOp {
    fn id(&self) -> &'static str {
        "RS107"
    }

    fn name(&self) -> &'static str {
        "misrefactored-assign-op"
    }

    fn description(&self) -> &'static str {
        "Checks for incorrectly refactored assignment operations"
    }

    fn explanation(&self) -> &'static str {
        "Assignment operations like `a += a + b` should be `a += b`. \
         The variable appears unnecessarily on both sides."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl MisrefactoredAssignOp {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "compound_assignment_expr" {
            if let (Some(left), Some(right)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) {
                let left_text = source[left.byte_range()].trim();

                // Check if right side is a binary operation containing the left variable
                if right.kind() == "binary_expression" {
                    if let Some(right_left) = right.child_by_field_name("left") {
                        let right_left_text = source[right_left.byte_range()].trim();
                        if left_text == right_left_text {
                            let position = node.start_position();
                            violations.push(LintViolation {
                                line: position.row + 1,
                                column: position.column + 1,
                                message: format!(
                                    "Misrefactored assignment: `{} op= {} + ...` should be `{} op= ...`",
                                    left_text, left_text, left_text
                                ),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
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
