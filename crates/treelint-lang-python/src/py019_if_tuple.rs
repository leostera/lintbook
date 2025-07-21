use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct IfTuple;

impl Rule for IfTuple {
    fn id(&self) -> &'static str {
        "PY019"
    }

    fn name(&self) -> &'static str {
        "if-tuple"
    }

    fn description(&self) -> &'static str {
        "If test is a non-empty tuple"
    }

    fn explanation(&self) -> &'static str {
        "If statements with non-empty tuple literals are always true. This is likely a mistake where commas were used instead of logical operators."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl IfTuple {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for if statements and elif clauses
        match node.kind() {
            "if_statement" => {
                self.check_if_statement(node, source, violations);
            },
            "elif_clause" => {
                self.check_elif_clause(node, source, violations);
            },
            _ => {}
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_if_statement(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // The condition is typically the second child (after 'if' keyword)
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "tuple" && self.is_non_empty_tuple(child) {
                    self.report_violation(node, child, source, violations);
                    break;
                }
            }
        }
    }

    fn check_elif_clause(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Similar to if statement, check for tuple after 'elif' keyword
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "tuple" && self.is_non_empty_tuple(child) {
                    self.report_violation(node, child, source, violations);
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

    fn report_violation(&self, statement_node: Node, tuple_node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let start_point = statement_node.start_position();
        let tuple_text = tuple_node.utf8_text(source.as_bytes()).unwrap_or("<tuple>");
        
        violations.push(LintViolation {
            line: start_point.row + 1,
            column: start_point.column + 1,
            message: format!(
                "If test is a non-empty tuple: {}. This is always True. Did you mean to use 'and' or 'or' instead of commas?",
                tuple_text
            ),
            lint_id: self.id().to_string(),
            lint_name: self.name().to_string(),
        });
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
    fn test_if_with_tuple() {
        let source = r#"
# If with tuple - always True
if (1, 2):
    pass

if (x > 0, y < 10):
    do_something()

if ("error", msg):
    handle_error()
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY019"));
    }

    #[test]
    fn test_elif_with_tuple() {
        let source = r#"
# Elif with tuple
if x < 0:
    negative()
elif (x > 0, x < 10):
    small_positive()
elif (x >= 10, x < 100):
    medium()
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY019"));
    }

    #[test]
    fn test_single_element_tuple() {
        let source = r#"
# Single element tuple (with trailing comma)
if (x > 0,):
    pass

if (condition,):
    execute()
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_if_with_parentheses_not_tuple() {
        let source = r#"
# Parentheses but not a tuple - should not trigger
if (x > 0):
    pass

if (x > 0 and y < 10):
    pass

if (x or y):
    pass
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_if_with_empty_tuple() {
        let source = r#"
# Empty tuple - is False, different issue
if ():
    pass
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        // Empty tuples are always False, not covered by this rule
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_if_without_parentheses() {
        let source = r#"
# If without parentheses
if x > 0:
    pass

if condition:
    pass

if not error:
    pass
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_tuple_if() {
        let source = r#"
# Complex tuple conditions
if (x > 0, y < 10, z != 0):
    process()

if (validate(x), check(y), verify(z)):
    continue_processing()
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.message.contains("'and' or 'or'")));
    }

    #[test]
    fn test_nested_if_statements() {
        let source = r#"
# Nested if statements
if x > 0:
    if (y > 0, z > 0):  # Inner if has tuple
        process()
"#;
        let tree = parse_python(source);
        let rule = IfTuple;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }
}