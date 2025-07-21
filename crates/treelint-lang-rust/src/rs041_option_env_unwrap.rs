use treelint_core::*;
use tree_sitter::{Node, Tree};

pub struct OptionEnvUnwrap;

impl Rule for OptionEnvUnwrap {
    fn id(&self) -> &'static str {
        "RS041"
    }

    fn name(&self) -> &'static str {
        "option-env-unwrap"
    }

    fn description(&self) -> &'static str {
        "Checks for usage of `option_env!(...).unwrap()` and suggests usage of the `env!` macro"
    }

    fn explanation(&self) -> &'static str {
        "Unwrapping the result of `option_env!` will panic at run-time if the environment variable doesn't exist, \
        whereas `env!` catches it at compile-time. Use `env!()` instead if you want compile-time failure, \
        or handle the Option properly if you want runtime handling."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl OptionEnvUnwrap {
    fn check_node(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for method call expressions (.unwrap())
        if node.kind() == "call_expression" {
            if let Some(function) = node.child_by_field_name("function") {
                // Check if this is a method call (.unwrap())
                if function.kind() == "field_expression" {
                    if let Some(field) = function.child_by_field_name("field") {
                        let field_text = &source[field.byte_range()];
                        
                        if field_text == "unwrap" {
                            // Check if the object is option_env! macro call
                            if let Some(object) = function.child_by_field_name("object") {
                                if self.is_option_env_macro(object, source) {
                                    let position = node.start_position();
                                    let env_var = self.extract_env_var(object, source);
                                    
                                    let suggestion = if let Some(var) = env_var {
                                        format!("Use `env!(\"{}\")` instead for compile-time failure", var)
                                    } else {
                                        "Use `env!()` instead for compile-time failure, or handle the Option properly".to_string()
                                    };
                                    
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: format!("Using `option_env!().unwrap()` can panic at runtime. {}", suggestion),
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

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn is_option_env_macro(&self, node: Node, source: &str) -> bool {
        if node.kind() == "macro_invocation" {
            if let Some(macro_name) = node.child_by_field_name("macro") {
                let name_text = &source[macro_name.byte_range()];
                return name_text == "option_env";
            }
        }
        false
    }

    fn extract_env_var(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() == "macro_invocation" {
            // Look for the token_tree containing the arguments
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "token_tree" {
                    // Look for string literals in the token tree
                    let mut inner_cursor = child.walk();
                    for inner_child in child.children(&mut inner_cursor) {
                        if inner_child.kind() == "string_literal" {
                            let literal_text = &source[inner_child.byte_range()];
                            // Remove quotes and return the variable name
                            if literal_text.len() >= 2 {
                                return Some(literal_text[1..literal_text.len()-1].to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }
}