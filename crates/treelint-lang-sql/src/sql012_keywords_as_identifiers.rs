use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct KeywordsAsIdentifiers;

impl Rule for KeywordsAsIdentifiers {
    fn id(&self) -> &'static str {
        "SQL012"
    }

    fn name(&self) -> &'static str {
        "keywords-as-identifiers"
    }

    fn description(&self) -> &'static str {
        "Avoid using SQL keywords as unquoted identifiers"
    }

    fn explanation(&self) -> &'static str {
        "Using SQL keywords as table names, column names, or aliases can cause parsing errors 
        and confusion. Either use different names or quote them properly."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_identifier_usage(tree.root_node(), source, &mut violations);

        violations
    }
}

impl KeywordsAsIdentifiers {
    fn check_identifier_usage(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
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
            "count",
            "sum",
            "avg",
            "max",
            "min",
            "date",
            "time",
            "year",
            "month",
            "day",
            "user",
            "data",
            "table",
            "column",
            "row",
            "check",
            "default",
            "unique",
        ];

        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Check table names in CREATE TABLE
            if lower_line.contains("create table") {
                self.check_create_table_keywords(line, line_idx, violations, node, &sql_keywords);
            }

            // Check column names in CREATE TABLE column definitions
            if lower_line.contains("create table")
                || (line_idx > 0 && lines[line_idx - 1].to_lowercase().contains("create table"))
            {
                self.check_column_definitions(line, line_idx, violations, node, &sql_keywords);
            }

            // Check aliases in SELECT/FROM
            if lower_line.contains(" as ") {
                self.check_aliases(line, line_idx, violations, node, &sql_keywords);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_identifier_usage(child, source, violations);
            }
        }
    }

    fn check_create_table_keywords(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
        keywords: &[&str],
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();

        for (word_idx, word) in words.iter().enumerate() {
            if word_idx > 0 && words[word_idx - 1].to_lowercase() == "table" {
                let table_name = word
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .to_lowercase();
                if keywords.contains(&table_name.as_str()) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Table name '{}' is a SQL keyword. Consider using a different name or quoting it",
                            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn check_column_definitions(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
        keywords: &[&str],
    ) {
        // Look for column definitions (word followed by data type)
        let trimmed = line.trim();
        if trimmed.starts_with('(') || (!trimmed.contains("create table") && trimmed.contains('('))
        {
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            for (word_idx, word) in words.iter().enumerate() {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                let lower_word = clean_word.to_lowercase();

                // Check if this looks like a column name (followed by data type)
                if word_idx + 1 < words.len() {
                    let next_word = words[word_idx + 1].to_lowercase();
                    if matches!(
                        next_word.as_str(),
                        "int"
                            | "integer"
                            | "varchar"
                            | "text"
                            | "char"
                            | "date"
                            | "timestamp"
                            | "decimal"
                            | "float"
                            | "boolean"
                            | "bool"
                    ) {
                        if keywords.contains(&lower_word.as_str()) {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "Column name '{}' is a SQL keyword. Consider using a different name or quoting it",
                                    clean_word
                                ),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    fn check_aliases(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
        keywords: &[&str],
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();

        for (word_idx, word) in words.iter().enumerate() {
            if word.to_lowercase() == "as" && word_idx + 1 < words.len() {
                let alias =
                    words[word_idx + 1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                let lower_alias = alias.to_lowercase();

                if keywords.contains(&lower_alias.as_str()) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Alias '{}' is a SQL keyword. Consider using a different alias or quoting it",
                            alias
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}
