use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct IterSkipZero;

impl Rule for IterSkipZero {
    fn id(&self) -> &'static str {
        "RS027"
    }

    fn name(&self) -> &'static str {
        "iter-skip-zero"
    }

    fn description(&self) -> &'static str {
        "Detects calls to `.skip(0)` on iterators"
    }

    fn explanation(&self) -> &'static str {
        "Calling `.skip(0)` on an iterator is redundant as it doesn't skip any elements. \
         This is likely a mistake or leftover code that should be removed."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl IterSkipZero {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for method calls
        if node.kind() == "call_expression" {
            if let Some(function) = node.child_by_field_name("function") {
                if function.kind() == "field_expression" {
                    if let Some(field) = function.child_by_field_name("field") {
                        let field_text = &source[field.byte_range()];

                        // Check if the method is "skip"
                        if field_text == "skip" {
                            if let Some(args) = node.child_by_field_name("arguments") {
                                if self.has_zero_argument(args, source) {
                                    // Verify this is likely an iterator call
                                    if let Some(object) = function.child_by_field_name("object") {
                                        if self.looks_like_iterator_context(object, source) {
                                            let position = node.start_position();
                                            violations.push(LintViolation {
                                                line: position.row + 1,
                                                column: position.column + 1,
                                                message: "Calling `.skip(0)` is redundant and can be removed".to_string(),
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
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn has_zero_argument(&self, args_node: Node, source: &str) -> bool {
        // Look for arguments that are just "0"
        let mut cursor = args_node.walk();
        for child in args_node.children(&mut cursor) {
            if matches!(child.kind(), "integer_literal" | "literal") {
                let arg_text = &source[child.byte_range()];
                if arg_text.trim() == "0" {
                    return true;
                }
            }
        }
        false
    }

    fn looks_like_iterator_context(&self, node: Node, source: &str) -> bool {
        let text = &source[node.byte_range()];

        // Check for common iterator patterns and method chains
        text.contains(".iter()") ||
        text.contains(".into_iter()") ||
        text.contains(".chars()") ||
        text.contains(".bytes()") ||
        text.contains(".lines()") ||
        text.contains(".split(") ||
        text.contains(".filter(") ||
        text.contains(".map(") ||
        text.contains(".take(") ||
        text.contains(".enumerate()") ||
        text.contains(".chain(") ||
        text.contains(".zip(") ||
        text.contains(".rev()") ||
        text.contains(".collect()") ||
        // Check if the variable/expression name suggests it's an iterator
        text.ends_with("_iter") ||
        text.ends_with("_iterator") ||
        text.contains("iter") ||
        // Check for range expressions
        text.contains("..") ||
        // Check if it's a method call that typically returns an iterator
        node.kind() == "call_expression" ||
        node.kind() == "field_expression" ||
        // Check for common collection types
        text.contains("vec!") ||
        text.contains("Vec::") ||
        text.contains("HashMap") ||
        text.contains("HashSet") ||
        text.contains("BTreeMap") ||
        text.contains("BTreeSet")
    }
}
