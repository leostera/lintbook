use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct NotInTest;

impl Rule for NotInTest {
    fn id(&self) -> &'static str {
        "PY007"
    }

    fn name(&self) -> &'static str {
        "not-in-test"
    }

    fn description(&self) -> &'static str {
        "Use 'not in' instead of 'not x in y'"
    }

    fn explanation(&self) -> &'static str {
        "In Python, 'not x in y' should be written as 'x not in y' for better readability and consistency with Python idioms."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl NotInTest {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for 'not' unary expressions
        if node.kind() == "not_operator" {
            // Check the operand of the not operator
            if let Some(operand) = node.child(1) {
                // Check if it's directly an 'in' comparison
                if operand.kind() == "comparison_operator" {
                    if self.has_in_operator(operand) {
                        let start_point = node.start_position();
                        let full_text = source[node.byte_range()].to_string();
                        
                        violations.push(LintViolation {
                            line: start_point.row + 1,
                            column: start_point.column + 1,
                            message: format!("Use 'not in' instead of '{}'. Consider rewriting the comparison.", full_text.trim()),
                            lint_id: self.id().to_string(),
                            lint_name: self.name().to_string(),
                        });
                    }
                }
                // Check if it's a parenthesized expression containing 'in'
                else if operand.kind() == "parenthesized_expression" {
                    if let Some(inner) = operand.child(1) {  // Skip the opening paren
                        if inner.kind() == "comparison_operator" && self.has_in_operator(inner) {
                            let start_point = node.start_position();
                            let full_text = source[node.byte_range()].to_string();
                            
                            violations.push(LintViolation {
                                line: start_point.row + 1,
                                column: start_point.column + 1,
                                message: format!("Use 'not in' instead of '{}'. Consider rewriting the comparison.", full_text.trim()),
                                lint_id: self.id().to_string(),
                                lint_name: self.name().to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn has_in_operator(&self, node: Node) -> bool {
        // Check if this comparison_operator node contains an 'in' operator
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "in" {
                    return true;
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
    fn test_not_in_direct() {
        let source = r#"
if not x in items:
    pass
"#;
        let tree = parse_python(source);
        let rule = NotInTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY007");
        assert!(violations[0].message.contains("not in"));
    }

    #[test]
    fn test_not_in_parentheses() {
        let source = r#"
if not (x in items):
    pass
"#;
        let tree = parse_python(source);
        let rule = NotInTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY007");
        assert!(violations[0].message.contains("not in"));
    }

    #[test]
    fn test_correct_not_in() {
        let source = r#"
if x not in items:
    pass
if y not in collection:
    pass
"#;
        let tree = parse_python(source);
        let rule = NotInTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_not_with_other_operators() {
        let source = r#"
if not x == y:
    pass
if not x > 5:
    pass
"#;
        let tree = parse_python(source);
        let rule = NotInTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_violations() {
        let source = r#"
if not x in items:
    pass
elif not (y in collection):
    pass
"#;
        let tree = parse_python(source);
        let rule = NotInTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
    }
}