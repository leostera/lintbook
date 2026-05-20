use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct SuspiciousDocComments;

impl Rule for SuspiciousDocComments {
    fn id(&self) -> &'static str {
        "RS132"
    }

    fn name(&self) -> &'static str {
        "suspicious-doc-comments"
    }

    fn description(&self) -> &'static str {
        "Checks for suspicious documentation patterns"
    }

    fn explanation(&self) -> &'static str {
        "Suspicious documentation patterns like ///< or /** with incorrect formatting \
         might not generate proper documentation. Use /// or /** */ correctly."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SuspiciousDocComments {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "line_comment" || node.kind() == "block_comment" {
            let comment_text = source[node.byte_range()].trim();

            if is_suspicious_doc_comment(comment_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Suspicious documentation comment formatting - check syntax"
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

fn is_suspicious_doc_comment(comment_text: &str) -> bool {
    // Look for suspicious doc comment patterns
    comment_text.starts_with("///<")
        || comment_text.starts_with("/**<")
        || (comment_text.starts_with("/**") && !comment_text.ends_with("*/"))
        || comment_text.starts_with("////")
        || comment_text.starts_with("//!<")
}
