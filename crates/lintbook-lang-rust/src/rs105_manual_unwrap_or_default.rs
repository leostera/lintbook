use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct ManualUnwrapOrDefault;

impl Rule for ManualUnwrapOrDefault {
    fn id(&self) -> &'static str {
        "RS105"
    }

    fn name(&self) -> &'static str {
        "manual-unwrap-or-default"
    }

    fn description(&self) -> &'static str {
        "Checks for manual implementations of unwrap_or_default"
    }

    fn explanation(&self) -> &'static str {
        "Use .unwrap_or_default() instead of manually implementing the same logic."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl ManualUnwrapOrDefault {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = &source[field_node.byte_range()];

                        if method_name == "unwrap_or" || method_name == "unwrap_or_else" {
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                let args_text = &source[args_node.byte_range()];
                                if is_default_expression(args_text) {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: "Use .unwrap_or_default() instead of manual default implementation".to_string(),
                                        lint_name: self.name().to_string(),
                                        lint_id: self.id().to_string(),
                                    });
                                }
                            }
                        }
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

fn is_default_expression(args: &str) -> bool {
    // Remove parentheses and whitespace
    let cleaned = args
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();

    // Check for various default patterns
    cleaned == "Default::default()"
        || cleaned.ends_with("::default()")
        || cleaned == "Vec::new()"
        || cleaned == "String::new()"
        || cleaned == "HashMap::new()"
        || cleaned == "HashSet::new()"
        || cleaned == "0"
        || cleaned == "false"
        || cleaned == "None"
        || cleaned == "|| Default::default()"
        || cleaned == "|| Vec::new()"
        || cleaned == "|| String::new()"
}
