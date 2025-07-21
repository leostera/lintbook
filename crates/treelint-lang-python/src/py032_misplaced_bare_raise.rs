use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct MisplacedBareRaise;

impl Rule for MisplacedBareRaise {
    fn id(&self) -> &'static str {
        "PY032"
    }

    fn name(&self) -> &'static str {
        "misplaced-bare-raise"
    }

    fn description(&self) -> &'static str {
        "Misplaced bare raise statement"
    }

    fn explanation(&self) -> &'static str {
        "Bare 'raise' statements can only be used within exception handlers (except clauses) to re-raise the currently handled exception. Using 'raise' outside of exception handling context will cause a RuntimeError."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations, false);
        violations
    }
}

impl MisplacedBareRaise {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>, in_except_clause: bool) {
        match node.kind() {
            "raise_statement" => {
                if self.is_bare_raise(node, source) && !in_except_clause {
                    self.report_violation(node, violations);
                }
            },
            "except_clause" => {
                // Enter exception handler context
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.visit_node(child, source, violations, true);
                    }
                }
                return; // Don't continue with normal traversal
            },
            _ => {}
        }

        // Continue traversal for other nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                // Don't propagate exception context to nested try statements or other scopes
                let new_in_except = match node.kind() {
                    "try_statement" | "function_definition" | "async_function_definition" |
                    "class_definition" | "module" => false,
                    _ => in_except_clause
                };
                self.visit_node(child, source, violations, new_in_except);
            }
        }
    }

    fn is_bare_raise(&self, raise_node: Node, source: &str) -> bool {
        // A bare raise statement has no exception expression
        // Check if the raise statement only contains the "raise" keyword and no expression
        for i in 0..raise_node.child_count() {
            if let Some(child) = raise_node.child(i) {
                // If there's any child that's not just the "raise" keyword itself, it's not bare
                if child.kind() != "raise" && !child.utf8_text(source.as_bytes()).unwrap_or("").trim().is_empty() {
                    return false;
                }
            }
        }
        
        // Also check by examining the text content
        if let Ok(text) = raise_node.utf8_text(source.as_bytes()) {
            let trimmed = text.trim();
            return trimmed == "raise" || trimmed == "raise\n";
        }
        
        false
    }

    fn report_violation(&self, node: Node, violations: &mut Vec<LintViolation>) {
        let start_point = node.start_position();
        violations.push(LintViolation {
            line: start_point.row + 1,
            column: start_point.column + 1,
            message: "Bare 'raise' statement outside exception handler".to_string(),
            lint_id: self.id().to_string(),
            lint_name: self.name().to_string(),
        });
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
    fn test_bare_raise_at_module_level() {
        let source = r#"
raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_function() {
        let source = r#"
def bad_function():
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_except_clause() {
        let source = r#"
try:
    risky_operation()
except ValueError:
    log_error()
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // This should be valid
    }

    #[test]
    fn test_bare_raise_in_if_statement() {
        let source = r#"
if condition:
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_finally() {
        let source = r#"
try:
    operation()
finally:
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // finally blocks don't have exception context
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_nested_except() {
        let source = r#"
try:
    dangerous_operation()
except Exception:
    try:
        cleanup()
    except CleanupError:
        raise
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // Both raises are in except clauses
    }

    #[test]
    fn test_raise_with_exception() {
        let source = r#"
def function():
    raise ValueError("This is not a bare raise")
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // This is not a bare raise
    }

    #[test]
    fn test_bare_raise_after_except_block() {
        let source = r#"
try:
    operation()
except Exception:
    handle_error()

raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Outside the try/except structure
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_class_method() {
        let source = r#"
class BadClass:
    def method(self):
        raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_loop() {
        let source = r#"
for item in items:
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_multiple_except_clauses_with_bare_raise() {
        let source = r#"
try:
    operation()
except ValueError:
    raise
except TypeError:
    raise
except Exception:
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // All raises are in except clauses
    }

    #[test]
    fn test_bare_raise_in_async_function() {
        let source = r#"
async def bad_async():
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_except_inside_function() {
        let source = r#"
def error_handler():
    try:
        operation()
    except Exception:
        raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // Valid: raise in except clause within function
    }

    #[test]
    fn test_bare_raise_in_while_loop() {
        let source = r#"
while condition:
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_generator() {
        let source = r#"
def bad_generator():
    yield 1
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("outside exception handler"));
    }

    #[test]
    fn test_bare_raise_in_else_clause() {
        let source = r#"
try:
    operation()
except ValueError:
    handle_error()
else:
    raise
"#;
        let tree = parse_python(source);
        let rule = MisplacedBareRaise;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // else clause doesn't have exception context
        assert!(violations[0].message.contains("outside exception handler"));
    }
}