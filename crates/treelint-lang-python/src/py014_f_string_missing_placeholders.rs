use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct FStringMissingPlaceholders;

impl Rule for FStringMissingPlaceholders {
    fn id(&self) -> &'static str {
        "PY014"
    }

    fn name(&self) -> &'static str {
        "f-string-missing-placeholders"
    }

    fn description(&self) -> &'static str {
        "F-strings without placeholders should be regular strings"
    }

    fn explanation(&self) -> &'static str {
        "F-strings without any placeholders ({}) are unnecessary. Use regular strings instead for better performance and clarity."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl FStringMissingPlaceholders {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for string nodes
        if node.kind() == "string" {
            self.check_f_string(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_f_string(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        
        // Check if it's an f-string
        if !self.is_f_string(text) {
            return;
        }

        // Check if it has placeholders
        let has_placeholders = self.has_placeholders(node);
        
        if !has_placeholders {
            let start_point = node.start_position();
            
            violations.push(LintViolation {
                line: start_point.row + 1,
                column: start_point.column + 1,
                message: format!(
                    "F-string without placeholders. Remove the 'f' prefix to make it a regular string."
                ),
                lint_id: self.id().to_string(),
                lint_name: self.name().to_string(),
            });
        }
    }

    fn is_f_string(&self, text: &str) -> bool {
        text.starts_with("f\"") || text.starts_with("f'") ||
        text.starts_with("F\"") || text.starts_with("F'") ||
        text.starts_with("fr\"") || text.starts_with("fr'") ||
        text.starts_with("Fr\"") || text.starts_with("Fr'") ||
        text.starts_with("fR\"") || text.starts_with("fR'") ||
        text.starts_with("FR\"") || text.starts_with("FR'") ||
        text.starts_with("rf\"") || text.starts_with("rf'") ||
        text.starts_with("Rf\"") || text.starts_with("Rf'") ||
        text.starts_with("rF\"") || text.starts_with("rF'") ||
        text.starts_with("RF\"") || text.starts_with("RF'")
    }

    fn has_placeholders(&self, node: Node) -> bool {
        // Check if the string node has any interpolation children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "interpolation" {
                    return true;
                }
                // Recursively check for nested interpolations
                if self.has_placeholders(child) {
                    return true;
                }
            }
        }
        false
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
    fn test_f_string_without_placeholders() {
        let source = r#"
# F-strings without placeholders
text1 = f"Hello, World!"
text2 = f'No variables here'
text3 = F"UPPERCASE F"
text4 = f"""Multiline
without placeholders"""
"#;
        let tree = parse_python(source);
        let rule = FStringMissingPlaceholders;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 4);
        assert!(violations.iter().all(|v| v.lint_id == "PY014"));
    }

    #[test]
    fn test_f_string_with_placeholders() {
        let source = r#"
# F-strings with placeholders - should not trigger
name = "Alice"
age = 30
greeting = f"Hello, {name}!"
info = f'Name: {name}, Age: {age}'
calc = f"Result: {2 + 2}"
formatted = f"{value:.2f}"
"#;
        let tree = parse_python(source);
        let rule = FStringMissingPlaceholders;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_regular_strings() {
        let source = r#"
# Regular strings - should not trigger
text1 = "Hello, World!"
text2 = 'No f-prefix'
text3 = """Multiline
regular string"""
raw = r"Raw string"
"#;
        let tree = parse_python(source);
        let rule = FStringMissingPlaceholders;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_f_string_with_braces_but_no_expressions() {
        let source = r#"
# F-strings with literal braces (escaped)
text1 = f"Use {{braces}} like this"
text2 = f"Empty {{}}"
"#;
        let tree = parse_python(source);
        let rule = FStringMissingPlaceholders;
        let violations = rule.check(&tree, source);

        // These should trigger since {{ and }} are escaped braces, not placeholders
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_raw_f_strings() {
        let source = r#"
# Raw f-strings
path1 = fr"C:\Users\{username}"  # With placeholder
path2 = rf"C:\Users\{username}"  # With placeholder
path3 = fr"C:\Users\name"         # Without placeholder
path4 = rf"C:\Users\name"         # Without placeholder
"#;
        let tree = parse_python(source);
        let rule = FStringMissingPlaceholders;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // path3 and path4
    }

    #[test]
    fn test_multiline_f_strings() {
        let source = r#"
# Multiline f-strings
name = "Alice"
multiline_with = f"""
Hello {name},
Welcome!
"""

multiline_without = f"""
Hello World,
Welcome!
"""
"#;
        let tree = parse_python(source);
        let rule = FStringMissingPlaceholders;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Only multiline_without
    }

    #[test]
    fn test_nested_expressions() {
        let source = r#"
# Complex expressions in f-strings
data = {"name": "Alice", "age": 30}
nested = f"Name: {data['name']}, Age: {data.get('age', 0)}"
formatted = f"Pi: {3.14159:.2f}"
expression = f"Sum: {sum([1, 2, 3])}"
"#;
        let tree = parse_python(source);
        let rule = FStringMissingPlaceholders;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }
}