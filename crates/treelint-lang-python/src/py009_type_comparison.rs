use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct TypeComparison;

impl Rule for TypeComparison {
    fn id(&self) -> &'static str {
        "PY009"
    }

    fn name(&self) -> &'static str {
        "type-comparison"
    }

    fn description(&self) -> &'static str {
        "Use isinstance() instead of type() comparison"
    }

    fn explanation(&self) -> &'static str {
        "Comparing types using type() == or type() is should be replaced with isinstance() for better inheritance support and readability."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl TypeComparison {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for comparison operators
        if node.kind() == "comparison_operator" {
            self.check_type_comparison(node, source, violations);
        }

        // Recursively visit child nodes
        for child in node.children(&mut node.walk()) {
            self.visit_node(child, source, violations);
        }
    }

    fn check_type_comparison(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        // Look for patterns like "type(x) == SomeType", "type(x) is SomeType", etc.
        for i in 0..children.len().saturating_sub(2) {
            let left = children[i];
            let operator = children[i + 1];
            let right = children[i + 2];

            if operator.kind() == "==" || operator.kind() == "is" || operator.kind() == "!=" || operator.kind() == "is_not" {
                let left_text = left.utf8_text(source.as_bytes()).unwrap_or("");
                let right_text = right.utf8_text(source.as_bytes()).unwrap_or("");
                let operator_text = operator.utf8_text(source.as_bytes()).unwrap_or("");

                let left_is_type_call = self.is_type_call(&left_text);
                let right_is_type_call = self.is_type_call(&right_text);

                if left_is_type_call || right_is_type_call {
                    let start_point = operator.start_position();
                    let suggestion = self.create_isinstance_suggestion(&left_text, &right_text, &operator_text);
                    
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: format!("Use isinstance() instead of type() comparison. Suggestion: {}", suggestion),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            }
        }
    }

    fn is_type_call(&self, text: &str) -> bool {
        // Simple check for type() function calls
        text.trim().starts_with("type(") && text.trim().ends_with(")")
    }

    fn create_isinstance_suggestion(&self, left: &str, right: &str, operator: &str) -> String {
        let left_trimmed = left.trim();
        let right_trimmed = right.trim();
        
        if self.is_type_call(left_trimmed) {
            // type(obj) == SomeType -> isinstance(obj, SomeType)
            let obj = self.extract_object_from_type_call(left_trimmed);
            match operator {
                "==" | "is" => format!("isinstance({}, {})", obj, right_trimmed),
                "!=" | "is_not" => format!("not isinstance({}, {})", obj, right_trimmed),
                _ => format!("isinstance({}, {})", obj, right_trimmed),
            }
        } else if self.is_type_call(right_trimmed) {
            // SomeType == type(obj) -> isinstance(obj, SomeType)
            let obj = self.extract_object_from_type_call(right_trimmed);
            match operator {
                "==" | "is" => format!("isinstance({}, {})", obj, left_trimmed),
                "!=" | "is_not" => format!("not isinstance({}, {})", obj, left_trimmed),
                _ => format!("isinstance({}, {})", obj, left_trimmed),
            }
        } else {
            format!("isinstance(...)")
        }
    }

    fn extract_object_from_type_call(&self, type_call: &str) -> &str {
        // Extract object from "type(object)" -> "object"
        if type_call.starts_with("type(") && type_call.ends_with(")") {
            &type_call[5..type_call.len()-1]
        } else {
            "obj"
        }
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
    fn test_type_equals_comparison() {
        let source = r#"
if type(obj) == str:
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY009");
        assert!(violations[0].message.contains("isinstance(obj, str)"));
    }

    #[test]
    fn test_type_is_comparison() {
        let source = r#"
if type(value) is int:
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY009");
        assert!(violations[0].message.contains("isinstance(value, int)"));
    }

    #[test]
    fn test_type_not_equals_comparison() {
        let source = r#"
if type(data) != list:
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY009");
        assert!(violations[0].message.contains("not isinstance(data, list)"));
    }

    #[test]
    fn test_reversed_type_comparison() {
        let source = r#"
if str == type(text):
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY009");
        assert!(violations[0].message.contains("isinstance(text, str)"));
    }

    #[test]
    fn test_correct_isinstance_usage() {
        let source = r#"
if isinstance(obj, str):
    pass
if isinstance(value, (int, float)):
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_non_type_comparisons() {
        let source = r#"
if obj == "hello":
    pass
if value > 5:
    pass
if data is None:
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_type_expressions() {
        let source = r#"
if type(obj.attribute) == dict:
    pass
if type(result[0]) is tuple:
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY009"));
    }
}
