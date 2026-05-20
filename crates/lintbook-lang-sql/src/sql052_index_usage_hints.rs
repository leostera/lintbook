use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct IndexUsageHints;

impl Rule for IndexUsageHints {
    fn id(&self) -> &'static str {
        "SQL052"
    }

    fn name(&self) -> &'static str {
        "index-usage-hints"
    }

    fn description(&self) -> &'static str {
        "Provide hints for better index usage and query optimization"
    }

    fn explanation(&self) -> &'static str {
        "Detect patterns that may benefit from indexes, identify potential index usage issues,
        and suggest opportunities for query optimization through proper indexing strategies."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_index_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl IndexUsageHints {
    fn check_index_patterns(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }

            // Check for range queries that might benefit from indexes
            self.check_range_queries(&lower_line, line_idx, node, violations);

            // Check for frequent equality filters
            self.check_equality_filters(&lower_line, line_idx, node, violations);

            // Check for ORDER BY without potential index
            self.check_order_by_patterns(&lower_line, line_idx, node, violations);

            // Check for GROUP BY patterns
            self.check_group_by_patterns(&lower_line, line_idx, node, violations);

            // Check for JOIN conditions
            self.check_join_conditions(&lower_line, line_idx, node, violations);

            // Check for potential covering index opportunities
            self.check_covering_index_opportunities(line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_index_patterns(child, source, violations);
            }
        }
    }

    fn check_range_queries(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for range queries that would benefit from indexes
        let range_patterns = [
            (" between ", "BETWEEN clause"),
            (" > ", "Greater than comparison"),
            (" < ", "Less than comparison"),
            (" >= ", "Greater than or equal comparison"),
            (" <= ", "Less than or equal comparison"),
        ];

        for (pattern, description) in range_patterns.iter() {
            if lower_line.contains(" where ") && lower_line.contains(pattern) {
                // Extract column name (simple heuristic)
                if let Some(where_pos) = lower_line.find(" where ") {
                    let where_clause = &lower_line[where_pos + 7..];
                    if let Some(pattern_pos) = where_clause.find(pattern) {
                        let column_part = where_clause[..pattern_pos].trim();

                        // Simple column name extraction
                        if !column_part.is_empty() && !column_part.contains("(") {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "{} on '{}' - consider adding an index on this column for better performance",
                                    description, column_part
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

    fn check_equality_filters(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" where ") && lower_line.contains(" = ") {
            // Look for common filtering patterns
            let common_filter_columns = [
                "user_id",
                "customer_id",
                "order_id",
                "product_id",
                "account_id",
                "status",
                "type",
                "category",
                "state",
                "active",
                "enabled",
            ];

            for column in common_filter_columns.iter() {
                if lower_line.contains(&format!("{} =", column)) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Frequent equality filter on '{}' - consider adding an index if this query runs frequently",
                            column
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn check_order_by_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" order by ") {
            // Extract ORDER BY columns
            if let Some(order_pos) = lower_line.find(" order by ") {
                let order_clause = &lower_line[order_pos + 10..];

                // Simple check for multiple columns
                if order_clause.contains(",") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Multi-column ORDER BY - consider creating a composite index on these columns".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }

                // Check for ORDER BY with WHERE clause
                if lower_line.contains(" where ") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Query with both WHERE and ORDER BY - consider a composite index covering filter and sort columns".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn check_group_by_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" group by ") {
            // Check for GROUP BY with multiple columns
            if let Some(group_pos) = lower_line.find(" group by ") {
                let group_clause = &lower_line[group_pos + 10..];

                if group_clause.contains(",") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Multi-column GROUP BY - consider creating a composite index on grouping columns".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }

                // Check for GROUP BY with WHERE
                if lower_line.contains(" where ") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Query with WHERE and GROUP BY - consider index covering filter columns first, then grouping columns".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn check_join_conditions(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" join ") && lower_line.contains(" on ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message:
                    "JOIN detected - ensure both sides of join condition have appropriate indexes"
                        .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });

            // Check for foreign key patterns
            if lower_line.contains("_id = ") || lower_line.contains("id = ") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "JOIN on ID column - verify foreign key indexes exist for optimal performance".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_covering_index_opportunities(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();

        // Look for SELECT with specific columns and WHERE clause
        if lower_line.starts_with("select ")
            && !lower_line.contains("select *")
            && lower_line.contains(" where ")
        {
            // Count selected columns (simple heuristic)
            let select_part = if let Some(from_pos) = lower_line.find(" from ") {
                &lower_line[7..from_pos] // Skip "SELECT "
            } else {
                return;
            };

            let column_count = select_part.matches(",").count() + 1;

            if column_count <= 5 && column_count > 1 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "SELECT {} specific columns with WHERE clause - consider a covering index including selected columns",
                        column_count
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
}
