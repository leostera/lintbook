use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct NoOsGetenv;

impl Rule for NoOsGetenv {
    fn id(&self) -> &'static str {
        "PY003"
    }

    fn name(&self) -> &'static str {
        "no-os-getenv"
    }

    fn description(&self) -> &'static str {
        "Disallow os.getenv usage"
    }

    fn explanation(&self) -> &'static str {
        "Use application-specific config module functions instead of os.getenv for better type safety and centralized configuration management."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root = tree.root_node();
        check_node(&root, source, &mut violations);
        violations
    }
}

fn check_node(node: &Node, source: &str, violations: &mut Vec<LintViolation>) {
    // Look for os.getenv() calls
    if node.kind() == "call" {
        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "attribute" {
                if let Some(object_node) = function_node.child_by_field_name("object") {
                    if let Some(attr_node) = function_node.child_by_field_name("attribute") {
                        let object_text = &source[object_node.byte_range()];
                        let attr_text = &source[attr_node.byte_range()];

                        if object_text == "os" && attr_text == "getenv" {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + 1,
                                column: start_pos.column + 1,
                                message: "Use config module instead of os.getenv".to_string(),
                                lint_name: "no-os-getenv".to_string(),
                                lint_id: "PY003".to_string(),
                            });
                        }
                    }
                }
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
        let rule = NoOsGetenv;
        rule.check(&tree, code)
    }

    #[test]
    fn test_os_getenv_detected() {
        let code = r#"
import os
port = os.getenv('PORT')
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert_eq!(
            violations[0].message,
            "Use config module instead of os.getenv"
        );
    }

    #[test]
    fn test_os_getenv_with_default() {
        let code = r#"
import os
port = os.getenv('PORT', '8080')
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_no_violation() {
        let code = r#"
import config
port = config.port()
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_environ_access_not_detected() {
        let code = r#"
import os
# This lint only catches os.getenv, not os.environ
port = os.environ.get('PORT')
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 0);
    }
}
