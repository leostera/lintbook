use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct EmptyLineAfterOuterAttr;

impl Rule for EmptyLineAfterOuterAttr {
    fn id(&self) -> &'static str {
        "RS091"
    }

    fn name(&self) -> &'static str {
        "empty-line-after-outer-attr"
    }

    fn description(&self) -> &'static str {
        "Checks for missing empty lines after outer attributes"
    }

    fn explanation(&self) -> &'static str {
        "Consider adding an empty line after outer attributes for better readability."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl EmptyLineAfterOuterAttr {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "attribute_item" {
            let attr_text = &source[node.byte_range()];
            
            // Check if this is an outer attribute (starts with #[)
            if attr_text.starts_with("#[") {
                if let Some(next_sibling) = node.next_sibling() {
                    let attr_end = node.end_position();
                    let next_start = next_sibling.start_position();
                    
                    // Check if there's no empty line and next item is not another attribute
                    if next_start.row == attr_end.row + 1 && 
                       !matches!(next_sibling.kind(), "attribute_item" | "line_comment") {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: "Consider adding an empty line after outer attributes".to_string(),
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