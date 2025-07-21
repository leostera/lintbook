use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct LambdaAssignment;

impl Rule for LambdaAssignment {
    fn id(&self) -> &'static str {
        "PY010"
    }

    fn name(&self) -> &'static str {
        "lambda-assignment"
    }

    fn description(&self) -> &'static str {
        "Do not assign lambda expressions, use def"
    }

    fn explanation(&self) -> &'static str {
        "Assigning lambda expressions to variables is discouraged. Use a def statement instead for better readability and debugging capabilities."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl LambdaAssignment {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for assignment statements
        if node.kind() == "assignment" {
            self.check_lambda_assignment(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_lambda_assignment(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Check if the right side of assignment is a lambda
        if let Some(right) = node.child_by_field_name("right") {
            if right.kind() == "lambda" {
                // Get the left side (variable name) for better error message
                let var_name = if let Some(left) = node.child_by_field_name("left") {
                    left.utf8_text(source.as_bytes()).unwrap_or("variable")
                } else {
                    "variable"
                };
                
                let start_point = node.start_position();
                
                violations.push(LintViolation {
                    line: start_point.row + 1,
                    column: start_point.column + 1,
                    message: format!(
                        "Do not assign lambda expressions to variables. Use 'def {}(...)' instead.",
                        var_name
                    ),
                    lint_id: self.id().to_string(),
                    lint_name: self.name().to_string(),
                });
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
    fn test_lambda_assignment() {
        let source = r#"
add = lambda x, y: x + y
multiply = lambda a, b: a * b
"#;
        let tree = parse_python(source);
        let rule = LambdaAssignment;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY010"));
        assert!(violations[0].message.contains("def add"));
        assert!(violations[1].message.contains("def multiply"));
    }

    #[test]
    fn test_lambda_in_expression() {
        let source = r#"
# Lambda used directly (not assigned) - should not trigger
sorted_items = sorted(items, key=lambda x: x.name)
map_result = map(lambda x: x * 2, numbers)
"#;
        let tree = parse_python(source);
        let rule = LambdaAssignment;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_def_function() {
        let source = r#"
def add(x, y):
    return x + y

def multiply(a, b):
    return a * b
"#;
        let tree = parse_python(source);
        let rule = LambdaAssignment;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_regular_assignment() {
        let source = r#"
x = 5
name = "test"
result = add(1, 2)
"#;
        let tree = parse_python(source);
        let rule = LambdaAssignment;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_lambda_with_complex_body() {
        let source = r#"
process = lambda data: data.strip().lower() if data else ""
"#;
        let tree = parse_python(source);
        let rule = LambdaAssignment;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY010");
        assert!(violations[0].message.contains("def process"));
    }

    #[test]
    fn test_multiple_assignment() {
        let source = r#"
# Multiple assignment with lambda
func1 = func2 = lambda x: x * 2
"#;
        let tree = parse_python(source);
        let rule = LambdaAssignment;
        let violations = rule.check(&tree, source);

        // This should detect at least one violation
        assert!(violations.len() >= 1);
        assert!(violations.iter().all(|v| v.lint_id == "PY010"));
    }
}