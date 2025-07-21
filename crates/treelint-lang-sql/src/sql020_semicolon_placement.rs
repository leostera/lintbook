use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct SemicolonPlacement;

impl Rule for SemicolonPlacement {
    fn id(&self) -> &'static str {
        "SQL020"
    }

    fn name(&self) -> &'static str {
        "semicolon-placement"
    }

    fn description(&self) -> &'static str {
        "Consistent semicolon placement and usage"
    }

    fn explanation(&self) -> &'static str {
        "SQL statements should have consistent semicolon placement. 
        Either require semicolons at the end of all statements or consistently omit them."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_semicolon_consistency(tree.root_node(), source, &mut violations);

        violations
    }
}

impl SemicolonPlacement {
    fn check_semicolon_consistency(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut statements = Vec::new();
        let mut current_statement = Vec::new();

        // Group lines into statements
        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            current_statement.push((line_idx, *line));

            // Check if this line ends a statement
            if self.is_statement_end(line) {
                if !current_statement.is_empty() {
                    statements.push(current_statement.clone());
                    current_statement.clear();
                }
            }
        }

        // Add final statement if it doesn't end with semicolon
        if !current_statement.is_empty() {
            statements.push(current_statement);
        }

        // Check each statement for semicolon consistency
        for statement in statements {
            if let Some((last_line_idx, last_line)) = statement.last() {
                self.check_statement_semicolon(
                    &statement,
                    *last_line_idx,
                    last_line,
                    violations,
                    node,
                );
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_semicolon_consistency(child, source, violations);
            }
        }
    }

    fn is_statement_end(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // Ends with semicolon
        if trimmed.ends_with(';') {
            return true;
        }

        // Common statement ending keywords
        let lower_line = trimmed.to_lowercase();
        if lower_line.ends_with("end") || lower_line.ends_with("go") {
            return true;
        }

        false
    }

    fn check_statement_semicolon(
        &self,
        statement: &[(usize, &str)],
        last_line_idx: usize,
        last_line: &str,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        let trimmed_last = last_line.trim();

        // Skip if this is not a complete SQL statement
        let first_line = statement
            .first()
            .map(|(_, line)| line.trim().to_lowercase())
            .unwrap_or_default();
        if !self.is_sql_statement_start(&first_line) {
            return;
        }

        // Check for missing semicolon on multi-line statements
        if statement.len() > 1 && !trimmed_last.ends_with(';') {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + last_line_idx + 1,
                column: start_pos.column + last_line.len(),
                message: "Multi-line SQL statement should end with semicolon".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for semicolon in wrong position (middle of line with content after)
        if let Some(semicolon_pos) = trimmed_last.find(';') {
            let after_semicolon = &trimmed_last[semicolon_pos + 1..].trim();
            if !after_semicolon.is_empty() && !after_semicolon.starts_with("--") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + last_line_idx + 1,
                    column: start_pos.column + semicolon_pos + 1,
                    message: "Semicolon should be at the end of the statement".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn is_sql_statement_start(&self, line: &str) -> bool {
        let keywords = [
            "select", "insert", "update", "delete", "create", "alter", "drop", "grant", "revoke",
            "commit", "rollback", "begin", "declare", "exec", "execute", "with", "merge",
        ];

        keywords.iter().any(|&keyword| line.starts_with(keyword))
    }
}
