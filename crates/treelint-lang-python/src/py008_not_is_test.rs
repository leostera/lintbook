use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct NotIsTest;

impl Rule for NotIsTest {
    fn id(&self) -> &'static str {
        "PY008"
    }

    fn name(&self) -> &'static str {
        "not-is-test"
    }

    fn description(&self) -> &'static str {
        "Use 'is not' instead of 'not x is y'"
    }

    fn explanation(&self) -> &'static str {
        "In Python, 'not x is y' should be written as 'x is not y' for better readability and consistency with Python idioms."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl NotIsTest {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for 'not' expressions
        if node.kind() == "not_operator" {
            self.check_not_is_pattern(node, source, violations);
        }

        // Recursively visit child nodes
        for child in node.children(&mut node.walk()) {
            self.visit_node(child, source, violations);
        }
    }

    fn check_not_is_pattern(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for pattern: not (x is y)
        if let Some(parent) = node.parent() {
            if parent.kind() == "parenthesized_expression" {
                // Check if the parenthesized expression contains an 'is' comparison
                if let Some(comparison) = self.find_is_comparison(parent) {
                    let start_point = node.start_position();
                    let comparison_text = comparison.utf8_text(source.as_bytes()).unwrap_or("");
                    
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: format!("Use 'is not' instead of 'not ({})'. Rewrite as: {}", 
                                       comparison_text, 
                                       self.suggest_rewrite(comparison_text)),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            } else {
                // Direct case: not x is y (without parentheses)
                let mut cursor = parent.walk();
                let children: Vec<Node> = parent.children(&mut cursor).collect();
                
                for i in 0..children.len() {
                    if children[i] == node && i + 1 < children.len() {
                        let next_node = children[i + 1];
                        if self.is_is_comparison(next_node) {
                            let start_point = node.start_position();
                            let comparison_text = next_node.utf8_text(source.as_bytes()).unwrap_or("");
                            
                            violations.push(LintViolation {
                                line: start_point.row + 1,
                                column: start_point.column + 1,
                                message: format!("Use 'is not' instead of 'not {}'. Rewrite as: {}", 
                                               comparison_text, 
                                               self.suggest_rewrite(comparison_text)),
                                lint_id: self.id().to_string(),
                                lint_name: self.name().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    fn find_is_comparison(&self, node: Node) -> Option<Node> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.is_is_comparison(child) {
                return Some(child);
            }
            // Recursively search in child nodes
            if let Some(found) = self.find_is_comparison(child) {
                return Some(found);
            }
        }
        None
    }

    fn is_is_comparison(&self, node: Node) -> bool {
        if node.kind() == "comparison_operator" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "is" {
                    return true;
                }
            }
        }
        false
    }

    fn suggest_rewrite(&self, comparison_text: &str) -> String {
        // Simple rewrite: "x is y" -> "x is not y"
        comparison_text.replace(" is ", " is not ")
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
    fn test_not_is_parentheses() {
        let source = r#"
if not (x is None):
    pass
"#;
        let tree = parse_python(source);
        let rule = NotIsTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY008");
        assert!(violations[0].message.contains("x is not None"));
    }

    #[test]
    fn test_not_is_direct() {
        let source = r#"
if not x is None:
    pass
"#;
        let tree = parse_python(source);
        let rule = NotIsTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY008");
        assert!(violations[0].message.contains("is not"));
    }

    #[test]
    fn test_correct_is_not() {
        let source = r#"
if x is not None:
    pass
if y is not obj:
    pass
"#;
        let tree = parse_python(source);
        let rule = NotIsTest;
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
if not x in items:
    pass
"#;
        let tree = parse_python(source);
        let rule = NotIsTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_not_is() {
        let source = r#"
if not a is b:
    pass
if not (c is d):
    pass
"#;
        let tree = parse_python(source);
        let rule = NotIsTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY008"));
    }

    #[test]
    fn test_is_with_variables() {
        let source = r#"
if not result is expected:
    handle_error()
"#;
        let tree = parse_python(source);
        let rule = NotIsTest;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY008");
        assert!(violations[0].message.contains("result is not expected"));
    }
}
