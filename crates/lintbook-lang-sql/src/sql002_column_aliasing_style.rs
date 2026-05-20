use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct ColumnAliasingStyle;

impl Rule for ColumnAliasingStyle {
    fn id(&self) -> &'static str {
        "SQL002"
    }

    fn name(&self) -> &'static str {
        "column-aliasing-style"
    }

    fn description(&self) -> &'static str {
        "Column aliases should use explicit AS keyword"
    }

    fn explanation(&self) -> &'static str {
        "For clarity and consistency, column aliases should explicitly use the AS keyword.
        Instead of `SELECT name alias_name`, use `SELECT name AS alias_name`."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_simple_pattern(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ColumnAliasingStyle {
    fn check_simple_pattern(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];

        // Look for SELECT statements with column aliases
        if node_text.to_lowercase().contains("select ") {
            let lines: Vec<&str> = node_text.split('\n').collect();
            for (line_idx, line) in lines.iter().enumerate() {
                let lower_line = line.to_lowercase();
                if lower_line.contains("select ") {
                    // Split by comma to handle multiple columns
                    let select_part = if let Some(from_pos) = lower_line.find(" from ") {
                        &line[..line.len() - (lower_line.len() - from_pos)]
                    } else {
                        line
                    };

                    if let Some(select_pos) = select_part.to_lowercase().find("select ") {
                        let columns_part = &select_part[select_pos + 7..]; // Skip "SELECT "
                        let columns: Vec<&str> = columns_part.split(',').collect();

                        for column in columns {
                            let trimmed = column.trim();
                            let parts: Vec<&str> = trimmed.split_whitespace().collect();

                            // Check for pattern: column_name alias_name (without AS)
                            if parts.len() == 2 && !trimmed.to_lowercase().contains(" as ") {
                                // Exclude common SQL functions and keywords
                                let sql_keywords = [
                                    "count", "sum", "avg", "max", "min", "distinct", "from",
                                    "where",
                                ];
                                let first_part_lower = parts[0].to_lowercase();

                                if !sql_keywords.iter().any(|&kw| first_part_lower.starts_with(kw)) &&
                                   !first_part_lower.contains('(') && // Exclude function calls
                                   !parts[1].to_lowercase().starts_with("from")
                                {
                                    let start_pos = node.start_position();
                                    violations.push(LintViolation {
                                        line: start_pos.row + line_idx + 1,
                                        column: start_pos.column + 1,
                                        message: format!(
                                            "Column alias should use explicit AS keyword: '{}' AS '{}'",
                                            parts[0], parts[1]
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
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_simple_pattern(child, source, violations);
            }
        }
    }
}
