use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct ReturnOutsideFunction;

impl Rule for ReturnOutsideFunction {
    fn id(&self) -> &'static str {
        "PY023"
    }

    fn name(&self) -> &'static str {
        "return-outside-function"
    }

    fn description(&self) -> &'static str {
        "Return statement outside function"
    }

    fn explanation(&self) -> &'static str {
        "Return statements can only be used inside functions or methods. Using return outside a function is a syntax error."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations, false);
        violations
    }
}

impl ReturnOutsideFunction {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>, in_function: bool) {
        match node.kind() {
            "return_statement" => {
                if !in_function {
                    let start_point = node.start_position();
                    
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: "Return statement outside function. Return can only be used inside functions or methods.".to_string(),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            },
            "function_definition" | "async_function_definition" => {
                // Recursively visit children with in_function=true
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, true);
                    }
                }
                return; // Don't fall through to general recursion
            },
            "lambda" => {
                // Lambdas are functions, so returns inside them are valid
                // Note: Actually, lambdas can't have return statements in Python, only expressions
                // But we handle this for completeness
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, true);
                    }
                }
                return; // Don't fall through to general recursion
            },
            _ => {}
        }

        // Recursively visit child nodes with current function state
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations, in_function);
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
    fn test_return_outside_function() {
        let source = r#"
# Return at module level
if condition:
    return value
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY023");
        assert!(violations[0].message.contains("outside function"));
    }

    #[test]
    fn test_return_in_function() {
        let source = r#"
# Correct usage in function
def my_function():
    if condition:
        return "early"
    return "normal"
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_return_in_method() {
        let source = r#"
# Correct usage in method
class MyClass:
    def my_method(self):
        return self.value
    
    def another_method(self):
        if self.condition:
            return None
        return self.process()
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_return_in_async_function() {
        let source = r#"
# Correct usage in async function
async def async_function():
    await some_operation()
    return "result"
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_return_in_nested_function() {
        let source = r#"
# Return in nested function
def outer():
    def inner():
        return 42  # Valid: inside nested function
    return inner()
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_return_in_class_body() {
        let source = r#"
# Return in class body (invalid)
class MyClass:
    if debug:
        return "debug"  # Invalid: not in a method
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_multiple_returns_outside_function() {
        let source = r#"
# Multiple returns at module level
try:
    return 1  # Invalid
except:
    return 2  # Invalid
finally:
    return 3  # Invalid
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY023"));
    }

    #[test]
    fn test_return_in_control_structures_inside_function() {
        let source = r#"
# Returns in control structures inside function - valid
def my_function():
    if condition:
        return 1  # Valid
    
    try:
        return 2  # Valid
    except:
        return 3  # Valid
    
    for i in range(10):
        if i == 5:
            return i  # Valid
    
    while True:
        if ready:
            return result  # Valid
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_return_in_lambda() {
        let source = r#"
# Lambdas typically use expressions, not return statements
# But if they did, they should be valid
func = lambda x: x * 2  # This is an expression, not a return statement
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        // No return statements in this code
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_nesting() {
        let source = r#"
# Complex nesting scenarios
class MyClass:
    def method(self):
        def inner():
            return 1  # Valid: inside inner function
        return inner()  # Valid: inside method
    
# This would be invalid - return in module level
if True:
    return "module_level"  # Invalid: module level
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_return_in_generator() {
        let source = r#"
# Return in generator function (valid)
def my_generator():
    yield 1
    yield 2
    return "done"  # Valid: return inside generator function
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_return_in_decorator() {
        let source = r#"
# Return in decorator function
def my_decorator(func):
    def wrapper(*args, **kwargs):
        result = func(*args, **kwargs)
        return result  # Valid: inside wrapper function
    return wrapper  # Valid: inside decorator function
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_module_level_loops_with_return() {
        let source = r#"
# Module-level control structures with returns
for item in items:
    if item.done():
        return item  # Invalid: return at module level

while condition:
    if should_exit:
        return "exit"  # Invalid: return at module level
"#;
        let tree = parse_python(source);
        let rule = ReturnOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY023"));
    }
}