use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct IterNextLoop;

impl Rule for IterNextLoop {
    fn id(&self) -> &'static str {
        "RS026"
    }

    fn name(&self) -> &'static str {
        "iter-next-loop"
    }

    fn description(&self) -> &'static str {
        "Detects for loops iterating over iterator.next() calls"
    }

    fn explanation(&self) -> &'static str {
        "Using `for item in iterator.next()` is incorrect and will only iterate over a single \
         `Option<T>` value. Use `for item in iterator` or `while let Some(item) = iterator.next()` instead."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl IterNextLoop {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for for loops
        if node.kind() == "for_expression" {
            if let Some(iterable) = node.child_by_field_name("value") {
                self.check_for_loop_iterable(iterable, source, violations);
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn check_for_loop_iterable(
        &self,
        iterable: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check if the iterable is a method call
        if iterable.kind() == "call_expression" {
            if let Some(function) = iterable.child_by_field_name("function") {
                // Check if it's a field access (method call)
                if function.kind() == "field_expression" {
                    if let Some(field) = function.child_by_field_name("field") {
                        let field_text = &source[field.byte_range()];

                        // Check if the method is "next"
                        if field_text == "next" {
                            // Check if there are no arguments (just calling .next())
                            if let Some(args) = iterable.child_by_field_name("arguments") {
                                if self.is_empty_argument_list(args, source) {
                                    let position = iterable.start_position();
                                    violations.push(LintViolation {
                                        line: position.row + 1,
                                        column: position.column + 1,
                                        message: "For loop over `iterator.next()` only iterates once. Use `for item in iterator` or `while let Some(item) = iterator.next()` instead".to_string(),
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

        // Also check for chained next calls like `iter.skip(1).next()`
        self.check_chained_next_call(iterable, source, violations);
    }

    fn check_chained_next_call(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        if node.kind() == "call_expression" {
            if let Some(function) = node.child_by_field_name("function") {
                if function.kind() == "field_expression" {
                    if let Some(field) = function.child_by_field_name("field") {
                        let field_text = &source[field.byte_range()];

                        if field_text == "next" {
                            // Check if this is part of a method chain that ends with .next()
                            if let Some(object) = function.child_by_field_name("object") {
                                if self.looks_like_iterator(object, source) {
                                    if let Some(args) = node.child_by_field_name("arguments") {
                                        if self.is_empty_argument_list(args, source) {
                                            let position = node.start_position();
                                            violations.push(LintViolation {
                                                line: position.row + 1,
                                                column: position.column + 1,
                                                message: "For loop over iterator chain ending with `.next()` only iterates once. Remove `.next()` to iterate over all items".to_string(),
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
    }

    fn is_empty_argument_list(&self, args_node: Node, source: &str) -> bool {
        let args_text = &source[args_node.byte_range()];
        // Check if it's just "()" or "( )"
        args_text.trim() == "()"
    }

    fn looks_like_iterator(&self, node: Node, source: &str) -> bool {
        let text = &source[node.byte_range()];

        // Check for common iterator patterns
        text.contains(".iter()") ||
        text.contains(".into_iter()") ||
        text.contains(".chars()") ||
        text.contains(".bytes()") ||
        text.contains(".lines()") ||
        text.contains(".split(") ||
        text.contains(".filter(") ||
        text.contains(".map(") ||
        text.contains(".skip(") ||
        text.contains(".take(") ||
        text.contains(".enumerate()") ||
        text.contains(".chain(") ||
        text.contains(".zip(") ||
        // Check if the variable name suggests it's an iterator
        text.ends_with("_iter") ||
        text.ends_with("_iterator") ||
        text.contains("iter") ||
        // Check for range expressions
        text.contains("..") ||
        // Check for common collection methods that return iterators
        node.kind() == "call_expression"
    }
}
