use std::collections::HashMap;
use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct DuplicateBases;

impl Rule for DuplicateBases {
    fn id(&self) -> &'static str {
        "PY029"
    }

    fn name(&self) -> &'static str {
        "duplicate-bases"
    }

    fn description(&self) -> &'static str {
        "Duplicate bases in class definition"
    }

    fn explanation(&self) -> &'static str {
        "Class definitions should not have duplicate base classes. Duplicate inheritance can lead to unexpected behavior and is likely a mistake."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl DuplicateBases {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for class definitions
        if node.kind() == "class_definition" {
            self.check_class_definition(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_class_definition(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Find the argument list (inheritance list)
        if let Some(argument_list) = self.find_argument_list(node) {
            let mut base_classes: HashMap<String, Vec<Node>> = HashMap::new();

            // Collect all base classes
            self.collect_base_classes(argument_list, source, &mut base_classes);

            // Check for duplicates
            for (base_name, nodes) in &base_classes {
                if nodes.len() > 1 {
                    // Report each duplicate occurrence
                    for node in nodes {
                        let start_point = node.start_position();
                        violations.push(LintViolation {
                            line: start_point.row + 1,
                            column: start_point.column + 1,
                            message: format!(
                                "Duplicate base class '{}' in class definition.",
                                base_name
                            ),
                            lint_id: self.id().to_string(),
                            lint_name: self.name().to_string(),
                        });
                    }
                }
            }
        }
    }

    fn find_argument_list<'a>(&self, class_node: Node<'a>) -> Option<Node<'a>> {
        // Look for argument_list in class definition
        for i in 0..class_node.child_count() {
            if let Some(child) = class_node.child(i) {
                if child.kind() == "argument_list" {
                    return Some(child);
                }
            }
        }
        None
    }

    fn collect_base_classes<'a>(
        &self,
        argument_list: Node<'a>,
        source: &str,
        base_classes: &mut HashMap<String, Vec<Node<'a>>>,
    ) {
        for i in 0..argument_list.child_count() {
            if let Some(child) = argument_list.child(i) {
                if let Some(base_name) = self.extract_base_class_name(child, source) {
                    base_classes
                        .entry(base_name)
                        .or_insert_with(Vec::new)
                        .push(child);
                }
            }
        }
    }

    fn extract_base_class_name(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "identifier" => {
                // Simple identifier like BaseClass
                if let Ok(name) = node.utf8_text(source.as_bytes()) {
                    Some(name.to_string())
                } else {
                    None
                }
            }
            "attribute" => {
                // Attribute access like module.BaseClass
                if let Ok(name) = node.utf8_text(source.as_bytes()) {
                    Some(name.to_string())
                } else {
                    None
                }
            }
            "subscript" => {
                // Generic types like Generic[T]
                if let Ok(name) = node.utf8_text(source.as_bytes()) {
                    Some(name.to_string())
                } else {
                    None
                }
            }
            "keyword_argument" => {
                // Skip keyword arguments like metaclass=MetaClass
                None
            }
            "," => {
                // Skip commas
                None
            }
            "(" | ")" => {
                // Skip parentheses
                None
            }
            _ => {
                // For other node types, try to get the full text
                if let Ok(name) = node.utf8_text(source.as_bytes()) {
                    // Skip if it contains = (keyword argument)
                    if !name.contains('=') && !name.trim().is_empty() {
                        Some(name.trim().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
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
    fn test_duplicate_base_classes() {
        let source = r#"
class BadClass(BaseClass, BaseClass):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both occurrences reported
        assert!(violations.iter().all(|v| v.lint_id == "PY029"));
        assert!(violations.iter().all(|v| v.message.contains("BaseClass")));
    }

    #[test]
    fn test_multiple_duplicates() {
        let source = r#"
class AnotherBad(A, B, A):
    def method(self):
        pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both A occurrences
        assert!(violations.iter().all(|v| v.message.contains("'A'")));
    }

    #[test]
    fn test_no_duplicate_base_classes() {
        let source = r#"
class GoodClass(BaseClass):
    pass

class MultipleUnique(A, B, C):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_inheritance() {
        let source = r#"
class NoInheritance:
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_duplicates() {
        let source = r#"
class MultipleDuplicates(A, B, C, A, B):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        // Should detect duplicates for both A and B
        assert_eq!(violations.len(), 4); // 2 A's + 2 B's
        assert!(violations.iter().any(|v| v.message.contains("'A'")));
        assert!(violations.iter().any(|v| v.message.contains("'B'")));
    }

    #[test]
    fn test_attribute_access_duplicates() {
        let source = r#"
class AttributeAccess(package.module.Class, SomeOther, package.module.Class):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both package.module.Class occurrences
        assert!(violations
            .iter()
            .all(|v| v.message.contains("package.module.Class")));
    }

    #[test]
    fn test_nested_class() {
        let source = r#"
class Outer:
    class Inner(Base, Base):
        pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both Base occurrences in nested class
    }

    #[test]
    fn test_similar_but_different_names() {
        let source = r#"
class SimilarNames(BaseClass, BaseClass2):
    pass

class VersionedClasses(ClassV1, ClassV2):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // No duplicates, just similar names
    }

    #[test]
    fn test_simple_duplicate() {
        let source = r#"
class Simple(A, A):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.message.contains("'A'")));
    }

    #[test]
    fn test_three_way_duplicate() {
        let source = r#"
class Triple(X, Y, X, Z, Y):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        // Should detect duplicates for both X and Y
        assert_eq!(violations.len(), 4); // 2 X's + 2 Y's
    }

    #[test]
    fn test_multiline_class_definition() {
        let source = r#"
class FormattingIssue(
    BaseClass,
    AnotherBase,
    BaseClass
):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both BaseClass occurrences
    }

    #[test]
    fn test_mixed_with_object() {
        let source = r#"
class MixedDuplicates(object, MyBase, object):
    pass
"#;
        let tree = parse_python(source);
        let rule = DuplicateBases;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both object occurrences
        assert!(violations.iter().all(|v| v.message.contains("object")));
    }
}
