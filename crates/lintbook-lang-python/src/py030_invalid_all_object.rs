use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct InvalidAllObject;

impl Rule for InvalidAllObject {
    fn id(&self) -> &'static str {
        "PY030"
    }

    fn name(&self) -> &'static str {
        "invalid-all-object"
    }

    fn description(&self) -> &'static str {
        "Invalid objects in __all__ list"
    }

    fn explanation(&self) -> &'static str {
        "__all__ should contain only string literals that specify the public API of the module. Other types like numbers, variables, or complex expressions should not be used."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl InvalidAllObject {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for assignments to __all__
        if node.kind() == "assignment" {
            if let Some(target) = self.find_all_assignment_target(node) {
                if self.is_all_identifier(target, source) {
                    if let Some(value) = self.find_assignment_value(node) {
                        self.check_all_value(value, source, violations);
                    }
                }
            }
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn find_all_assignment_target<'a>(&self, assignment_node: Node<'a>) -> Option<Node<'a>> {
        // Find the left side of the assignment (target)
        assignment_node.child(0)
    }

    fn find_assignment_value<'a>(&self, assignment_node: Node<'a>) -> Option<Node<'a>> {
        // Find the right side of the assignment (value)
        // Look for the assignment operator "=" and get the next node
        for i in 0..assignment_node.child_count() {
            if let Some(child) = assignment_node.child(i) {
                if child.kind() == "=" {
                    return assignment_node.child(i + 1);
                }
            }
        }
        None
    }

    fn is_all_identifier(&self, node: Node, source: &str) -> bool {
        node.kind() == "identifier"
            && node
                .utf8_text(source.as_bytes())
                .map_or(false, |text| text == "__all__")
    }

    fn check_all_value(&self, value_node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        match value_node.kind() {
            "list" => {
                self.check_list_contents(value_node, source, violations);
            }
            "tuple" => {
                // Tuples are also invalid for __all__, but we check their contents anyway
                self.report_violation(
                    value_node,
                    "Use list instead of tuple for __all__",
                    violations,
                );
                self.check_list_contents(value_node, source, violations);
            }
            "list_comprehension" => {
                // List comprehensions are invalid for __all__
                self.report_violation(
                    value_node,
                    "Use simple list of string literals instead of comprehension",
                    violations,
                );
            }
            _ => {
                // Any other type is invalid
                self.report_violation(value_node, "__all__ must be a list of strings", violations);
            }
        }
    }

    fn check_list_contents(
        &self,
        list_node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        for i in 0..list_node.child_count() {
            if let Some(child) = list_node.child(i) {
                self.check_list_element(child, source, violations);
            }
        }
    }

    fn check_list_element(&self, element: Node, source: &str, violations: &mut Vec<LintViolation>) {
        match element.kind() {
            "string" => {
                // Valid: string literal
            }
            "identifier" => {
                // Invalid: variable reference instead of string literal
                if let Ok(text) = element.utf8_text(source.as_bytes()) {
                    self.report_violation(
                        element,
                        &format!(
                            "Use string literal \"{}\" instead of variable reference",
                            text
                        ),
                        violations,
                    );
                }
            }
            "integer" | "float" => {
                // Invalid: numeric literal
                self.report_violation(element, "Use string literal instead of number", violations);
            }
            "true" | "false" => {
                // Invalid: boolean literal
                self.report_violation(element, "Use string literal instead of boolean", violations);
            }
            "none" => {
                // Invalid: None literal
                self.report_violation(element, "Use string literal instead of None", violations);
            }
            "formatted_string" => {
                // Invalid: f-string
                self.report_violation(
                    element,
                    "Use plain string literal instead of f-string",
                    violations,
                );
            }
            "binary_operator" => {
                // Invalid: string concatenation or other operations
                self.report_violation(
                    element,
                    "Use plain string literal instead of expression",
                    violations,
                );
            }
            "call" => {
                // Invalid: function call
                self.report_violation(
                    element,
                    "Use string literal instead of function call",
                    violations,
                );
            }
            "attribute" => {
                // Invalid: attribute access
                self.report_violation(
                    element,
                    "Use string literal instead of attribute access",
                    violations,
                );
            }
            "dictionary" => {
                // Invalid: dictionary
                self.report_violation(
                    element,
                    "Use string literal instead of dictionary",
                    violations,
                );
            }
            "list" => {
                // Invalid: nested list
                self.report_violation(
                    element,
                    "Use string literal instead of nested list",
                    violations,
                );
            }
            "tuple" => {
                // Invalid: tuple
                self.report_violation(element, "Use string literal instead of tuple", violations);
            }
            "set" => {
                // Invalid: set
                self.report_violation(element, "Use string literal instead of set", violations);
            }
            "list_comprehension" => {
                // Invalid: list comprehension (check its contents too)
                self.report_violation(
                    element,
                    "Use simple list of string literals instead of comprehension",
                    violations,
                );
            }
            "," | "[" | "]" | "(" | ")" => {
                // Skip punctuation
            }
            _ => {
                // Any other node type is invalid
                if let Ok(text) = element.utf8_text(source.as_bytes()) {
                    if !text.trim().is_empty() && text != "," {
                        self.report_violation(
                            element,
                            "Use string literal instead of complex expression",
                            violations,
                        );
                    }
                }
            }
        }
    }

    fn report_violation(&self, node: Node, message: &str, violations: &mut Vec<LintViolation>) {
        let start_point = node.start_position();
        violations.push(LintViolation {
            line: start_point.row + 1,
            column: start_point.column + 1,
            message: format!("Invalid __all__ entry: {}", message),
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
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_valid_all_declarations() {
        let source = r#"
__all__ = [
    "public_function",
    "PublicClass",
    "PUBLIC_CONSTANT",
]

__all__ = ["single_export"]

__all__ = []
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_invalid_number_in_all() {
        let source = r#"
__all__ = [
    "valid_string",
    123,
    "another_valid",
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("number"));
    }

    #[test]
    fn test_invalid_variable_reference() {
        let source = r#"
variable_name = "some_function"
__all__ = [
    "valid_string",
    variable_name,
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("variable_name"));
        assert!(violations[0].message.contains("string literal"));
    }

    #[test]
    fn test_invalid_none_and_boolean() {
        let source = r#"
__all__ = [
    "valid_string",
    None,
    True,
    False,
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3); // None, True, False
        assert!(violations.iter().any(|v| v.message.contains("None")));
        assert!(violations.iter().any(|v| v.message.contains("boolean")));
    }

    #[test]
    fn test_invalid_expressions() {
        let source = r#"
__all__ = [
    "valid_string",
    f"formatted_{var}",
    "prefix" + "suffix",
    func(),
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert!(violations.len() >= 2); // concatenation, function call (f-string might be detected as expression)
                                        // We should at least detect the function call and concatenation
        assert!(violations
            .iter()
            .any(|v| v.message.contains("function call")));
    }

    #[test]
    fn test_tuple_instead_of_list() {
        let source = r#"
__all__ = (
    "item1",
    "item2",
    42,
)
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert!(violations.len() >= 2); // tuple + number
        assert!(violations.iter().any(|v| v.message.contains("tuple")));
        assert!(violations.iter().any(|v| v.message.contains("number")));
    }

    #[test]
    fn test_complex_invalid_types() {
        let source = r#"
__all__ = [
    "valid_export",
    {},
    [],
    set(),
    len,
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert!(violations.len() >= 4); // dict, list, set, builtin
    }

    #[test]
    fn test_attribute_access() {
        let source = r#"
class Config:
    export_name = "some_export"

__all__ = [
    "valid_export",
    Config.export_name,
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("attribute access"));
    }

    #[test]
    fn test_imported_names() {
        let source = r#"
import os
from sys import path

__all__ = [
    "local_function",
    os,
    path,
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // os, path
        assert!(violations
            .iter()
            .all(|v| v.message.contains("variable reference")));
    }

    #[test]
    fn test_multiple_all_assignments() {
        let source = r#"
__all__ = ["first_definition"]

__all__ = [
    "second_definition",
    42,
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Only the number in second assignment
        assert!(violations[0].message.contains("number"));
    }

    #[test]
    fn test_list_comprehension() {
        let source = r#"
__all__ = [name for name in [
    "valid1",
    "valid2",
    123,
]]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert!(violations.len() >= 1);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("comprehension")));
    }

    #[test]
    fn test_nested_structures() {
        let source = r#"
__all__ = [
    "valid_name",
    ["nested", "list"],
    ("tuple", "values"),
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // nested list, tuple
        assert!(violations.iter().any(|v| v.message.contains("nested list")));
        assert!(violations.iter().any(|v| v.message.contains("tuple")));
    }

    #[test]
    fn test_builtin_types() {
        let source = r#"
__all__ = [
    "custom_function",
    str,
    int,
    list,
]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllObject;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 3); // str, int, list
        assert!(violations
            .iter()
            .all(|v| v.message.contains("variable reference")));
    }
}
