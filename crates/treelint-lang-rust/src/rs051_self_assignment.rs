use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct SelfAssignment;

impl Rule for SelfAssignment {
    fn id(&self) -> &'static str {
        "RS051"
    }

    fn name(&self) -> &'static str {
        "self-assignment"
    }

    fn description(&self) -> &'static str {
        "Checks for assignments where the left and right sides are identical"
    }

    fn explanation(&self) -> &'static str {
        "Self-assignments like `x = x` or `a.field = a.field` are redundant and likely indicate \
         a logic error. They have no effect and may suggest a copy-paste mistake."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SelfAssignment {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "assignment_expression" {
            if let (Some(left), Some(right)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) {
                let left_text = &source[left.byte_range()].trim();
                let right_text = &source[right.byte_range()].trim();
                
                // Check if both sides are textually identical
                if left_text == right_text && !left_text.is_empty() {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: format!(
                            "Self-assignment detected: `{} = {}` has no effect",
                            left_text, right_text
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Also check compound assignments like +=, -=, *=, etc.
        if matches!(node.kind(), 
            "compound_assignment_expr" | 
            "augmented_assignment_expression"
        ) {
            if let (Some(left), Some(right)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) {
                let left_text = &source[left.byte_range()].trim();
                let right_text = &source[right.byte_range()].trim();
                
                // For operations like x += x, x *= x, etc., check if operands are the same
                if left_text == right_text && !left_text.is_empty() {
                    // Get the operator
                    if let Some(operator_node) = node.child_by_field_name("operator") {
                        let op_text = &source[operator_node.byte_range()];
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Self-assignment detected: `{} {} {}` - operands are identical",
                                left_text, op_text, right_text
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