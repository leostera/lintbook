use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct IsLiteral;

impl Rule for IsLiteral {
    fn id(&self) -> &'static str {
        "PY017"
    }

    fn name(&self) -> &'static str {
        "is-literal"
    }

    fn description(&self) -> &'static str {
        "Use == for literal comparisons instead of 'is'"
    }

    fn explanation(&self) -> &'static str {
        "Using 'is' with literals (strings, numbers, etc.) is implementation-specific and unreliable. Use '==' for value comparisons with literals."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl IsLiteral {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for comparison operators
        if node.kind() == "comparison_operator" {
            self.check_is_literal_comparison(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_is_literal_comparison(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut has_is_operator = false;
        let mut has_literal = false;
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "is" | "is not" => {
                        has_is_operator = true;
                    },
                    kind if self.is_literal_kind(kind) => {
                        has_literal = true;
                    },
                    _ => {}
                }
            }
        }
        
        if has_is_operator && has_literal {
            let start_point = node.start_position();
            let full_text = source[node.byte_range()].to_string();
            let suggestion = if full_text.contains("is not") {
                full_text.replace("is not", "!=")
            } else {
                full_text.replace(" is ", " == ")
            };
            
            violations.push(LintViolation {
                line: start_point.row + 1,
                column: start_point.column + 1,
                message: format!(
                    "Use '==' for literal comparison instead of 'is': '{}'. Suggestion: '{}'",
                    full_text.trim(),
                    suggestion.trim()
                ),
                lint_id: self.id().to_string(),
                lint_name: self.name().to_string(),
            });
        }
    }

    fn is_literal_kind(&self, kind: &str) -> bool {
        matches!(kind, 
            "string" | 
            "integer" | 
            "float" | 
            "true" | 
            "false" |
            "list" |
            "dictionary" |
            "tuple" |
            "set"
        )
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
    fn test_is_with_string_literal() {
        let source = r#"
# Using 'is' with string literals
if x is "hello":
    pass
if name is not "admin":
    pass
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY017"));
        assert!(violations[0].message.contains("=="));
        assert!(violations[1].message.contains("!="));
    }

    #[test]
    fn test_is_with_numeric_literal() {
        let source = r#"
# Using 'is' with numeric literals
if count is 0:
    pass
if value is not 42:
    pass
if price is 99.99:
    pass
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY017"));
    }

    #[test]
    fn test_is_with_boolean_literal() {
        let source = r#"
# Using 'is' with boolean literals
if result is True:
    pass
if success is not False:
    pass
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY017"));
    }

    #[test]
    fn test_is_with_collection_literals() {
        let source = r#"
# Using 'is' with collection literals
if items is []:
    pass
if data is {}:
    pass
if values is ():
    pass
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY017"));
    }

    #[test]
    fn test_is_with_none() {
        let source = r#"
# Using 'is' with None is correct - should not trigger
if x is None:
    pass
if value is not None:
    pass
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_equals_with_literals() {
        let source = r#"
# Using '==' with literals is correct - should not trigger
if x == "hello":
    pass
if count == 0:
    pass
if result != True:
    pass
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_is_with_variables() {
        let source = r#"
# Using 'is' with variables - should not trigger
if x is y:
    pass
if obj1 is not obj2:
    pass
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_violations() {
        let source = r#"
# Multiple violations in one file
def check_values():
    if status is "active":
        return True
    elif status is "pending":
        return None
    elif status is not "inactive":
        return False
"#;
        let tree = parse_python(source);
        let rule = IsLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY017"));
    }
}