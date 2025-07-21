use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct SuspiciousElseFormatting;

impl Rule for SuspiciousElseFormatting {
    fn id(&self) -> &'static str {
        "RS133"
    }

    fn name(&self) -> &'static str {
        "suspicious-else-formatting"
    }

    fn description(&self) -> &'static str {
        "Checks for suspicious else formatting"
    }

    fn explanation(&self) -> &'static str {
        "Suspicious else formatting with unusual spacing or placement might indicate \
         a mistake or make code harder to read. Consider consistent formatting."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SuspiciousElseFormatting {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "if_expression" {
            let if_text = &source[node.byte_range()];
            
            if has_suspicious_else_formatting(if_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Suspicious else formatting - check spacing and placement".to_string(),
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

fn has_suspicious_else_formatting(if_text: &str) -> bool {
    // Look for suspicious else formatting patterns
    if_text.contains("}\nelse") ||         // else on next line after }
    if_text.contains("} else{") ||         // no space before {
    if_text.contains("}else ") ||          // no space before else
    if_text.contains("} \nelse") ||        // extra space before newline
    if_text.contains("}\t\telse")          // excessive tabs
}