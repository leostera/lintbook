use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct AssertTuple;

impl Rule for AssertTuple {
    fn id(&self) -> &'static str {
        "PY016"
    }

    fn name(&self) -> &'static str {
        "assert-tuple"
    }

    fn description(&self) -> &'static str {
        "Assert test is a non-empty tuple"
    }

    fn explanation(&self) -> &'static str {
        "Assert statements with non-empty tuple literals are always true. This is likely a mistake where commas were used instead of logical operators."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl AssertTuple {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for assert statements
        if node.kind() == "assert_statement" {
            self.check_assert_statement(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_assert_statement(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Get the test expression (first child after 'assert' keyword)
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "tuple" {
                    // Check if it's a non-empty tuple
                    if self.is_non_empty_tuple(child) {
                        let start_point = node.start_position();
                        let tuple_text = child.utf8_text(source.as_bytes()).unwrap_or("<tuple>");
                        
                        violations.push(LintViolation {
                            line: start_point.row + 1,
                            column: start_point.column + 1,
                            message: format!(
                                "Assert test is a non-empty tuple: {}. This is always True. Did you mean to use 'and' or 'or' instead of commas?",
                                tuple_text
                            ),
                            lint_id: self.id().to_string(),
                            lint_name: self.name().to_string(),
                        });
                    }
                    break;
                }
            }
        }
    }

    fn is_non_empty_tuple(&self, node: Node) -> bool {
        // A tuple is non-empty if it has any expression children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    // Skip parentheses and commas
                    "(" | ")" | "," => continue,
                    // Any other node means the tuple has content
                    _ => return true,
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_python(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_assert_with_tuple() {
        let source = r#"
# Assert with tuple - always True
assert (1, 2)
assert (x > 0, y < 10)
assert ("error", msg)
"#;
        let tree = parse_python(source);
        let rule = AssertTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY016"));
    }

    #[test]
    fn test_assert_with_single_element_tuple() {
        let source = r#"
# Single element tuple (with trailing comma)
assert (x > 0,)
assert (condition,)
"#;
        let tree = parse_python(source);
        let rule = AssertTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_assert_with_parentheses_not_tuple() {
        let source = r#"
# Parentheses but not a tuple - should not trigger
assert (x > 0)
assert (x > 0 and y < 10)
assert (x or y)
"#;
        let tree = parse_python(source);
        let rule = AssertTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_assert_with_empty_tuple() {
        let source = r#"
# Empty tuple - is False, different issue
assert ()
"#;
        let tree = parse_python(source);
        let rule = AssertTuple;
        let violations = rule.check(&tree, source);

        // Empty tuples are always False, not covered by this rule
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_assert_with_message() {
        let source = r#"
# Assert with message - second argument is fine
assert x > 0, "x must be positive"
assert result, f"Expected result, got {result}"
"#;
        let tree = parse_python(source);
        let rule = AssertTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_assert_without_parentheses() {
        let source = r#"
# Assert without parentheses
assert x > 0
assert condition
assert not error
"#;
        let tree = parse_python(source);
        let rule = AssertTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_tuple_assert() {
        let source = r#"
# Complex tuple assertions
assert (x > 0, y < 10, z != 0)
assert (validate(x), check(y), verify(z))
"#;
        let tree = parse_python(source);
        let rule = AssertTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.message.contains("'and' or 'or'")));
    }
}