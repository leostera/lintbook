use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct ReversedEmptyRanges;

impl Rule for ReversedEmptyRanges {
    fn id(&self) -> &'static str {
        "RS050"
    }

    fn name(&self) -> &'static str {
        "reversed-empty-ranges"
    }

    fn description(&self) -> &'static str {
        "Checks for reversed range literals that result in empty ranges"
    }

    fn explanation(&self) -> &'static str {
        "Ranges where the start is greater than the end result in empty ranges. \
         This is usually a mistake and can lead to unexpected behavior in loops."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl ReversedEmptyRanges {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "range_expression" {
            if let (Some(left), Some(right)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) {
                let left_text = &source[left.byte_range()].trim();
                let right_text = &source[right.byte_range()].trim();

                // Try to parse both sides as integers for comparison
                if let (Ok(left_val), Ok(right_val)) =
                    (left_text.parse::<i64>(), right_text.parse::<i64>())
                {
                    if left_val > right_val {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Reversed range `{}..{}` will be empty (start > end)",
                                left_text, right_text
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        } else if node.kind() == "range_inclusive_expression" {
            if let (Some(left), Some(right)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) {
                let left_text = &source[left.byte_range()].trim();
                let right_text = &source[right.byte_range()].trim();

                // Try to parse both sides as integers for comparison
                if let (Ok(left_val), Ok(right_val)) =
                    (left_text.parse::<i64>(), right_text.parse::<i64>())
                {
                    if left_val > right_val {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Reversed inclusive range `{}..={}` will be empty (start > end)",
                                left_text, right_text
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
