use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct CrateInMacroDef;

impl Rule for CrateInMacroDef {
    fn id(&self) -> &'static str {
        "RS082"
    }

    fn name(&self) -> &'static str {
        "crate-in-macro-def"
    }

    fn description(&self) -> &'static str {
        "Checks for usage of 'crate' in macro definitions"
    }

    fn explanation(&self) -> &'static str {
        "Using 'crate' in macro definitions can be confusing as it refers to the crate where \
         the macro is defined, not where it's used. Consider using $crate instead."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl CrateInMacroDef {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "macro_definition" {
            self.check_macro_body(node, source, violations);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn check_macro_body(&self, macro_node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut cursor = macro_node.walk();
        for child in macro_node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let text = &source[child.byte_range()];
                if text == "crate" {
                    let position = child.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "Use '$crate' instead of 'crate' in macro definitions for correct hygiene".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
            self.check_macro_body(child, source, violations);
        }
    }
}