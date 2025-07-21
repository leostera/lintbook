use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct EmptyDocs;

impl Rule for EmptyDocs {
    fn id(&self) -> &'static str {
        "RS089"
    }

    fn name(&self) -> &'static str {
        "empty-docs"
    }

    fn description(&self) -> &'static str {
        "Checks for empty documentation comments"
    }

    fn explanation(&self) -> &'static str {
        "Empty documentation comments (/// or //!) provide no value and should be removed \
         or filled with meaningful content."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl EmptyDocs {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "line_comment" {
            let comment_text = &source[node.byte_range()];
            
            if is_empty_doc_comment(comment_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Empty documentation comment should be removed or filled with content".to_string(),
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

fn is_empty_doc_comment(comment: &str) -> bool {
    // Check for empty /// comments
    if comment.starts_with("///") {
        let content = comment.trim_start_matches("///").trim();
        return content.is_empty();
    }
    
    // Check for empty //! comments
    if comment.starts_with("//!") {
        let content = comment.trim_start_matches("//!").trim();
        return content.is_empty();
    }
    
    false
}