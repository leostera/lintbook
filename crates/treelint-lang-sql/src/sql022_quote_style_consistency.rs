use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct QuoteStyleConsistency;

impl Rule for QuoteStyleConsistency {
    fn id(&self) -> &'static str {
        "SQL022"
    }

    fn name(&self) -> &'static str {
        "quote-style-consistency"
    }

    fn description(&self) -> &'static str {
        "Use consistent quote style for string literals"
    }

    fn explanation(&self) -> &'static str {
        "String literals should use consistent quote style throughout the codebase. 
        Choose either single quotes (') or double quotes (\") and use consistently."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut quote_styles = Vec::new();

        // First pass: collect all quote styles
        self.collect_quote_styles(tree.root_node(), source, &mut quote_styles);

        // Determine preferred style (most common)
        if quote_styles.is_empty() {
            return violations;
        }

        let mut style_counts = std::collections::HashMap::new();
        for style in &quote_styles {
            *style_counts.entry(style.quote_char).or_insert(0) += 1;
        }

        let preferred_style = style_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&style, _)| style)
            .unwrap_or('\'');

        // Second pass: report inconsistencies
        for style_info in quote_styles {
            if style_info.quote_char != preferred_style {
                violations.push(LintViolation {
                    line: style_info.line,
                    column: style_info.column,
                    message: format!(
                        "Inconsistent quote style '{}'. Use '{}' for consistency",
                        style_info.quote_char, preferred_style
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        violations
    }
}

#[derive(Debug)]
struct QuoteStyle {
    quote_char: char,
    line: usize,
    column: usize,
}

impl QuoteStyleConsistency {
    fn collect_quote_styles(
        &self,
        node: tree_sitter::Node,
        source: &str,
        styles: &mut Vec<QuoteStyle>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            self.find_string_literals(line, line_idx, styles, node);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_quote_styles(child, source, styles);
            }
        }
    }

    fn find_string_literals(
        &self,
        line: &str,
        line_idx: usize,
        styles: &mut Vec<QuoteStyle>,
        node: tree_sitter::Node,
    ) {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // Found start of string literal
            if ch == '\'' || ch == '"' {
                // Skip if this quote is escaped
                if i > 0 && chars[i - 1] == '\\' {
                    i += 1;
                    continue;
                }

                // Find the end of the string literal
                let mut j = i + 1;
                let mut found_end = false;

                while j < chars.len() {
                    if chars[j] == ch {
                        // Check if this quote is escaped
                        if j > 0 && chars[j - 1] == '\\' {
                            j += 1;
                            continue;
                        }
                        found_end = true;
                        break;
                    }
                    j += 1;
                }

                // If we found a complete string literal, record its quote style
                if found_end {
                    let start_pos = node.start_position();
                    styles.push(QuoteStyle {
                        quote_char: ch,
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + i + 1,
                    });
                    i = j + 1; // Move past the closing quote
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
}
