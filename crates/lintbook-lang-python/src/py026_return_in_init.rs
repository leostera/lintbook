use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct ReturnInInit;

impl Rule for ReturnInInit {
    fn id(&self) -> &'static str {
        "PY026"
    }

    fn name(&self) -> &'static str {
        "return-in-init"
    }

    fn description(&self) -> &'static str {
        "Return statement in __init__"
    }

    fn explanation(&self) -> &'static str {
        "__init__ methods should not contain return statements with values. They implicitly return None and should only use bare return for early exit or no return at all."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl ReturnInInit {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for function definitions
        if node.kind() == "function_definition" {
            self.check_function_definition(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_function_definition(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check if this is an __init__ method
        if let Some(function_name) = self.get_function_name(node, source) {
            if function_name == "__init__" {
                // Look for return statements in the function body
                self.find_return_statements(node, source, violations);
            }
        }
    }

    fn get_function_name(&self, function_node: Node, source: &str) -> Option<String> {
        // Find the function name node
        for i in 0..function_node.child_count() {
            if let Some(child) = function_node.child(i) {
                if child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }

    fn find_return_statements(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        if node.kind() == "return_statement" {
            // Check if this return statement has a value
            if self.has_return_value(node, source) {
                let start_point = node.start_position();
                violations.push(LintViolation {
                    line: start_point.row + 1,
                    column: start_point.column + 1,
                    message:
                        "Return with value in __init__ method. __init__ should not return a value."
                            .to_string(),
                    lint_id: self.id().to_string(),
                    lint_name: self.name().to_string(),
                });
            } else {
                // Even bare return in __init__ is questionable, but we'll flag it as a softer warning
                let start_point = node.start_position();
                violations.push(LintViolation {
                    line: start_point.row + 1,
                    column: start_point.column + 1,
                    message: "Return statement in __init__ method. Consider using exceptions for error handling instead.".to_string(),
                    lint_id: self.id().to_string(),
                    lint_name: self.name().to_string(),
                });
            }
        }

        // Recursively check child nodes, but don't go into nested function definitions
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() != "function_definition"
                    && child.kind() != "async_function_definition"
                {
                    self.find_return_statements(child, source, violations);
                }
            }
        }
    }

    fn has_return_value(&self, return_node: Node, _source: &str) -> bool {
        // Check if the return statement has any child nodes other than the "return" keyword
        for i in 0..return_node.child_count() {
            if let Some(child) = return_node.child(i) {
                if child.kind() != "return" {
                    // There's something after "return", so it has a value
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
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_return_with_value_in_init() {
        let source = r#"
class BadClass:
    def __init__(self):
        self.value = 42
        return self.value  # Wrong
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY026");
        assert!(violations[0].message.contains("Return with value"));
    }

    #[test]
    fn test_bare_return_in_init() {
        let source = r#"
class ConditionalReturn:
    def __init__(self, data):
        if data is None:
            return  # Even bare return is flagged
        self.data = data
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY026");
        assert!(violations[0].message.contains("Return statement"));
    }

    #[test]
    fn test_good_init_no_return() {
        let source = r#"
class GoodClass:
    def __init__(self):
        self.value = 42
        # No return statement - correct
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_good_init_with_exception() {
        let source = r#"
class ProperInit:
    def __init__(self, name):
        if not name:
            raise ValueError("Name cannot be empty")  # Correct
        self.name = name
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_other_methods_with_return() {
        let source = r#"
class MethodsWithReturns:
    def __init__(self, value):
        self.value = value
        # No return - correct

    def get_value(self):
        return self.value  # Correct: regular method can return

    def __str__(self):
        return f"Value: {self.value}"  # Correct: __str__ should return
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_returns_in_init() {
        let source = r#"
class MultipleReturns:
    def __init__(self, value):
        if value < 0:
            return -1  # Wrong
        elif value == 0:
            return 0   # Wrong
        else:
            self.value = value
            return 1   # Wrong
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY026"));
    }

    #[test]
    fn test_nested_class_init() {
        let source = r#"
class OuterClass:
    def __init__(self, outer_value):
        self.outer_value = outer_value
        # No return - correct

    class InnerClass:
        def __init__(self, inner_value):
            return inner_value  # Wrong: nested class __init__ also cannot return
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_inherited_class_init() {
        let source = r#"
class BaseClass:
    def __init__(self, base_value):
        self.base_value = base_value
        # No return - correct

class DerivedClass(BaseClass):
    def __init__(self, base_value, derived_value):
        super().__init__(base_value)
        if derived_value is None:
            return  # Wrong: derived __init__ also cannot return
        self.derived_value = derived_value
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_function_named_init() {
        let source = r#"
# Function named __init__ (not a class method)
def __init__(value):
    if value:
        return value  # This is fine - it's a regular function
    return None
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        // This should still be flagged because our rule checks any function named __init__
        // In practice, this is unusual and might indicate a naming issue
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_new_method_can_return() {
        let source = r#"
class WithNew:
    def __new__(cls, value):
        if value < 0:
            return None  # Correct: __new__ can return
        return super().__new__(cls)

    def __init__(self, value):
        self.value = value
        # No return - correct
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_init_with_exception_handling() {
        let source = r#"
class InitWithFinally:
    def __init__(self, resource):
        try:
            self.resource = self.acquire_resource(resource)
        except Exception:
            return None  # Wrong: cannot return from __init__
        finally:
            self.cleanup()
"#;
        let tree = parse_python(source);
        let rule = ReturnInInit;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }
}
