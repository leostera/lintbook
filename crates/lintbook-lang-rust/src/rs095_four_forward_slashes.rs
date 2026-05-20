use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct FourForwardSlashes;

impl Rule for FourForwardSlashes {
    fn id(&self) -> &'static str {
        "RS095"
    }

    fn name(&self) -> &'static str {
        "four-forward-slashes"
    }

    fn description(&self) -> &'static str {
        "Checks for four or more forward slashes in comments"
    }

    fn explanation(&self) -> &'static str {
        "Using four or more forward slashes (//// or more) in comments is non-standard and \
         may indicate commented-out code or unintentional syntax. Use standard comment styles \
         (// or ///) for documentation."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl FourForwardSlashes {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "line_comment" {
            let comment_text = &source[node.byte_range()];

            // Check if the comment starts with four or more forward slashes
            if comment_text.starts_with("////") {
                let position = node.start_position();

                // Count the number of consecutive forward slashes
                let slash_count = comment_text.chars().take_while(|&c| c == '/').count();

                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: format!(
                        "Found {} forward slashes in comment. Use // for regular comments or /// for documentation",
                        slash_count
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}
