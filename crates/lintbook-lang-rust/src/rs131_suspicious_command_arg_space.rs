use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct SuspiciousCommandArgSpace;

impl Rule for SuspiciousCommandArgSpace {
    fn id(&self) -> &'static str {
        "RS131"
    }

    fn name(&self) -> &'static str {
        "suspicious-command-arg-space"
    }

    fn description(&self) -> &'static str {
        "Checks for spaces in command arguments"
    }

    fn explanation(&self) -> &'static str {
        "Command arguments with spaces might be incorrectly split by the shell. \
         Consider using proper escaping or separate arguments."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SuspiciousCommandArgSpace {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            let call_text = source[node.byte_range()].trim();

            // Check for Command::new or similar command creation with suspicious arguments
            if is_command_with_suspicious_args(call_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Command argument contains spaces - consider proper escaping or separate arguments".to_string(),
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

fn is_command_with_suspicious_args(call_text: &str) -> bool {
    // Look for Command::new or .arg() calls with strings containing spaces
    if call_text.contains("Command::") || call_text.contains(".arg(") {
        // Check for quoted strings with spaces that might be problematic
        return call_text.contains("\" ")
            || call_text.contains(" \"")
            || (call_text.contains("\"") && call_text.matches(' ').count() > 2);
    }
    false
}
