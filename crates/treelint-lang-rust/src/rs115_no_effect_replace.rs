use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct NoEffectReplace;

impl Rule for NoEffectReplace {
    fn id(&self) -> &'static str {
        "RS115"
    }

    fn name(&self) -> &'static str {
        "no-effect-replace"
    }

    fn description(&self) -> &'static str {
        "Checks for replace operations with no effect"
    }

    fn explanation(&self) -> &'static str {
        "Replace operations where the pattern and replacement are identical have no effect. \
         Remove such calls or check if this is intentional."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl NoEffectReplace {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "call_expression" {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "field_expression" {
                    if let Some(field_node) = function_node.child_by_field_name("field") {
                        let method_name = source[field_node.byte_range()].trim();
                        
                        if method_name == "replace" {
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                let args = collect_arguments(args_node, source);
                                if args.len() >= 2 && args[0] == args[1] {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: format!(
                                            "Replace operation has no effect: replacing '{}' with '{}'",
                                            args[0], args[1]
                                        ),
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

fn collect_arguments(args_node: Node, source: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cursor = args_node.walk();
    
    for child in args_node.children(&mut cursor) {
        if child.kind() != "," {
            let arg_text = source[child.byte_range()].trim();
            args.push(arg_text.to_string());
        }
    }
    
    args
}