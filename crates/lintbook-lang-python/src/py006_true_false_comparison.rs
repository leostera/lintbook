use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct TrueFalseComparison;

impl Rule for TrueFalseComparison {
    fn id(&self) -> &'static str {
        "PY006"
    }

    fn name(&self) -> &'static str {
        "true-false-comparison"
    }

    fn description(&self) -> &'static str {
        "Comparison to True/False should be 'if cond:' or 'if not cond:'"
    }

    fn explanation(&self) -> &'static str {
        "In Python, comparisons to True/False should be simplified to use the truthiness of the expression instead of explicit comparison."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl TrueFalseComparison {
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

        // Look for patterns like "x == True", "False != y", etc.
        for i in 0..children.len().saturating_sub(2) {
            let left = children[i];
            let operator = children[i + 1];
            let right = children[i + 2];

            if operator.kind() == "==" || operator.kind() == "!=" {
                let left_text = left.utf8_text(source.as_bytes()).unwrap_or("");
                let right_text = right.utf8_text(source.as_bytes()).unwrap_or("");
                let operator_text = operator.utf8_text(source.as_bytes()).unwrap_or("");

                let is_true_false_comparison = left_text == "True"
                    || left_text == "False"
                    || right_text == "True"
                    || right_text == "False";

                if is_true_false_comparison {
                    let suggestion = match operator_text {
                        "==" => {
                            if right_text == "True" {
                                format!("Use 'if {}:' instead", left_text)
                            } else if right_text == "False" {
                                format!("Use 'if not {}:' instead", left_text)
                            } else if left_text == "True" {
                                format!("Use 'if {}:' instead", right_text)
                            } else {
                                format!("Use 'if not {}:' instead", right_text)
                            }
                        }
                        "!=" => {
                            if right_text == "True" {
                                format!("Use 'if not {}:' instead", left_text)
                            } else if right_text == "False" {
                                format!("Use 'if {}:' instead", left_text)
                            } else if left_text == "True" {
                                format!("Use 'if not {}:' instead", right_text)
                            } else {
                                format!("Use 'if {}:' instead", right_text)
                            }
                        }
                        _ => continue,
                    };

                    let start_point = operator.start_position();
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: suggestion,
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
    fn test_true_comparison_with_equals() {
        let source = r#"
if x == True:
    pass
"#;
        let tree = parse_python(source);
        let rule = TrueFalseComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY006");
        assert!(violations[0].message.contains("if x:"));
    }

    #[test]
    fn test_false_comparison_with_equals() {
        let source = r#"
if result == False:
    pass
"#;
        let tree = parse_python(source);
        let rule = TrueFalseComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY006");
        assert!(violations[0].message.contains("if not result:"));
    }

    #[test]
    fn test_true_comparison_with_not_equals() {
        let source = r#"
if True != condition:
    pass
"#;
        let tree = parse_python(source);
        let rule = TrueFalseComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY006");
        assert!(violations[0].message.contains("if not condition:"));
    }

    #[test]
    fn test_false_comparison_with_not_equals() {
        let source = r#"
if value != False:
    pass
"#;
        let tree = parse_python(source);
        let rule = TrueFalseComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY006");
        assert!(violations[0].message.contains("if value:"));
    }

    #[test]
    fn test_valid_truthiness_checks() {
        let source = r#"
if x:
    pass
if not y:
    pass
if condition:
    pass
"#;
        let tree = parse_python(source);
        let rule = TrueFalseComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_non_boolean_comparisons() {
        let source = r#"
if x == 5:
    pass
if name != "hello":
    pass
if result == "True":
    pass
"#;
        let tree = parse_python(source);
        let rule = TrueFalseComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }
}
