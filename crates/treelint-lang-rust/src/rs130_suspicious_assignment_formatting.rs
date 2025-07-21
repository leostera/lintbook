use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct SuspiciousAssignmentFormatting;

impl Rule for SuspiciousAssignmentFormatting {
    fn id(&self) -> &'static str {
        "RS130"
    }

    fn name(&self) -> &'static str {
        "suspicious-assignment-formatting"
    }

    fn description(&self) -> &'static str {
        "Checks for suspicious assignment formatting"
    }

    fn explanation(&self) -> &'static str {
        "Suspicious assignment formatting like 'a =- b' or 'a =+ b' might be typos. \
         Consider using 'a -= b' or 'a += b' for compound assignment operators."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SuspiciousAssignmentFormatting {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "assignment_expression" {
            let assign_text = source[node.byte_range()].trim();
            
            // Check for suspicious patterns like =- or =+
            if contains_suspicious_assignment_pattern(assign_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Suspicious assignment formatting - did you mean to use compound assignment operator?".to_string(),
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

fn contains_suspicious_assignment_pattern(text: &str) -> bool {
    // Look for patterns like =- or =+ (with space after =)
    text.contains("= -") ||
    text.contains("= +") ||
    text.contains("=\t-") ||
    text.contains("=\t+")
}