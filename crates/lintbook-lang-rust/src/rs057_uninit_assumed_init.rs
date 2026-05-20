use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct UninitAssumedInit;

impl Rule for UninitAssumedInit {
    fn id(&self) -> &'static str {
        "RS057"
    }

    fn name(&self) -> &'static str {
        "uninit-assumed-init"
    }

    fn description(&self) -> &'static str {
        "Checks for MaybeUninit::uninit().assume_init() pattern"
    }

    fn explanation(&self) -> &'static str {
        "Calling assume_init() on uninitialized memory is undefined behavior. \
         Initialize the memory first or use MaybeUninit::zeroed() for zero-initialization."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl UninitAssumedInit {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for method call expressions
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    let call_text = &source[node.byte_range()];

                    // Check for the dangerous pattern: MaybeUninit::uninit().assume_init()
                    if is_uninit_assume_init_pattern(call_text) {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: "Calling assume_init() on uninitialized MaybeUninit is undefined behavior. Initialize the value first".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_uninit_assume_init_pattern(text: &str) -> bool {
    // Look for patterns like:
    // MaybeUninit::uninit().assume_init()
    // MaybeUninit::<T>::uninit().assume_init()
    // std::mem::MaybeUninit::uninit().assume_init()

    text.contains("uninit()")
        && text.contains("assume_init()")
        && (text.contains("MaybeUninit::uninit().assume_init()")
            || text.contains("MaybeUninit::<") && text.contains(">::uninit().assume_init()")
            || text.contains("mem::MaybeUninit::uninit().assume_init()")
            || text.contains("std::mem::MaybeUninit::uninit().assume_init()"))
}
