use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct SuspiciousUnaryOpFormatting;

impl Rule for SuspiciousUnaryOpFormatting {
    fn id(&self) -> &'static str {
        "RS138"
    }

    fn name(&self) -> &'static str {
        "suspicious-unary-op-formatting"
    }

    fn description(&self) -> &'static str {
        "Checks for suspicious unary operator formatting"
    }

    fn explanation(&self) -> &'static str {
        "Suspicious unary operator formatting with unusual spacing might be confusing. \
         Consider consistent spacing around unary operators like ! or -."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SuspiciousUnaryOpFormatting {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "unary_expression" {
            let unary_text = source[node.byte_range()].trim();
            
            if has_suspicious_unary_formatting(unary_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Suspicious unary operator formatting - check spacing".to_string(),
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

fn has_suspicious_unary_formatting(unary_text: &str) -> bool {
    // Look for suspicious unary operator formatting
    unary_text.contains("! ") ||          // space after !
    unary_text.contains("- ") ||          // space after -
    unary_text.contains("+ ") ||          // space after +
    unary_text.contains("!\t") ||         // tab after !
    unary_text.contains("-\t") ||         // tab after -
    unary_text.contains("+\t")            // tab after +
}