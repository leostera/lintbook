use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct BreakOutsideLoop;

impl Rule for BreakOutsideLoop {
    fn id(&self) -> &'static str {
        "PY020"
    }

    fn name(&self) -> &'static str {
        "break-outside-loop"
    }

    fn description(&self) -> &'static str {
        "Break statement outside loop"
    }

    fn explanation(&self) -> &'static str {
        "Break statements can only be used inside loops (for/while). Using break outside a loop is a syntax error."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations, false);
        violations
    }
}

impl BreakOutsideLoop {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>, in_loop: bool) {
        match node.kind() {
            "break_statement" => {
                if !in_loop {
                    let start_point = node.start_position();
                    
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: "Break statement outside loop. Break can only be used inside for or while loops.".to_string(),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            },
            "for_statement" | "while_statement" => {
                // Recursively visit children with in_loop=true
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, true);
                    }
                }
                return; // Don't fall through to general recursion
            },
            "function_definition" | "async_function_definition" => {
                // Functions create a new scope - break statements inside are not in the outer loop
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, false);
                    }
                }
                return; // Don't fall through to general recursion
            },
            "class_definition" => {
                // Classes create a new scope - break statements inside are not in the outer loop
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, false);
                    }
                }
                return; // Don't fall through to general recursion
            },
            _ => {}
        }

        // Recursively visit child nodes with current loop state
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations, in_loop);
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
    fn test_break_outside_loop() {
        let source = r#"
# Break outside any loop
def my_function():
    if condition:
        break
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY020");
        assert!(violations[0].message.contains("outside loop"));
    }

    #[test]
    fn test_break_in_for_loop() {
        let source = r#"
# Correct usage in for loop
for item in items:
    if condition:
        break
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_break_in_while_loop() {
        let source = r#"
# Correct usage in while loop
while condition:
    if should_exit:
        break
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_break_in_nested_loops() {
        let source = r#"
# Nested loops - both breaks are valid
for outer in outer_items:
    for inner in inner_items:
        if inner_condition:
            break  # Valid: in inner loop
    if outer_condition:
        break  # Valid: in outer loop
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_break_after_loop() {
        let source = r#"
# Break after loop ends
for item in items:
    process(item)

# This break is outside the loop
if error:
    break
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_break_in_function_not_in_loop() {
        let source = r#"
# Break in function but outside any loop
def process_data():
    try:
        data = load_data()
        if not data:
            break  # Invalid: function scope, no loop
    except Exception:
        break  # Invalid: exception handler, no loop
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_break_in_loop_inside_function() {
        let source = r#"
# Break in loop inside function - valid
def process_items():
    for item in items:
        if item.is_invalid():
            break  # Valid: inside loop within function
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_break_in_class_method() {
        let source = r#"
class Processor:
    def run(self):
        # This break is outside any loop
        if self.should_stop:
            break
            
    def process_items(self):
        # This break is inside a loop - valid
        for item in self.items:
            if item.done():
                break
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_break_at_module_level() {
        let source = r#"
# Module level break - invalid
if global_condition:
    break
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_multiple_breaks_outside_loop() {
        let source = r#"
def function_with_multiple_breaks():
    if condition1:
        break  # Invalid
    
    try:
        risky_operation()
    except:
        break  # Invalid
    finally:
        break  # Invalid
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY020"));
    }

    #[test]
    fn test_break_in_nested_function() {
        let source = r#"
def outer():
    for item in items:
        def inner():
            # This break is in inner function scope, not in the loop
            if condition:
                break  # Invalid: function boundary
        inner()
        
        # This break is valid - in the loop
        if item.done():
            break  # Valid
"#;
        let tree = parse_python(source);
        let rule = BreakOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }
}