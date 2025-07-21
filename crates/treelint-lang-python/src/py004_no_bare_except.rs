use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct NoBareExcept;

impl Rule for NoBareExcept {
    fn id(&self) -> &'static str {
        "PY004"
    }

    fn name(&self) -> &'static str {
        "no-bare-except"
    }

    fn description(&self) -> &'static str {
        "Disallow bare except clauses"
    }

    fn explanation(&self) -> &'static str {
        "Bare except clauses catch all exceptions including SystemExit and KeyboardInterrupt. Be explicit about which exceptions you want to catch."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root = tree.root_node();
        check_node(&root, source, &mut violations);
        violations
    }
}

fn check_node(node: &Node, source: &str, violations: &mut Vec<LintViolation>) {
    // Look for except_clause nodes
    if node.kind() == "except_clause" {
        // A bare except clause only has the "except" keyword and colon, no exception type
        // Check the text content to see if it's just "except:"
        let clause_text = &source[node.byte_range()];
        let trimmed = clause_text.trim();

        // Check if it's a bare except by looking for the absence of exception types
        // between "except" and ":"
        if trimmed.starts_with("except") && trimmed.contains(':') {
            let between = trimmed
                .strip_prefix("except")
                .unwrap()
                .split(':')
                .next()
                .unwrap()
                .trim();

            if between.is_empty() {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + 1,
                    column: start_pos.column + 1,
                    message: "Bare except clause catches all exceptions".to_string(),
                    lint_name: "no-bare-except".to_string(),
                    lint_id: "PY004".to_string(),
                });
            }
        }
    }

    // Recurse through children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        check_node(&child, source, violations);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn check_python_code(code: &str) -> Vec<LintViolation> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        let rule = NoBareExcept;
        rule.check(&tree, code)
    }

    #[test]
    fn test_bare_except_detected() {
        let code = r#"
try:
    risky_operation()
except:
    print("Something went wrong")
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 4);
        assert_eq!(
            violations[0].message,
            "Bare except clause catches all exceptions"
        );
    }

    #[test]
    fn test_specific_exception_ok() {
        let code = r#"
try:
    risky_operation()
except ValueError:
    print("Invalid value")
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_exceptions_ok() {
        let code = r#"
try:
    risky_operation()
except (ValueError, TypeError):
    print("Type or value error")
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_exception_as_ok() {
        let code = r#"
try:
    risky_operation()
except Exception as e:
    print(f"Error: {e}")
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_except_mixed() {
        let code = r#"
try:
    risky_operation()
except ValueError:
    print("Value error")
except:
    print("Any other error")
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 6);
    }
}
