use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct YieldOutsideFunction;

impl Rule for YieldOutsideFunction {
    fn id(&self) -> &'static str {
        "PY022"
    }

    fn name(&self) -> &'static str {
        "yield-outside-function"
    }

    fn description(&self) -> &'static str {
        "Yield statement outside function"
    }

    fn explanation(&self) -> &'static str {
        "Yield statements can only be used inside functions or methods. Using yield outside a function is a syntax error."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations, false);
        violations
    }
}

impl YieldOutsideFunction {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>, in_function: bool) {
        match node.kind() {
            "expression_statement" => {
                // Check if this expression statement contains a yield at the top level
                if !in_function && self.contains_yield_at_top_level(node) {
                    let start_point = node.start_position();
                    
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: "Yield statement outside function. Yield can only be used inside functions or methods.".to_string(),
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
            "generator_expression" | "list_comprehension" | "set_comprehension" | "dictionary_comprehension" => {
                // Comprehensions create their own scope and can contain yield in generator expressions
                // For list/set/dict comprehensions, yields would still be invalid
                // But generator expressions can contain yields (though it's unusual)
                let is_generator = node.kind() == "generator_expression";
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, is_generator || in_function);
                    }
                }
                return; // Don't fall through to general recursion
            },
            "lambda" => {
                // Lambdas are functions, so yields inside them are valid
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

    fn contains_yield_at_top_level(&self, node: Node) -> bool {
        // Check direct children for yield expressions
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "yield" {
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
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_yield_outside_function() {
        let source = r#"
# Yield at module level
if condition:
    yield value
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY022");
        assert!(violations[0].message.contains("outside function"));
    }

    #[test]
    fn test_yield_in_function() {
        let source = r#"
# Correct usage in function
def my_generator():
    yield 42
    yield from range(10)
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_yield_in_method() {
        let source = r#"
# Correct usage in method
class MyClass:
    def my_generator(self):
        yield self.value
        yield from self.other_values()
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_yield_in_async_function() {
        let source = r#"
# Correct usage in async function
async def async_generator():
    yield 1
    yield 2
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_yield_in_nested_function() {
        let source = r#"
# Yield in nested function
def outer():
    def inner_generator():
        yield 42  # Valid: inside nested function
    return inner_generator()
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_yield_in_class_body() {
        let source = r#"
# Yield in class body (invalid)
class MyClass:
    if debug:
        yield "debug"  # Invalid: not in a method
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_yield_in_lambda() {
        let source = r#"
# Yield in lambda (unusual but valid)
generator_lambda = lambda: (yield 42)
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        // Lambdas are functions, so yields inside them should be valid
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_yields_outside_function() {
        let source = r#"
# Multiple yields at module level
try:
    yield 1  # Invalid
except:
    yield 2  # Invalid
finally:
    yield 3  # Invalid
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY022"));
    }

    #[test]
    fn test_yield_in_control_structures_inside_function() {
        let source = r#"
# Yields in control structures inside function - valid
def my_generator():
    if condition:
        yield 1  # Valid
    
    try:
        yield 2  # Valid
    except:
        yield 3  # Valid
    
    for i in range(10):
        yield i  # Valid
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_yield_from_variations() {
        let source = r#"
# Test yield from statements
def valid_generator():
    yield from range(10)  # Valid
    yield from other_gen()  # Valid

# Invalid yield from at module level
yield from some_generator()  # Invalid
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_complex_nesting() {
        let source = r#"
# Complex nesting scenarios
class MyClass:
    def method(self):
        def inner():
            yield 1  # Valid: inside inner function
        
        for i in range(10):
            yield i  # Valid: inside method
    
    # This would be invalid - yield in class body
    # yield "class_level"  # We comment this out to avoid syntax errors

if True:
    yield "module_level"  # Invalid: module level
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_generator_expression() {
        let source = r#"
# Generator expressions create their own scope
# This should be valid
gen = (yield x for x in range(10))

# But yield outside generator expression is invalid
if condition:
    yield "outside"  # Invalid
"#;
        let tree = parse_python(source);
        let rule = YieldOutsideFunction;
        let violations = rule.check(&tree, source);

        // The generator expression should be valid, but the module-level yield should be invalid
        assert_eq!(violations.len(), 1);
    }
}