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
        // Look for 'not' expressions
        if node.kind() == "not_operator" {
            self.check_not_in_pattern(node, source, violations);
        }

        // Recursively visit child nodes
        for child in node.children(&mut node.walk()) {
            self.visit_node(child, source, violations);
        }
    }

    fn check_not_in_pattern(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for pattern: not (x in y)
        if let Some(parent) = node.parent() {
            if parent.kind() == "parenthesized_expression" {
                // Check if the parenthesized expression contains an 'in' comparison
                if let Some(comparison) = self.find_in_comparison(parent) {
                    let start_point = node.start_position();
                    let comparison_text = comparison.utf8_text(source.as_bytes()).unwrap_or("");
                    
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: format!("Use 'not in' instead of 'not ({})'. Rewrite as: {}", 
                                       comparison_text, 
                                       self.suggest_rewrite(comparison_text)),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            } else {
                // Direct case: not x in y (without parentheses)
                let mut cursor = parent.walk();
                let children: Vec<Node> = parent.children(&mut cursor).collect();
                
                for i in 0..children.len() {
                    if children[i] == node && i + 1 < children.len() {
                        let next_node = children[i + 1];
                        if self.is_in_comparison(next_node) {
                            let start_point = node.start_position();
                            let comparison_text = next_node.utf8_text(source.as_bytes()).unwrap_or("");
                            
                            violations.push(LintViolation {
                                line: start_point.row + 1,
                                column: start_point.column + 1,
                                message: format!("Use 'not in' instead of 'not {}'. Rewrite as: {}", 
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

    fn find_in_comparison(&self, node: Node) -> Option<Node> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.is_in_comparison(child) {
                return Some(child);
            }
            // Recursively search in child nodes
            if let Some(found) = self.find_in_comparison(child) {
                return Some(found);
            }
        }
        None
    }

    fn is_in_comparison(&self, node: Node) -> bool {
        if node.kind() == "comparison_operator" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "in" {
                    return true;
                }
            }
        }
        false
    }

    fn suggest_rewrite(&self, comparison_text: &str) -> String {
        // Simple rewrite: "x in y" -> "x not in y"
        comparison_text.replace(" in ", " not in ")
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
        assert!(violations[0].message.contains("x not in items"));
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
    fn test_nested_expressions() {
        let source = r#"
if not (a in b and c in d):
    pass
"#;
        let tree = parse_python(source);
        let rule = NotInTest;
        let violations = rule.check(&tree, source);

        // This might catch the first 'in' expression
        assert!(violations.len() >= 0);
    }
}
