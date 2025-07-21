use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct NoneComparison;

impl Rule for NoneComparison {
    fn id(&self) -> &'static str {
        "PY005"
    }

    fn name(&self) -> &'static str {
        "none-comparison"
    }

    fn description(&self) -> &'static str {
        "Comparison to None should be 'cond is None'"
    }

    fn explanation(&self) -> &'static str {
        "In Python, comparisons to None should use 'is' or 'is not' operators instead of '==' or '!=' for better performance and clarity."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl NoneComparison {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for comparison nodes
        if node.kind() == "comparison_operator" {
            self.check_comparison(node, source, violations);
        }

        // Recursively visit child nodes
        for child in node.children(&mut node.walk()) {
            self.visit_node(child, source, violations);
        }
    }

    fn check_comparison(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        // Look for patterns like "x == None" or "None != y"
        for i in 0..children.len().saturating_sub(2) {
            let left = children[i];
            let operator = children[i + 1];
            let right = children[i + 2];

            if operator.kind() == "==" || operator.kind() == "!=" {
                let left_text = left.utf8_text(source.as_bytes()).unwrap_or("");
                let right_text = right.utf8_text(source.as_bytes()).unwrap_or("");
                let operator_text = operator.utf8_text(source.as_bytes()).unwrap_or("");

                let is_none_comparison = left_text == "None" || right_text == "None";

                if is_none_comparison {
                    let suggested_operator = match operator_text {
                        "==" => "is",
                        "!=" => "is not",
                        _ => continue,
                    };

                    let start_point = operator.start_position();
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: format!(
                            "Comparison to None should be 'cond {} None'",
                            suggested_operator
                        ),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_python(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_none_comparison_with_equals() {
        let source = r#"
if x == None:
    pass
"#;
        let tree = parse_python(source);
        let rule = NoneComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY005");
        assert!(violations[0].message.contains("is"));
    }

    #[test]
    fn test_none_comparison_with_not_equals() {
        let source = r#"
if None != y:
    pass
"#;
        let tree = parse_python(source);
        let rule = NoneComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY005");
        assert!(violations[0].message.contains("is not"));
    }

    #[test]
    fn test_valid_none_comparison() {
        let source = r#"
if x is None:
    pass
if y is not None:
    pass
"#;
        let tree = parse_python(source);
        let rule = NoneComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_non_none_comparison() {
        let source = r#"
if x == 5:
    pass
if "hello" != name:
    pass
"#;
        let tree = parse_python(source);
        let rule = NoneComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }
}
