use tree_sitter::{Node, Tree, TreeCursor};
use treelint_core::{LintViolation, Rule};

pub struct NoTryCatch;

impl Rule for NoTryCatch {
    fn id(&self) -> &'static str {
        "PY001"
    }

    fn name(&self) -> &'static str {
        "no-try-catch"
    }

    fn description(&self) -> &'static str {
        "Disallow try/except statements"
    }

    fn explanation(&self) -> &'static str {
        "Try/except blocks can hide errors and make debugging difficult. \
        Instead of catching exceptions, let them propagate naturally so they \
        can be handled at the appropriate level or fail fast with clear error messages. \
        This leads to more robust and maintainable code."
    }

    fn check(&self, tree: &Tree, _source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, &mut violations);
        violations
    }
}

impl NoTryCatch {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        violations: &mut Vec<LintViolation>,
    ) {
        if node.kind() == "try_statement" {
            let start_position = node.start_position();
            violations.push(LintViolation {
                line: start_position.row + 1,
                column: start_position.column + 1,
                message: self.description().to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        if cursor.goto_first_child() {
            loop {
                self.traverse_node(cursor.node(), cursor, violations);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_python_code(code: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_no_try_catch_detects_try_statement() {
        let code = r#"
try:
    result = risky_operation()
except Exception as e:
    print(f"Error: {e}")
"#;

        let tree = parse_python_code(code);
        let lint = NoTryCatch;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY001");
        assert_eq!(violations[0].lint_name, "no-try-catch");
        assert_eq!(violations[0].message, "Disallow try/except statements");
        assert_eq!(violations[0].line, 2);
    }

    #[test]
    fn test_no_try_catch_ignores_code_without_try() {
        let code = r#"
def safe_function():
    return "Hello, world!"
    
result = safe_function()
print(result)
"#;

        let tree = parse_python_code(code);
        let lint = NoTryCatch;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_try_catch_detects_multiple_try_statements() {
        let code = r#"
try:
    operation1()
except:
    pass

try:
    operation2()
except:
    pass
"#;

        let tree = parse_python_code(code);
        let lint = NoTryCatch;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 7);
    }
}
