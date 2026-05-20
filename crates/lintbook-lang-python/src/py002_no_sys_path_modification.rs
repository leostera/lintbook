use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct NoSysPathModification;

impl Rule for NoSysPathModification {
    fn id(&self) -> &'static str {
        "PY002"
    }

    fn name(&self) -> &'static str {
        "no-sys-path-modification"
    }

    fn description(&self) -> &'static str {
        "Disallow modification of sys.path"
    }

    fn explanation(&self) -> &'static str {
        "Modifying sys.path is discouraged as it can lead to unpredictable import behavior. Use proper package structure and relative imports instead."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root = tree.root_node();
        check_node(&root, source, &mut violations);
        violations
    }
}

fn check_node(node: &Node, source: &str, violations: &mut Vec<LintViolation>) {
    // Look for patterns like sys.path.append(), sys.path.insert(), etc.
    if node.kind() == "call" {
        if let Some(function_node) = node.child_by_field_name("function") {
            // Check if it's an attribute access on sys.path
            if function_node.kind() == "attribute" {
                if let Some(object_node) = function_node.child_by_field_name("object") {
                    if object_node.kind() == "attribute" {
                        if let Some(sys_node) = object_node.child_by_field_name("object") {
                            if let Some(path_node) = object_node.child_by_field_name("attribute") {
                                let sys_text = &source[sys_node.byte_range()];
                                let path_text = &source[path_node.byte_range()];

                                if sys_text == "sys" && path_text == "path" {
                                    if let Some(method_node) =
                                        function_node.child_by_field_name("attribute")
                                    {
                                        let method_text = &source[method_node.byte_range()];

                                        // Check for mutating methods
                                        if matches!(
                                            method_text,
                                            "append"
                                                | "insert"
                                                | "extend"
                                                | "remove"
                                                | "pop"
                                                | "clear"
                                        ) {
                                            let start_pos = node.start_position();
                                            violations.push(LintViolation {
                                                line: start_pos.row + 1,
                                                column: start_pos.column + 1,
                                                message: format!(
                                                    "sys.path.{} modifies sys.path",
                                                    method_text
                                                ),
                                                lint_name: "no-sys-path-modification".to_string(),
                                                lint_id: "PY002".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check for direct assignment to sys.path
    if node.kind() == "assignment" {
        if let Some(left_node) = node.child_by_field_name("left") {
            if left_node.kind() == "attribute" {
                if let Some(object_node) = left_node.child_by_field_name("object") {
                    if let Some(attr_node) = left_node.child_by_field_name("attribute") {
                        let object_text = &source[object_node.byte_range()];
                        let attr_text = &source[attr_node.byte_range()];

                        if object_text == "sys" && attr_text == "path" {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + 1,
                                column: start_pos.column + 1,
                                message: "Direct assignment to sys.path".to_string(),
                                lint_name: "no-sys-path-modification".to_string(),
                                lint_id: "PY002".to_string(),
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
        let rule = NoSysPathModification;
        rule.check(&tree, code)
    }

    #[test]
    fn test_sys_path_append() {
        let code = r#"
import sys
sys.path.append('/some/path')
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].message, "sys.path.append modifies sys.path");
    }

    #[test]
    fn test_sys_path_insert() {
        let code = r#"
import sys
sys.path.insert(0, '/some/path')
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].message, "sys.path.insert modifies sys.path");
    }

    #[test]
    fn test_sys_path_assignment() {
        let code = r#"
import sys
sys.path = ['/new/path']
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].message, "Direct assignment to sys.path");
    }

    #[test]
    fn test_no_violation() {
        let code = r#"
import sys
print(sys.path)  # Just reading, not modifying
"#;
        let violations = check_python_code(code);
        assert_eq!(violations.len(), 0);
    }
}
