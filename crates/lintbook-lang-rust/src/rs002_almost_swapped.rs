use crate::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct AlmostSwapped;

impl Rule for AlmostSwapped {
    fn id(&self) -> &'static str {
        "RS002"
    }

    fn name(&self) -> &'static str {
        "almost-swapped"
    }

    fn description(&self) -> &'static str {
        "Detects patterns like `foo = bar; bar = foo` that look like attempted swaps"
    }

    fn explanation(&self) -> &'static str {
        "The pattern `foo = bar; bar = foo` assigns the same value to both variables \
         instead of swapping them. Use `std::mem::swap(&mut foo, &mut bar)` or \
         tuple assignment `(foo, bar) = (bar, foo)` for proper swapping."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl AlmostSwapped {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for consecutive assignment expressions
        if node.kind() == "expression_statement" {
            if let Some(first_assignment) = node.child(0) {
                if first_assignment.kind() == "assignment_expression" {
                    // Check if there's a next sibling that's also an assignment
                    if let Some(next_stmt) = node.next_sibling() {
                        if next_stmt.kind() == "expression_statement" {
                            if let Some(second_assignment) = next_stmt.child(0) {
                                if second_assignment.kind() == "assignment_expression" {
                                    self.check_swap_pattern(
                                        first_assignment,
                                        second_assignment,
                                        source,
                                        violations,
                                    );
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

    fn check_swap_pattern(
        &self,
        first: Node,
        second: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Extract left and right sides of both assignments
        if let (Some(first_left), Some(first_right)) = (
            first.child_by_field_name("left"),
            first.child_by_field_name("right"),
        ) {
            if let (Some(second_left), Some(second_right)) = (
                second.child_by_field_name("left"),
                second.child_by_field_name("right"),
            ) {
                let first_left_text = &source[first_left.byte_range()];
                let first_right_text = &source[first_right.byte_range()];
                let second_left_text = &source[second_left.byte_range()];
                let second_right_text = &source[second_right.byte_range()];

                // Check for almost-swap pattern: a = b; b = a
                if first_left_text == second_right_text && first_right_text == second_left_text {
                    let position = first.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: format!(
                            "This looks like you are trying to swap `{}` and `{}`. Use `std::mem::swap` or tuple assignment instead",
                            first_left_text, first_right_text
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}
