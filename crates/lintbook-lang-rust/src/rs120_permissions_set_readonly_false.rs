use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct PermissionsSetReadonlyFalse;

impl Rule for PermissionsSetReadonlyFalse {
    fn id(&self) -> &'static str {
        "RS120"
    }

    fn name(&self) -> &'static str {
        "permissions-set-readonly-false"
    }

    fn description(&self) -> &'static str {
        "Checks for set_readonly(false) calls"
    }

    fn explanation(&self) -> &'static str {
        "Calling set_readonly(false) is redundant since files are writable by default. \
         This call has no effect unless the file was previously set to readonly."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl PermissionsSetReadonlyFalse {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = source[field_node.byte_range()].trim();

                        if method_name == "set_readonly" {
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                let mut cursor = args_node.walk();
                                for child in args_node.children(&mut cursor) {
                                    let arg_text = source[child.byte_range()].trim();
                                    if arg_text == "false" {
                                        let position = node.start_position();
                                        violations.push(LintViolation {
                                            line: position.row + 1,
                                            column: position.column + 1,
                                            message: "set_readonly(false) is redundant - files are writable by default".to_string(),
                                            lint_name: self.name().to_string(),
                                            lint_id: self.id().to_string(),
                                        });
                                        break;
                                    }
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
