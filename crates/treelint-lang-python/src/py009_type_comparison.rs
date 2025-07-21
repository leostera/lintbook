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
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_type_comparison(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Get children of comparison_operator
        let mut has_type_call = false;
        let mut has_comparison_operator = false;
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "==" | "!=" | "is" | "is not" => {
                        has_comparison_operator = true;
                    },
                    "call" => {
                        // Check if this is a type() call
                        if self.is_type_call(child, source) {
                            has_type_call = true;
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // If we found a type() call in a comparison, report violation
        if has_type_call && has_comparison_operator {
            let start_point = node.start_position();
            let full_text = source[node.byte_range()].to_string();
            
            violations.push(LintViolation {
                line: start_point.row + 1,
                column: start_point.column + 1,
                message: format!(
                    "Use isinstance() instead of type() comparison: '{}'. Consider using isinstance() for better inheritance support.",
                    full_text.trim()
                ),
                lint_id: self.id().to_string(),
                lint_name: self.name().to_string(),
            });
        }
    }

    fn is_type_call(&self, node: Node, source: &str) -> bool {
        if node.kind() != "call" {
            return false;
        }
        
        // Check if the function being called is 'type'
        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "identifier" {
                let function_name = function_node.utf8_text(source.as_bytes()).unwrap_or("");
                return function_name == "type";
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
        assert!(violations[0].message.contains("isinstance()"));
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
        assert!(violations[0].message.contains("isinstance()"));
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
        assert!(violations[0].message.contains("isinstance()"));
    }

    #[test]
    fn test_type_is_not_comparison() {
        let source = r#"
if type(result) is not dict:
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY009");
        assert!(violations[0].message.contains("isinstance()"));
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
        assert!(violations[0].message.contains("isinstance()"));
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

    #[test]
    fn test_multiple_type_checks() {
        let source = r#"
if type(x) == int and type(y) == str:
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY009"));
    }

    #[test]
    fn test_other_function_calls() {
        let source = r#"
if len(obj) == 5:
    pass
if str(value) == "test":
    pass
"#;
        let tree = parse_python(source);
        let rule = TypeComparison;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }
}