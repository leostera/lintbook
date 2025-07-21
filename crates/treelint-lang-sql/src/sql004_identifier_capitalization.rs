use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct IdentifierCapitalization;

impl Rule for IdentifierCapitalization {
    fn id(&self) -> &'static str {
        "SQL004"
    }

    fn name(&self) -> &'static str {
        "identifier-capitalization"
    }

    fn description(&self) -> &'static str {
        "Database identifiers should follow consistent capitalization"
    }

    fn explanation(&self) -> &'static str {
        "Database identifiers (table names, column names, etc.) should follow consistent 
        capitalization rules. Common conventions include snake_case or PascalCase."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_identifiers(tree.root_node(), source, &mut violations);

        violations
    }
}

impl IdentifierCapitalization {
    fn check_identifiers(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Check table names in CREATE TABLE
            if lower_line.contains("create table") {
                self.check_create_table_identifiers(line, line_idx, violations, node);
            }

            // Check column names in SELECT
            if lower_line.contains("select") && !lower_line.contains("*") {
                self.check_select_identifiers(line, line_idx, violations, node);
            }

            // Check table/column names in FROM clause
            if lower_line.contains("from ") {
                self.check_from_identifiers(line, line_idx, violations, node);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_identifiers(child, source, violations);
            }
        }
    }

    fn check_create_table_identifiers(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();

        for (word_idx, word) in words.iter().enumerate() {
            if word_idx > 0 && words[word_idx - 1].to_lowercase() == "table" {
                let table_name = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if self.has_mixed_case(table_name) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Table name '{}' should use consistent capitalization (snake_case recommended)",
                            table_name
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn check_select_identifiers(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        // Extract column names from SELECT clause
        if let Some(select_pos) = line.to_lowercase().find("select ") {
            let after_select = &line[select_pos + 7..];
            let from_pos = after_select.to_lowercase().find(" from ");
            let columns_part = if let Some(pos) = from_pos {
                &after_select[..pos]
            } else {
                after_select
            };

            let columns: Vec<&str> = columns_part.split(',').collect();
            for column in columns {
                let trimmed = column.trim();
                // Extract just the column name (before AS if present)
                let col_name = if let Some(as_pos) = trimmed.to_lowercase().find(" as ") {
                    trimmed[..as_pos].trim()
                } else {
                    trimmed
                };

                // Skip function calls and table.column references for now
                if !col_name.contains('(') && !col_name.contains('.') {
                    let clean_name =
                        col_name.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if !clean_name.is_empty() && self.has_mixed_case(clean_name) {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "Column identifier '{}' should use consistent capitalization (snake_case recommended)",
                                clean_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }

    fn check_from_identifiers(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();

        for (word_idx, word) in words.iter().enumerate() {
            if word_idx > 0 && words[word_idx - 1].to_lowercase() == "from" {
                let table_name = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !table_name.is_empty() && self.has_mixed_case(table_name) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Table identifier '{}' should use consistent capitalization (snake_case recommended)",
                            table_name
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn has_mixed_case(&self, identifier: &str) -> bool {
        // Check if identifier mixes uppercase and lowercase (not following snake_case or PascalCase)
        let has_upper = identifier.chars().any(|c| c.is_uppercase());
        let has_lower = identifier.chars().any(|c| c.is_lowercase());

        if !has_upper || !has_lower {
            return false; // All same case is fine
        }

        // Check if it's valid PascalCase (starts with uppercase, no underscores)
        if identifier.chars().next().unwrap().is_uppercase() && !identifier.contains('_') {
            return false; // Valid PascalCase
        }

        // Check if it's valid snake_case (all lowercase with underscores)
        if identifier
            .chars()
            .all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
        {
            return false; // Valid snake_case
        }

        // Check if it's valid SCREAMING_SNAKE_CASE
        if identifier
            .chars()
            .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
        {
            return false; // Valid SCREAMING_SNAKE_CASE
        }

        true // Mixed case that doesn't follow conventions
    }
}
