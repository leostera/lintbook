use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct InvalidAllFormat;

impl Rule for InvalidAllFormat {
    fn id(&self) -> &'static str {
        "PY031"
    }

    fn name(&self) -> &'static str {
        "invalid-all-format"
    }

    fn description(&self) -> &'static str {
        "Invalid format for __all__ declaration"
    }

    fn explanation(&self) -> &'static str {
        "__all__ should only be declared at module level as a simple assignment to a list. It should not be used inside functions, classes, conditionals, or modified dynamically."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations, 0);
        violations
    }
}

impl InvalidAllFormat {
    fn visit_node(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
        depth: usize,
    ) {
        let is_scope_creating = matches!(
            node.kind(),
            "function_definition"
                | "async_function_definition"
                | "class_definition"
                | "if_statement"
                | "for_statement"
                | "while_statement"
                | "try_statement"
                | "with_statement"
                | "match_statement"
                | "except_clause"
                | "finally_clause"
                | "else_clause"
        );

        match node.kind() {
            "assignment" => {
                if let Some(target) = self.find_all_assignment_target(node) {
                    if self.is_all_identifier(target, source) {
                        // Check if this assignment is not at module level
                        // At module level, depth should be 0 (module root)
                        if depth > 0 {
                            self.report_violation(
                                node,
                                "__all__ should only be assigned at module level",
                                violations,
                            );
                        }
                    }
                }
            }
            "expression_statement" => {
                // Check for dynamic modifications like __all__.append()
                if let Some(call_node) = self.find_child_by_kind(node, "call") {
                    if self.is_all_method_call(call_node, source) {
                        self.report_violation(
                            call_node,
                            "__all__ should not be modified dynamically",
                            violations,
                        );
                    }
                }
            }
            "augmented_assignment" => {
                // Check for += style assignments
                if let Some(target) = node.child(0) {
                    if self.is_all_identifier(target, source) {
                        self.report_violation(
                            node,
                            "__all__ should not use augmented assignment operators",
                            violations,
                        );
                    }
                }
            }
            "named_expression" => {
                // Check for walrus operator with __all__
                if let Some(target) = node.child(0) {
                    if self.is_all_identifier(target, source) {
                        self.report_violation(
                            node,
                            "__all__ should not be used with walrus operator",
                            violations,
                        );
                    }
                }
            }
            _ => {}
        }

        // Continue traversal for child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let new_depth = if is_scope_creating { depth + 1 } else { depth };
                self.visit_node(child, source, violations, new_depth);
            }
        }
    }

    fn find_all_assignment_target<'a>(&self, assignment_node: Node<'a>) -> Option<Node<'a>> {
        assignment_node.child(0)
    }

    fn is_all_identifier(&self, node: Node, source: &str) -> bool {
        node.kind() == "identifier"
            && node
                .utf8_text(source.as_bytes())
                .map_or(false, |text| text == "__all__")
    }

    fn find_child_by_kind<'a>(&self, node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == kind {
                    return Some(child);
                }
            }
        }
        None
    }

    fn is_all_method_call(&self, call_node: Node, source: &str) -> bool {
        // Check if this is a method call on __all__ (like __all__.append())
        if let Some(attribute_node) = self.find_child_by_kind(call_node, "attribute") {
            if let Some(object_node) = attribute_node.child(0) {
                if self.is_all_identifier(object_node, source) {
                    return true;
                }
            }
        }
        false
    }

    fn report_violation(&self, node: Node, message: &str, violations: &mut Vec<LintViolation>) {
        let start_point = node.start_position();
        violations.push(LintViolation {
            line: start_point.row + 1,
            column: start_point.column + 1,
            message: format!("Invalid __all__ format: {}", message),
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
    fn test_valid_module_level_all() {
        let source = r#"
__all__ = [
    "public_function",
    "PublicClass",
]

__all__ = ["single_export"]

__all__ = []
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_all_inside_function() {
        let source = r#"
def my_function():
    __all__ = ["should_not_be_here"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_all_inside_class() {
        let source = r#"
class MyClass:
    __all__ = ["class_level_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_all_inside_conditional() {
        let source = r#"
if some_condition:
    __all__ = ["conditional_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_all_dynamic_modification() {
        let source = r#"
__all__ = ["initial"]
__all__.append("dynamic")
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dynamically"));
    }

    #[test]
    fn test_all_augmented_assignment() {
        let source = r#"
__all__ = ["start"]
__all__ += ["added"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("augmented assignment"));
    }

    #[test]
    fn test_all_extend_method() {
        let source = r#"
__all__ = ["base"]
__all__.extend(["extended1", "extended2"])
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dynamically"));
    }

    #[test]
    fn test_all_in_try_except() {
        let source = r#"
try:
    __all__ = ["try_block_all"]
except Exception:
    __all__ = ["except_block_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both try and except blocks
        assert!(violations
            .iter()
            .all(|v| v.message.contains("module level")));
    }

    #[test]
    fn test_all_in_with_statement() {
        let source = r#"
with open("file.txt") as f:
    __all__ = ["with_block_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_all_in_loop() {
        let source = r#"
for item in items:
    __all__ = ["loop_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_all_in_nested_function() {
        let source = r#"
def outer():
    def inner():
        __all__ = ["nested_function"]
    return inner
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_all_method_inside_class() {
        let source = r#"
class BadClass:
    def method(self):
        __all__ = ["method_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_all_multiplication_assignment() {
        let source = r#"
__all__ = ["base"]
__all__ *= 2
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("augmented assignment"));
    }

    #[test]
    fn test_all_async_function() {
        let source = r#"
async def async_function():
    __all__ = ["async_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }

    #[test]
    fn test_multiple_valid_module_assignments() {
        let source = r#"
__all__ = ["first"]
# This is technically valid Python, though not recommended
__all__ = ["second"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        // Both assignments are at module level, so they should be valid format-wise
        // (Though logically questionable)
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_all_finally_block() {
        let source = r#"
try:
    pass
finally:
    __all__ = ["finally_all"]
"#;
        let tree = parse_python(source);
        let rule = InvalidAllFormat;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("module level"));
    }
}
