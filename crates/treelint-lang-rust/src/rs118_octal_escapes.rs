use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct OctalEscapes;

impl Rule for OctalEscapes {
    fn id(&self) -> &'static str {
        "RS118"
    }

    fn name(&self) -> &'static str {
        "octal-escapes"
    }

    fn description(&self) -> &'static str {
        "Checks for octal escape sequences in strings"
    }

    fn explanation(&self) -> &'static str {
        "Octal escape sequences like \\101 are less readable than hex \\x41 or Unicode \\u{41}. \
         Use more readable escape formats."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl OctalEscapes {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if matches!(node.kind(), "string_literal" | "char_literal") {
            let text = source[node.byte_range()].trim();
            
            if contains_octal_escapes(text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Octal escape sequences found - consider using hex (\\x) or Unicode (\\u{}) escapes for better readability".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn contains_octal_escapes(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() && next_ch != '8' && next_ch != '9' {
                    // Potential octal escape - check if it's 1-3 octal digits
                    let mut octal_count = 0;
                    let mut temp_chars = chars.clone();
                    
                    while let Some(&digit) = temp_chars.peek() {
                        if digit.is_ascii_digit() && digit < '8' && octal_count < 3 {
                            octal_count += 1;
                            temp_chars.next();
                        } else {
                            break;
                        }
                    }
                    
                    if octal_count > 0 {
                        return true;
                    }
                }
            }
        }
    }
    
    false
}