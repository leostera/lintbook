use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct UnnecessaryQuotedIdentifiers;

impl Rule for UnnecessaryQuotedIdentifiers {
    fn id(&self) -> &'static str {
        "SQL025"
    }

    fn name(&self) -> &'static str {
        "unnecessary-quoted-identifiers"
    }

    fn description(&self) -> &'static str {
        "Remove unnecessary quotes around identifiers"
    }

    fn explanation(&self) -> &'static str {
        "Identifiers that don't conflict with SQL keywords or contain special characters 
        don't need to be quoted. Remove unnecessary quotes for cleaner code."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_quoted_identifiers(tree.root_node(), source, &mut violations);

        violations
    }
}

impl UnnecessaryQuotedIdentifiers {
    fn check_quoted_identifiers(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            self.find_quoted_identifiers(line, line_idx, violations, node);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_quoted_identifiers(child, source, violations);
            }
        }
    }

    fn find_quoted_identifiers(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        let sql_keywords = [
            "select",
            "from",
            "where",
            "join",
            "inner",
            "left",
            "right",
            "outer",
            "full",
            "on",
            "group",
            "by",
            "order",
            "having",
            "union",
            "all",
            "distinct",
            "as",
            "insert",
            "into",
            "values",
            "update",
            "set",
            "delete",
            "create",
            "table",
            "alter",
            "drop",
            "index",
            "view",
            "database",
            "schema",
            "primary",
            "key",
            "foreign",
            "references",
            "constraint",
            "null",
            "not",
            "and",
            "or",
            "in",
            "between",
            "like",
            "exists",
            "case",
            "when",
            "then",
            "else",
            "end",
            "if",
            "user",
            "date",
            "time",
            "year",
            "month",
            "day",
            "check",
            "default",
            "unique",
        ];

        // Find quoted identifiers (double quotes, backticks, or square brackets)
        self.check_quote_type(line, line_idx, violations, node, '"', &sql_keywords);
        self.check_quote_type(line, line_idx, violations, node, '`', &sql_keywords);
        self.check_bracket_identifiers(line, line_idx, violations, node, &sql_keywords);
    }

    fn check_quote_type(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
        quote_char: char,
        keywords: &[&str],
    ) {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == quote_char {
                // Find the closing quote
                let mut j = i + 1;
                let mut found_end = false;
                let mut identifier = String::new();

                while j < chars.len() {
                    if chars[j] == quote_char {
                        found_end = true;
                        break;
                    }
                    identifier.push(chars[j]);
                    j += 1;
                }

                if found_end && self.is_unnecessarily_quoted(&identifier, keywords) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + i + 1,
                        message: format!(
                            "Identifier '{}' doesn't need quotes. Remove {} quotes for cleaner code",
                            identifier, quote_char
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }

                i = j + 1;
            } else {
                i += 1;
            }
        }
    }

    fn check_bracket_identifiers(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
        keywords: &[&str],
    ) {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '[' {
                // Find the closing bracket
                let mut j = i + 1;
                let mut found_end = false;
                let mut identifier = String::new();

                while j < chars.len() {
                    if chars[j] == ']' {
                        found_end = true;
                        break;
                    }
                    identifier.push(chars[j]);
                    j += 1;
                }

                if found_end && self.is_unnecessarily_quoted(&identifier, keywords) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + i + 1,
                        message: format!(
                            "Identifier '{}' doesn't need square brackets. Remove brackets for cleaner code",
                            identifier
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }

                i = j + 1;
            } else {
                i += 1;
            }
        }
    }

    fn is_unnecessarily_quoted(&self, identifier: &str, keywords: &[&str]) -> bool {
        // Check if identifier needs quotes

        // If it's a keyword, quotes are necessary
        if keywords.contains(&identifier.to_lowercase().as_str()) {
            return false;
        }

        // If it contains spaces or special characters, quotes are necessary
        if identifier.contains(' ')
            || identifier.contains('-')
            || identifier.contains('.')
            || identifier.chars().any(|c| !c.is_alphanumeric() && c != '_')
        {
            return false;
        }

        // If it starts with a number, quotes might be necessary
        if identifier.chars().next().unwrap_or('a').is_numeric() {
            return false;
        }

        // If we get here, the quotes are unnecessary
        true
    }
}
