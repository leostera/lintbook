use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct InvalidEscapeSequence;

impl Rule for InvalidEscapeSequence {
    fn id(&self) -> &'static str {
        "PY012"
    }

    fn name(&self) -> &'static str {
        "invalid-escape-sequence"
    }

    fn description(&self) -> &'static str {
        "Use raw strings for regex patterns and escape sequences"
    }

    fn explanation(&self) -> &'static str {
        "Invalid escape sequences in string literals should be avoided. Use raw strings (r'...') for regex patterns and paths, or properly escape backslashes."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl InvalidEscapeSequence {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Check string literals
        if node.kind() == "string" {
            self.check_string_literal(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_string_literal(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        
        // Skip raw strings (r"...", r'...', etc.)
        if text.starts_with("r\"") || text.starts_with("r'") || 
           text.starts_with("R\"") || text.starts_with("R'") ||
           text.starts_with("br\"") || text.starts_with("br'") ||
           text.starts_with("rb\"") || text.starts_with("rb'") {
            return;
        }

        // Skip f-strings for now (they have their own parsing rules)
        if text.starts_with("f\"") || text.starts_with("f'") ||
           text.starts_with("F\"") || text.starts_with("F'") {
            return;
        }

        // Check for invalid escape sequences
        if self.has_invalid_escape_sequence(text) {
            let start_point = node.start_position();
            
            violations.push(LintViolation {
                line: start_point.row + 1,
                column: start_point.column + 1,
                message: format!(
                    "Invalid escape sequence in string literal. Use a raw string (r'...') or escape the backslash."
                ),
                lint_id: self.id().to_string(),
                lint_name: self.name().to_string(),
            });
        }
    }

    fn has_invalid_escape_sequence(&self, text: &str) -> bool {
        // Valid escape sequences in Python
        let valid_escapes = [
            "\\\\", "\\\'", "\\\"", "\\a", "\\b", "\\f", "\\n", "\\r", "\\t", "\\v",
            "\\0", "\\1", "\\2", "\\3", "\\4", "\\5", "\\6", "\\7", // Octal
            "\\x", "\\N", "\\u", "\\U" // Hex, Unicode name, Unicode
        ];

        let mut chars = text.chars().peekable();
        let mut in_escape = false;
        
        // Skip opening quote
        chars.next();
        
        while let Some(ch) = chars.next() {
            if in_escape {
                in_escape = false;
                
                // Check if this is a valid escape
                let escape_seq = format!("\\{}", ch);
                let is_valid = valid_escapes.iter().any(|&valid| {
                    escape_seq.starts_with(valid) || 
                    (valid == "\\x" && ch == 'x') ||
                    (valid == "\\N" && ch == 'N') ||
                    (valid == "\\u" && ch == 'u') ||
                    (valid == "\\U" && ch == 'U') ||
                    ch.is_ascii_digit() // Octal escape
                });
                
                if !is_valid && ch != '\'' && ch != '"' {
                    return true;
                }
            } else if ch == '\\' {
                in_escape = true;
            }
            
            // Stop before closing quote
            if chars.peek() == Some(&'"') || chars.peek() == Some(&'\'') {
                if chars.clone().count() == 1 {
                    break;
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
    fn test_invalid_escape_sequences() {
        let source = r#"
# Invalid escape sequences
path = "C:\Users\name\Documents"
regex = "\d+\.\d+"
text = "Line 1\nLine 2\tTabbed"
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // path and regex have invalid escapes
        assert!(violations.iter().all(|v| v.lint_id == "PY012"));
    }

    #[test]
    fn test_raw_strings() {
        let source = r#"
# Raw strings - should not trigger
path = r"C:\Users\name\Documents"
regex = r"\d+\.\d+"
pattern = r'(\w+)\s+(\d+)'
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_valid_escape_sequences() {
        let source = r#"
# Valid escape sequences - should not trigger
text = "Line 1\nLine 2\tTabbed"
quote = "He said \"Hello\""
backslash = "Path\\to\\file"
special = "\a\b\f\r\v"
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_unicode_escapes() {
        let source = r#"
# Valid unicode escapes
unicode1 = "\u0041"  # 'A'
unicode2 = "\U00000041"  # 'A'
unicode3 = "\N{LATIN CAPITAL LETTER A}"
hex_escape = "\x41"  # 'A'
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_octal_escapes() {
        let source = r#"
# Valid octal escapes
octal1 = "\101"  # 'A'
octal2 = "\0"    # Null
octal3 = "\777"  # Max octal
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_bytes_strings() {
        let source = r#"
# Bytes strings
data1 = b"bytes\x00data"
data2 = br"raw\bytes"
data3 = rb"raw\bytes"
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_f_strings() {
        let source = r#"
# F-strings are skipped for now
name = "world"
greeting = f"Hello\{name}"
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_common_invalid_patterns() {
        let source = r#"
# Common invalid patterns
windows_path = "C:\new\test\file.txt"
regex_digit = "\d+"
regex_word = "\w+"
latex = "\alpha \beta \gamma"
"#;
        let tree = parse_python(source);
        let rule = InvalidEscapeSequence;
        let violations = rule.check(&tree, source);

        assert!(violations.len() > 0);
        assert!(violations.iter().all(|v| v.lint_id == "PY012"));
    }
}