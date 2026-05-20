use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct DeprecatedClippyCfgAttr;

impl Rule for DeprecatedClippyCfgAttr {
    fn id(&self) -> &'static str {
        "RS083"
    }

    fn name(&self) -> &'static str {
        "deprecated-clippy-cfg-attr"
    }

    fn description(&self) -> &'static str {
        "Checks for deprecated clippy cfg attributes"
    }

    fn explanation(&self) -> &'static str {
        "Some clippy cfg attributes are deprecated. Use the newer equivalents."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl DeprecatedClippyCfgAttr {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "attribute_item" {
            let attr_text = &source[node.byte_range()];
            if is_deprecated_clippy_cfg(attr_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "This clippy cfg attribute is deprecated".to_string(),
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

fn is_deprecated_clippy_cfg(attr_text: &str) -> bool {
    attr_text.contains("clippy")
        && (attr_text.contains("feature = \"cargo-clippy\"") || attr_text.contains("cfg(clippy)"))
}
