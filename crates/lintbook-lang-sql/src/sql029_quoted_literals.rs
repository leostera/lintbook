use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct QuotedLiterals;

impl Rule for QuotedLiterals {
    fn id(&self) -> &'static str {
        "SQL029"
    }

    fn name(&self) -> &'static str {
        "quoted-literals"
    }

    fn description(&self) -> &'static str {
        "Use consistent quote style for string literals"
    }

    fn explanation(&self) -> &'static str {
        "String literals should use single quotes ('') as per SQL standard.
        Double quotes (\"\") should be reserved for identifiers (when needed).
        This improves SQL portability across different database systems."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_quote_usage(tree.root_node(), source, &mut violations);

        violations
    }
}

impl QuotedLiterals {
    fn check_quote_usage(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                // Check for double-quoted strings (not identifiers)
                if chars[i] == '"' {
                    let string_content = self.extract_quoted_string(&chars, i, '"');
                    if let Some((content, end_pos)) = string_content {
                        // Check if this looks like a string literal (not an identifier)
                        if self.is_likely_string_literal(&content, line, i) {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + i + 1,
                                message: format!(
                                    "String literal \"{}\" should use single quotes: '{}'",
                                    content, content
                                ),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
                        i = end_pos;
                    }
                }
                i += 1;
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_quote_usage(child, source, violations);
            }
        }
    }

    fn extract_quoted_string(
        &self,
        chars: &[char],
        start: usize,
        quote_char: char,
    ) -> Option<(String, usize)> {
        let mut content = String::new();
        let mut i = start + 1;

        while i < chars.len() {
            if chars[i] == quote_char {
                // Check for escaped quote
                if i + 1 < chars.len() && chars[i + 1] == quote_char {
                    content.push(quote_char);
                    i += 2;
                } else {
                    return Some((content, i + 1));
                }
            } else {
                content.push(chars[i]);
                i += 1;
            }
        }

        None // Unclosed quote
    }

    fn is_likely_string_literal(&self, content: &str, line: &str, position: usize) -> bool {
        // Check context to determine if this is a string literal or identifier

        // Get text before the quote
        let before = if position > 0 { &line[..position] } else { "" };

        let before_trimmed = before.trim_end();
        let before_lower = before_trimmed.to_lowercase();

        // Common patterns where double quotes are used for identifiers
        if before_lower.ends_with(" as")
            || before_lower.ends_with("from")
            || before_lower.ends_with("join")
            || before_lower.ends_with("table")
            || before_lower.ends_with("column")
            || before_lower.ends_with("index")
            || before_lower.ends_with("create")
            || before_lower.ends_with("alter")
            || before_lower.ends_with("drop")
        {
            return false;
        }

        // Common patterns where we expect string literals
        if before_lower.ends_with("=")
            || before_lower.ends_with("!=")
            || before_lower.ends_with("<>")
            || before_lower.ends_with("like")
            || before_lower.ends_with("in (")
            || before_lower.ends_with("values (")
            || before_lower.ends_with(",")
        {
            return true;
        }

        // Check content - identifiers typically don't have spaces or special chars
        if content.contains(' ')
            || content.contains('\n')
            || content.contains('\t')
            || content.contains('!')
            || content.contains('?')
            || content.contains('.')
                && !content
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '.' || c == '_')
        {
            return true;
        }

        // Default to identifier if unsure
        false
    }
}
