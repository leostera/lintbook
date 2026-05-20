use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct InlineFnWithoutBody;

impl Rule for InlineFnWithoutBody {
    fn id(&self) -> &'static str {
        "RS022"
    }

    fn name(&self) -> &'static str {
        "inline-fn-without-body"
    }

    fn description(&self) -> &'static str {
        "Detects inline attributes on trait methods or functions without bodies"
    }

    fn explanation(&self) -> &'static str {
        "The `#[inline]` attribute has no effect on trait method declarations without bodies \
         or extern function declarations. It only affects functions with implementations."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl InlineFnWithoutBody {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for function declarations with inline attributes
        if node.kind() == "function_item" || node.kind() == "function_signature_item" {
            self.check_function_with_inline(node, source, violations);
        }

        // Also check trait method declarations
        if node.kind() == "trait_item" {
            self.check_trait_methods(node, source, violations);
        }

        // Check extern function declarations
        if node.kind() == "foreign_mod_item" {
            self.check_extern_functions(node, source, violations);
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn check_function_with_inline(
        &self,
        function_node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check if function has inline attribute and no body
        if self.has_inline_attribute(function_node, source) {
            if !self.has_function_body(function_node) {
                let position = function_node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "The `#[inline]` attribute has no effect on functions without bodies"
                        .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_trait_methods(
        &self,
        trait_node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut cursor = trait_node.walk();
        for child in trait_node.children(&mut cursor) {
            if child.kind() == "function_signature_item" {
                if self.has_inline_attribute(child, source) && !self.has_function_body(child) {
                    let position = child.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "The `#[inline]` attribute has no effect on trait method declarations without default implementations".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn check_extern_functions(
        &self,
        extern_node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut cursor = extern_node.walk();
        for child in extern_node.children(&mut cursor) {
            if child.kind() == "function_signature_item" {
                if self.has_inline_attribute(child, source) {
                    let position = child.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "The `#[inline]` attribute has no effect on extern function declarations".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn has_inline_attribute(&self, node: Node, source: &str) -> bool {
        // Look for attribute lists before the function
        let mut current = node;

        // Check if there's an attribute list as a previous sibling or child
        if let Some(attr_list) = node.child_by_field_name("attributes") {
            return self.contains_inline_attribute(attr_list, source);
        }

        // Also check previous siblings for attributes
        while let Some(prev) = current.prev_sibling() {
            if prev.kind() == "attribute_item" {
                let attr_text = &source[prev.byte_range()];
                if attr_text.contains("#[inline") {
                    return true;
                }
            } else if !matches!(prev.kind(), "line_comment" | "block_comment") {
                // Stop looking if we hit a non-comment, non-attribute node
                break;
            }
            current = prev;
        }

        false
    }

    fn contains_inline_attribute(&self, attr_list: Node, source: &str) -> bool {
        let mut cursor = attr_list.walk();
        for child in attr_list.children(&mut cursor) {
            if child.kind() == "attribute_item" {
                let attr_text = &source[child.byte_range()];
                if attr_text.contains("#[inline") {
                    return true;
                }
            }
        }
        false
    }

    fn has_function_body(&self, function_node: Node) -> bool {
        // Check if the function has a body (block)
        function_node.child_by_field_name("body").is_some() ||
        // Also check for any block child
        {
            let mut cursor = function_node.walk();
            let has_block = function_node.children(&mut cursor).any(|child| child.kind() == "block");
            has_block
        }
    }
}
