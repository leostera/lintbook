use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct UselessAttribute;

impl Rule for UselessAttribute {
    fn id(&self) -> &'static str {
        "RS064"
    }

    fn name(&self) -> &'static str {
        "useless-attribute"
    }

    fn description(&self) -> &'static str {
        "Checks for attributes that have no effect"
    }

    fn explanation(&self) -> &'static str {
        "Some attributes have no effect in certain contexts and should be removed to avoid confusion."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl UselessAttribute {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "attribute_item" {
            if let Some(attr_name) = extract_attribute_name(node, source) {
                // Check what kind of item this attribute is applied to
                if let Some(parent) = node.parent() {
                    if let Some(next_sibling) = node.next_sibling() {
                        match next_sibling.kind() {
                            "trait_item" => {
                                if is_useless_on_trait(&attr_name) {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: format!(
                                            "Attribute `{}` has no effect on trait definitions",
                                            attr_name
                                        ),
                                        lint_name: self.name().to_string(),
                                        lint_id: self.id().to_string(),
                                    });
                                }
                            }
                            "function_item" => {
                                if is_useless_on_function(&attr_name, next_sibling, source) {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: format!(
                                            "Attribute `{}` has no effect on this function",
                                            attr_name
                                        ),
                                        lint_name: self.name().to_string(),
                                        lint_id: self.id().to_string(),
                                    });
                                }
                            }
                            "struct_item" | "enum_item" => {
                                if is_useless_on_type(&attr_name) {
                                    let position = node.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: format!(
                                            "Attribute `{}` has no effect on type definitions",
                                            attr_name
                                        ),
                                        lint_name: self.name().to_string(),
                                        lint_id: self.id().to_string(),
                                    });
                                }
                            }
                            _ => {}
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
}

fn extract_attribute_name(attr_node: Node, source: &str) -> Option<String> {
    let attr_text = &source[attr_node.byte_range()];
    if attr_text.starts_with("#[") && attr_text.ends_with("]") {
        let content = &attr_text[2..attr_text.len()-1];
        if let Some(first_word) = content.split_whitespace().next() {
            return Some(first_word.split('(').next().unwrap_or(first_word).to_string());
        }
    }
    None
}

fn is_useless_on_trait(attr_name: &str) -> bool {
    matches!(attr_name, "inline" | "cold" | "hot")
}

fn is_useless_on_function(attr_name: &str, function_node: Node, source: &str) -> bool {
    // Check if function has no body (trait method)
    let function_text = &source[function_node.byte_range()];
    let has_body = function_text.contains("{") && function_text.contains("}");
    
    if !has_body && attr_name == "inline" {
        return true; // inline on trait methods has no effect
    }
    
    false
}

fn is_useless_on_type(attr_name: &str) -> bool {
    matches!(attr_name, "inline" | "cold" | "hot")
}