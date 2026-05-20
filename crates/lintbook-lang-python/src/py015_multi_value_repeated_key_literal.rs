use std::collections::HashSet;
use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct MultiValueRepeatedKeyLiteral;

impl Rule for MultiValueRepeatedKeyLiteral {
    fn id(&self) -> &'static str {
        "PY015"
    }

    fn name(&self) -> &'static str {
        "multi-value-repeated-key-literal"
    }

    fn description(&self) -> &'static str {
        "Dictionary contains duplicate keys"
    }

    fn explanation(&self) -> &'static str {
        "Dictionary literals with duplicate keys will overwrite earlier values. This is likely a mistake and can lead to unexpected behavior."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl MultiValueRepeatedKeyLiteral {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for dictionary nodes
        if node.kind() == "dictionary" {
            self.check_dictionary(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_dictionary(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut seen_keys = HashSet::new();
        let mut duplicate_keys = Vec::new();

        // Iterate through dictionary pairs
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "pair" {
                    if let Some(key_node) = child.child_by_field_name("key") {
                        // Get the key value
                        let key_value = self.get_key_value(key_node, source);

                        if let Some(key) = key_value {
                            if !seen_keys.insert(key.clone()) {
                                // This key was already seen
                                duplicate_keys.push((key_node, key));
                            }
                        }
                    }
                }
            }
        }

        // Report violations for duplicate keys
        for (key_node, key_str) in duplicate_keys {
            let start_point = key_node.start_position();

            violations.push(LintViolation {
                line: start_point.row + 1,
                column: start_point.column + 1,
                message: format!(
                    "Dictionary contains duplicate key: {}. The earlier value will be overwritten.",
                    key_str
                ),
                lint_id: self.id().to_string(),
                lint_name: self.name().to_string(),
            });
        }
    }

    fn get_key_value(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "string" => {
                // Get string literal value
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                Some(text.to_string())
            }
            "integer" | "float" => {
                // Get numeric literal value
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                Some(text.to_string())
            }
            "true" | "false" | "none" => {
                // Get boolean/None literal value
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                Some(text.to_string())
            }
            "identifier" => {
                // For identifiers, we'll use the name itself
                // Note: This won't catch cases where different identifiers have the same value
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                Some(format!("identifier:{}", text))
            }
            _ => None,
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
    fn test_duplicate_string_keys() {
        let source = r#"
# Dictionary with duplicate string keys
data = {
    "name": "Alice",
    "age": 30,
    "name": "Bob",  # Duplicate key
    "city": "NYC"
}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY015");
        assert!(violations[0].message.contains("\"name\""));
    }

    #[test]
    fn test_duplicate_numeric_keys() {
        let source = r#"
# Dictionary with duplicate numeric keys
scores = {
    1: "first",
    2: "second",
    1: "first_again",  # Duplicate key
    3: "third"
}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY015");
        assert!(violations[0].message.contains("1"));
    }

    #[test]
    fn test_multiple_duplicates() {
        let source = r#"
# Multiple duplicate keys
config = {
    "debug": True,
    "verbose": False,
    "debug": False,    # Duplicate
    "port": 8080,
    "verbose": True,   # Duplicate
    "host": "localhost"
}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY015"));
    }

    #[test]
    fn test_no_duplicates() {
        let source = r#"
# Dictionary without duplicates
person = {
    "name": "Alice",
    "age": 30,
    "city": "NYC",
    "email": "alice@example.com"
}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_nested_dictionaries() {
        let source = r#"
# Nested dictionaries - only check duplicates within same dict
data = {
    "user": {
        "name": "Alice",
        "id": 123
    },
    "admin": {
        "name": "Bob",  # Not a duplicate (different dict)
        "id": 456
    },
    "user": {  # Duplicate key in outer dict
        "name": "Charlie"
    }
}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("\"user\""));
    }

    #[test]
    fn test_mixed_key_types() {
        let source = r#"
# Mixed key types
mixed = {
    "1": "string one",
    1: "number one",     # Different from "1"
    True: "boolean",     # In Python, True == 1, but we treat as different
    None: "none value",
    "1": "duplicate"     # Duplicate of first key
}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("\"1\""));
    }

    #[test]
    fn test_empty_dictionary() {
        let source = r#"
# Empty dictionary
empty = {}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_single_key_dictionary() {
        let source = r#"
# Single key dictionary
single = {"only": "one"}
"#;
        let tree = parse_python(source);
        let rule = MultiValueRepeatedKeyLiteral;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }
}
