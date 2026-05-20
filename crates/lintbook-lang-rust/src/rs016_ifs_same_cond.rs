use crate::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct IfsSameCond;

impl Rule for IfsSameCond {
    fn id(&self) -> &'static str {
        "RS016"
    }

    fn name(&self) -> &'static str {
        "ifs-same-cond"
    }

    fn description(&self) -> &'static str {
        "Detects consecutive if statements with the same condition"
    }

    fn explanation(&self) -> &'static str {
        "Consecutive if statements with identical conditions are redundant. The second condition \
         will never be true if the first one wasn't. Consider using `else if` or combining the conditions."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl IfsSameCond {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for consecutive if expressions
        if node.kind() == "if_expression" {
            if let Some(next_sibling) = node.next_sibling() {
                // Skip whitespace and find the next actual node
                let next_node = self.find_next_non_whitespace_sibling(next_sibling);

                if let Some(next_if) = next_node {
                    if next_if.kind() == "if_expression" {
                        self.check_same_conditions(node, next_if, source, violations);
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

    fn find_next_non_whitespace_sibling<'a>(&self, mut node: Node<'a>) -> Option<Node<'a>> {
        loop {
            // Skip whitespace, comments, and other non-significant nodes
            if matches!(node.kind(), "line_comment" | "block_comment") {
                if let Some(next) = node.next_sibling() {
                    node = next;
                    continue;
                } else {
                    return None;
                }
            }

            // If we find a significant node, return it
            if !node.kind().is_empty() {
                return Some(node);
            }

            // Move to next sibling
            if let Some(next) = node.next_sibling() {
                node = next;
            } else {
                return None;
            }
        }
    }

    fn check_same_conditions(
        &self,
        first_if: Node,
        second_if: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        if let Some(first_condition) = first_if.child_by_field_name("condition") {
            if let Some(second_condition) = second_if.child_by_field_name("condition") {
                let first_text = &source[first_condition.byte_range()];
                let second_text = &source[second_condition.byte_range()];

                // Compare the condition text (normalize whitespace)
                if normalize_condition(first_text) == normalize_condition(second_text) {
                    let position = second_if.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: format!(
                            "This `if` has the same condition as the previous `if`: `{}`. Consider using `else if` or combining conditions",
                            first_text.trim()
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}

fn normalize_condition(condition: &str) -> String {
    // Remove extra whitespace and normalize for comparison
    condition
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}
