use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct ContinueInFinally;

impl Rule for ContinueInFinally {
    fn id(&self) -> &'static str {
        "PY028"
    }

    fn name(&self) -> &'static str {
        "continue-in-finally"
    }

    fn description(&self) -> &'static str {
        "Continue not supported in finally"
    }

    fn explanation(&self) -> &'static str {
        "Continue statements are not allowed in finally blocks as they can lead to unexpected control flow behavior."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations, false);
        violations
    }
}

impl ContinueInFinally {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>, in_finally: bool) {
        match node.kind() {
            "continue_statement" => {
                if in_finally {
                    let start_point = node.start_position();
                    violations.push(LintViolation {
                        line: start_point.row + 1,
                        column: start_point.column + 1,
                        message: "Continue statement in finally block. Continue is not supported in finally blocks.".to_string(),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            },
            "finally_clause" => {
                // Recursively visit children with in_finally=true
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, true);
                    }
                }
                return; // Don't fall through to general recursion
            },
            "function_definition" | "async_function_definition" => {
                // Functions create a new scope - continue statements inside are in a different context
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, false);
                    }
                }
                return; // Don't fall through to general recursion
            },
            "class_definition" => {
                // Classes create a new scope - continue statements inside are in a different context
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, false);
                    }
                }
                return; // Don't fall through to general recursion
            },
            _ => {}
        }

        // Recursively visit child nodes with current finally state
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations, in_finally);
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
    fn test_continue_in_finally() {
        let source = r#"
def bad_function():
    for i in range(10):
        try:
            process_item(i)
        finally:
            if should_skip(i):
                continue  # Wrong: continue in finally
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY028");
        assert!(violations[0].message.contains("finally block"));
    }

    #[test]
    fn test_continue_in_try_block() {
        let source = r#"
def good_function():
    for i in range(10):
        try:
            if should_skip(i):
                continue  # Correct: continue in try block
            process_item(i)
        finally:
            cleanup()
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_continue_in_except_block() {
        let source = r#"
def good_function():
    for i in range(10):
        try:
            risky_operation(i)
        except SkipException:
            continue  # Correct: continue in except block
        finally:
            cleanup()
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_continues_in_finally() {
        let source = r#"
def bad_function():
    for i in range(10):
        try:
            process(i)
        finally:
            if condition1:
                continue  # Wrong
            elif condition2:
                continue  # Wrong
            else:
                cleanup()
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY028"));
    }

    #[test]
    fn test_continue_outside_finally() {
        let source = r#"
def good_function():
    for i in range(10):
        if should_skip(i):
            continue  # Correct: not in finally block
        
        try:
            process_item(i)
        finally:
            cleanup()
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_nested_try_finally() {
        let source = r#"
def nested_function():
    for outer in range(5):
        try:
            for inner in range(3):
                try:
                    risky_operation(outer, inner)
                finally:
                    if error_condition:
                        continue  # Wrong: continue in finally
        except Exception:
            pass
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_break_in_finally() {
        let source = r#"
def function_with_break():
    for i in range(10):
        try:
            process_item(i)
        finally:
            if critical_error():
                break  # This is break, not continue - different rule
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        // This rule only checks for continue, not break
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_return_in_finally() {
        let source = r#"
def function_with_return():
    for i in range(10):
        try:
            result = process_item(i)
        finally:
            if should_return_early():
                return result  # Return is allowed in finally
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_continue_in_nested_function() {
        let source = r#"
def outer_function():
    for i in range(10):
        def nested_function():
            try:
                helper_operation()
            finally:
                if helper_failed():
                    continue  # Wrong: continue in finally (even in nested function)
        
        nested_function()
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_async_continue_in_finally() {
        let source = r#"
async def async_function():
    for i in range(10):
        try:
            await async_process(i)
        finally:
            if should_skip():
                continue  # Wrong: continue in finally (async context)
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_while_loop_continue_in_finally() {
        let source = r#"
def while_loop_function():
    i = 0
    while i < 10:
        try:
            process_item(i)
            i += 1
        finally:
            if error_occurred():
                continue  # Wrong: continue in finally (while loop)
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_proper_finally_usage() {
        let source = r#"
def proper_function():
    for i in range(10):
        resource = None
        try:
            resource = acquire_resource()
            process_item(i, resource)
        except Exception as e:
            if can_skip_error(e):
                continue  # Correct: in except block
            else:
                raise
        finally:
            if resource:
                resource.cleanup()
            # No continue here - correct
"#;
        let tree = parse_python(source);
        let rule = ContinueInFinally;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }
}