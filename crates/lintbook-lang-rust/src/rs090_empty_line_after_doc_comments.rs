use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct EmptyLineAfterDocComments;

impl Rule for EmptyLineAfterDocComments {
    fn id(&self) -> &'static str {
        "RS090"
    }

    fn name(&self) -> &'static str {
        "empty-line-after-doc-comments"
    }

    fn description(&self) -> &'static str {
        "Checks for missing empty lines after documentation comments"
    }

    fn explanation(&self) -> &'static str {
        "Consider adding an empty line after documentation comments for better readability."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl EmptyLineAfterDocComments {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "line_comment" {
            let comment_text = &source[node.byte_range()];

            if is_doc_comment(comment_text) {
                if let Some(next_sibling) = node.next_sibling() {
                    // Check if there's an empty line between doc comment and next item
                    let comment_end = node.end_position();
                    let next_start = next_sibling.start_position();

                    if next_start.row == comment_end.row + 1
                        && !matches!(next_sibling.kind(), "line_comment")
                    {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: "Consider adding an empty line after documentation comments"
                                .to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_doc_comment(comment: &str) -> bool {
    comment.starts_with("///") || comment.starts_with("//!")
}
