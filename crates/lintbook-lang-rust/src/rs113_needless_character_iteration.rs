use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct NeedlessCharacterIteration;

impl Rule for NeedlessCharacterIteration {
    fn id(&self) -> &'static str {
        "RS113"
    }

    fn name(&self) -> &'static str {
        "needless-character-iteration"
    }

    fn description(&self) -> &'static str {
        "Checks for unnecessary character iteration"
    }

    fn explanation(&self) -> &'static str {
        "Using .chars().nth(0) instead of .chars().next(), or similar inefficient patterns. \
         Use more direct methods when available."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl NeedlessCharacterIteration {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            let call_text = source[node.byte_range()].trim();

            // Check for inefficient character access patterns
            if call_text.contains(".chars().nth(0)") {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Use .chars().next() instead of .chars().nth(0)".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            if call_text.contains(".chars().last()") && call_text.contains(".collect::<String>()") {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Needless character iteration - consider more direct string methods"
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
