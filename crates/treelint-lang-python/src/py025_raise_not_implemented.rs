use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct RaiseNotImplemented;

impl Rule for RaiseNotImplemented {
    fn id(&self) -> &'static str {
        "PY025"
    }

    fn name(&self) -> &'static str {
        "raise-not-implemented"
    }

    fn description(&self) -> &'static str {
        "Use NotImplementedError instead of NotImplemented"
    }

    fn explanation(&self) -> &'static str {
        "NotImplemented is a singleton constant, not an exception. Use NotImplementedError() for raising exceptions."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl RaiseNotImplemented {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for raise statements
        if node.kind() == "raise_statement" {
            self.check_raise_statement(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_raise_statement(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Check what's being raised
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" {
                    let identifier = child.utf8_text(source.as_bytes()).unwrap_or("");
                    if identifier == "NotImplemented" {
                        let start_point = node.start_position();
                        
                        violations.push(LintViolation {
                            line: start_point.row + 1,
                            column: start_point.column + 1,
                            message: format!(
                                "Use 'raise NotImplementedError()' instead of 'raise NotImplemented'. NotImplemented is a constant, not an exception."
                            ),
                            lint_id: self.id().to_string(),
                            lint_name: self.name().to_string(),
                        });
                    }
                }
                // Also check if it's being called (which is still wrong)
                else if child.kind() == "call" {
                    if let Some(func) = child.child_by_field_name("function") {
                        if func.kind() == "identifier" {
                            let identifier = func.utf8_text(source.as_bytes()).unwrap_or("");
                            if identifier == "NotImplemented" {
                                let start_point = node.start_position();
                                
                                violations.push(LintViolation {
                                    line: start_point.row + 1,
                                    column: start_point.column + 1,
                                    message: format!(
                                        "Use 'raise NotImplementedError()' instead of 'raise NotImplemented()'. NotImplemented is a constant, not an exception."
                                    ),
                                    lint_id: self.id().to_string(),
                                    lint_name: self.name().to_string(),
                                });
                            }
                        }
                    }
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
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_raise_not_implemented() {
        let source = r#"
# Raising NotImplemented (wrong)
def my_method():
    raise NotImplemented
"#;
        let tree = parse_python(source);
        let rule = RaiseNotImplemented;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY025");
        assert!(violations[0].message.contains("NotImplementedError()"));
    }

    #[test]
    fn test_raise_not_implemented_with_call() {
        let source = r#"
# Calling NotImplemented() (still wrong)
def my_method():
    raise NotImplemented()
"#;
        let tree = parse_python(source);
        let rule = RaiseNotImplemented;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY025");
        assert!(violations[0].message.contains("NotImplementedError()"));
    }

    #[test]
    fn test_raise_not_implemented_error() {
        let source = r#"
# Correct usage
def my_method():
    raise NotImplementedError()

def another_method():
    raise NotImplementedError("This feature is not implemented yet")
"#;
        let tree = parse_python(source);
        let rule = RaiseNotImplemented;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_other_raises() {
        let source = r#"
# Other exceptions - should not trigger
def test():
    raise ValueError("Invalid value")
    raise TypeError()
    raise Exception("Something went wrong")
"#;
        let tree = parse_python(source);
        let rule = RaiseNotImplemented;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_not_implemented_in_other_contexts() {
        let source = r#"
# NotImplemented used outside of raise - should not trigger this rule
result = NotImplemented
if value is NotImplemented:
    pass
return NotImplemented
"#;
        let tree = parse_python(source);
        let rule = RaiseNotImplemented;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_violations() {
        let source = r#"
class MyClass:
    def method1(self):
        raise NotImplemented
    
    def method2(self):
        raise NotImplemented()
    
    def method3(self):
        if condition:
            raise NotImplemented
"#;
        let tree = parse_python(source);
        let rule = RaiseNotImplemented;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY025"));
    }

    #[test]
    fn test_raise_from_not_implemented() {
        let source = r#"
# Raise from syntax
try:
    something()
except Exception as e:
    raise NotImplemented from e  # Wrong
"#;
        let tree = parse_python(source);
        let rule = RaiseNotImplemented;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }
}