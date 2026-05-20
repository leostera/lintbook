use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct TableAliasingStyle;

impl Rule for TableAliasingStyle {
    fn id(&self) -> &'static str {
        "SQL001"
    }

    fn name(&self) -> &'static str {
        "table-aliasing-style"
    }

    fn description(&self) -> &'static str {
        "Table aliases should use explicit AS keyword"
    }

    fn explanation(&self) -> &'static str {
        "For clarity and consistency, table aliases should explicitly use the AS keyword.
        Instead of `FROM users u`, use `FROM users AS u`."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        // Simple pattern matching approach since we don't know the exact SQL grammar structure yet
        self.check_simple_pattern(tree.root_node(), source, &mut violations);

        violations
    }
}

impl TableAliasingStyle {
    fn check_simple_pattern(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for patterns like "FROM table_name alias_name" without AS
        let node_text = &source[node.start_byte()..node.end_byte()];

        // Simple regex-like pattern for table aliasing
        if node_text.to_lowercase().contains("from ") {
            let lines: Vec<&str> = node_text.split('\n').collect();
            for (line_idx, line) in lines.iter().enumerate() {
                let lower_line = line.to_lowercase();
                if lower_line.contains("from ") && !lower_line.contains(" as ") {
                    // Look for pattern: FROM table_name identifier (without AS)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(from_idx) = parts.iter().position(|&p| p.to_lowercase() == "from") {
                        if from_idx + 2 < parts.len() {
                            let table_name = parts[from_idx + 1];
                            let potential_alias = parts[from_idx + 2];

                            // Check if potential_alias looks like an alias (not a keyword)
                            let sql_keywords = [
                                "where", "group", "order", "having", "join", "inner", "left",
                                "right", "on",
                            ];
                            if !sql_keywords.contains(&potential_alias.to_lowercase().as_str()) {
                                let start_pos = node.start_position();
                                violations.push(LintViolation {
                                    line: start_pos.row + line_idx + 1,
                                    column: start_pos.column + 1,
                                    message: format!(
                                        "Table alias should use explicit AS keyword: '{}' AS '{}'",
                                        table_name, potential_alias
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

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_simple_pattern(child, source, violations);
            }
        }
    }
}
