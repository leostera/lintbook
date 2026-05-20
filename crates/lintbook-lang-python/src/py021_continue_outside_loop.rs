use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct ContinueOutsideLoop;

impl Rule for ContinueOutsideLoop {
    fn id(&self) -> &'static str {
        "PY021"
    }

    fn name(&self) -> &'static str {
        "continue-outside-loop"
    }

    fn description(&self) -> &'static str {
        "Continue statement outside loop"
    }

    fn explanation(&self) -> &'static str {
        "Continue statements can only be used inside loops (for/while). Using continue outside a loop is a syntax error."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations, false);
        violations
    }
}

impl ContinueOutsideLoop {
    fn visit_node(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
        in_loop: bool,
    ) {
        match node.kind() {
            "continue_statement" => {
                if !in_loop {
                    let start_point = node.start_position();

                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: "Continue statement outside loop. Continue can only be used inside for or while loops.".to_string(),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            }
            "for_statement" | "while_statement" => {
                // Recursively visit children with in_loop=true
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, true);
                    }
                }
                return; // Don't fall through to general recursion
            }
            "function_definition" | "async_function_definition" => {
                // Functions create a new scope - continue statements inside are not in the outer loop
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, false);
                    }
                }
                return; // Don't fall through to general recursion
            }
            "class_definition" => {
                // Classes create a new scope - continue statements inside are not in the outer loop
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, false);
                    }
                }
                return; // Don't fall through to general recursion
            }
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
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_continue_outside_loop() {
        let source = r#"
# Continue outside any loop
def my_function():
    if condition:
        continue
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY021");
        assert!(violations[0].message.contains("outside loop"));
    }

    #[test]
    fn test_continue_in_for_loop() {
        let source = r#"
# Correct usage in for loop
for item in items:
    if should_skip(item):
        continue
    process(item)
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_continue_in_while_loop() {
        let source = r#"
# Correct usage in while loop
while condition:
    if should_skip:
        continue
    process_item()
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_continue_in_nested_loops() {
        let source = r#"
# Nested loops - both continues are valid
for outer in outer_items:
    if outer.skip():
        continue  # Valid: in outer loop
    for inner in inner_items:
        if inner.skip():
            continue  # Valid: in inner loop
        process(outer, inner)
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_continue_after_loop() {
        let source = r#"
# Continue after loop ends
for item in items:
    process(item)

# This continue is outside the loop
if error:
    continue
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_continue_in_function_not_in_loop() {
        let source = r#"
# Continue in function but outside any loop
def process_data():
    try:
        data = load_data()
        if not data:
            continue  # Invalid: function scope, no loop
    except Exception:
        continue  # Invalid: exception handler, no loop
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_continue_in_loop_inside_function() {
        let source = r#"
# Continue in loop inside function - valid
def process_items():
    for item in items:
        if item.is_invalid():
            continue  # Valid: inside loop within function
        process_valid_item(item)
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_continue_in_class_method() {
        let source = r#"
class Processor:
    def run(self):
        # This continue is outside any loop
        if self.should_skip:
            continue

    def process_items(self):
        # This continue is inside a loop - valid
        for item in self.items:
            if item.skip():
                continue
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_continue_at_module_level() {
        let source = r#"
# Module level continue - invalid
if global_condition:
    continue
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_multiple_continues_outside_loop() {
        let source = r#"
def function_with_multiple_continues():
    if condition1:
        continue  # Invalid

    try:
        risky_operation()
    except:
        continue  # Invalid
    finally:
        continue  # Invalid
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.lint_id == "PY021"));
    }

    #[test]
    fn test_continue_in_nested_function() {
        let source = r#"
def outer():
    for item in items:
        def inner():
            # This continue is in inner function scope, not in the loop
            if condition:
                continue  # Invalid: function boundary
        inner()

        # This continue is valid - in the loop
        if item.skip():
            continue  # Valid
"#;
        let tree = parse_python(source);
        let rule = ContinueOutsideLoop;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }
}
