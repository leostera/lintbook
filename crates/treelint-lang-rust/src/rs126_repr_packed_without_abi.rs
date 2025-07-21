use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct ReprPackedWithoutAbi;

impl Rule for ReprPackedWithoutAbi {
    fn id(&self) -> &'static str {
        "RS126"
    }

    fn name(&self) -> &'static str {
        "repr-packed-without-abi"
    }

    fn description(&self) -> &'static str {
        "Checks for repr(packed) without explicit ABI specification"
    }

    fn explanation(&self) -> &'static str {
        "Using repr(packed) without specifying an ABI can lead to unexpected behavior. \
         Consider using repr(C, packed) or repr(transparent) for clarity."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl ReprPackedWithoutAbi {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "attribute_item" {
            let attr_text = source[node.byte_range()].trim();
            
            if attr_text.starts_with("#[repr(") && 
               attr_text.contains("packed") &&
               !attr_text.contains("C,") &&
               !attr_text.contains("transparent") {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "repr(packed) without explicit ABI specification - consider using repr(C, packed)".to_string(),
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