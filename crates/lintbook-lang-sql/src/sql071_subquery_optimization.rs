use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct SubqueryOptimization;

impl Rule for SubqueryOptimization {
    fn id(&self) -> &'static str {
        "SQL071"
    }

    fn name(&self) -> &'static str {
        "subquery-optimization"
    }

    fn description(&self) -> &'static str {
        "Identify subquery patterns that can be optimized"
    }

    fn explanation(&self) -> &'static str {
        "Subqueries can often be rewritten as JOINs or EXISTS clauses for better performance.
        This rule identifies optimization opportunities and potential performance issues."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_subquery_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl SubqueryOptimization {
    fn check_subquery_patterns(
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

            // Check for subquery in SELECT clause
            self.check_select_subqueries(&lower_line, line_idx, node, violations);

            // Check for subquery in WHERE clause
            self.check_where_subqueries(&lower_line, line_idx, node, violations);

            // Check for correlated subqueries
            self.check_correlated_subqueries(&lower_line, line_idx, node, violations);

            // Check for subquery alternatives
            self.check_subquery_alternatives(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_subquery_patterns(child, source, violations);
            }
        }
    }

    fn check_select_subqueries(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for subqueries in SELECT clause
        if lower_line.contains("select") && lower_line.contains("(select") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Subquery in SELECT clause. Consider LEFT JOIN for better performance and readability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for scalar subqueries returning aggregates
        if lower_line.contains("(select count") || lower_line.contains("(select sum") ||
           lower_line.contains("(select max") || lower_line.contains("(select min") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Scalar subquery with aggregate. Consider window functions or derived tables for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for multiple subqueries in same SELECT
        let subquery_count = lower_line.matches("(select").count();
        if subquery_count > 2 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Multiple subqueries ({}) in SELECT clause. Consider consolidating with JOINs or CTEs",
                    subquery_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_where_subqueries(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for IN with subquery
        if lower_line.contains(" in (select") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "IN with subquery. Consider EXISTS or INNER JOIN for potentially better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for NOT IN with subquery (dangerous with NULLs)
        if lower_line.contains(" not in (select") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "NOT IN with subquery can behave unexpectedly with NULLs. Use NOT EXISTS or LEFT JOIN with IS NULL".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for ANY/ALL with subqueries
        if lower_line.contains(" any (select") || lower_line.contains(" all (select") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ANY/ALL with subquery. Consider rewriting with MIN/MAX or EXISTS for better readability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for subquery with ORDER BY (usually unnecessary)
        if lower_line.contains("(select") && lower_line.contains("order by") &&
           !lower_line.contains("top ") && !lower_line.contains("limit") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ORDER BY in subquery without TOP/LIMIT is usually unnecessary and wastes resources".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_correlated_subqueries(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for patterns indicating correlated subqueries
        if lower_line.contains("exists (select") &&
           (lower_line.contains(" where ") || lower_line.contains(" and ")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "EXISTS with correlated subquery. Verify that indexes support the correlation for good performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for subquery referencing outer table
        if lower_line.contains("(select") && lower_line.contains(".") &&
           lower_line.contains(" = ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Possible correlated subquery detected. Consider if JOIN would be more efficient".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for correlated subquery in SELECT with aggregation
        if lower_line.contains("(select count") && lower_line.contains("where") &&
           lower_line.contains(".") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Correlated subquery with COUNT. Consider window functions or derived tables for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for multiple correlated subqueries
        if lower_line.matches("(select").count() > 1 && lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Multiple correlated subqueries. Consider consolidating into single JOIN or CTE".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_subquery_alternatives(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Suggest EXISTS instead of IN for existence checks
        if lower_line.contains(" in (select") && !lower_line.contains("distinct") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "IN subquery for existence check. EXISTS is often more efficient and semantically clearer".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Suggest window functions for ranking subqueries
        if lower_line.contains("(select top 1") || lower_line.contains("(select max") ||
           lower_line.contains("(select min") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Subquery for first/last/min/max value. Consider window functions (ROW_NUMBER, RANK) for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Suggest CTEs for complex subqueries
        if lower_line.contains("(select") && lower_line.contains("group by") &&
           lower_line.contains("having") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex subquery with GROUP BY and HAVING. Consider CTE for better readability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Suggest derived tables for repeated subqueries
        if lower_line.matches("(select").count() > 1 {
            // Simple heuristic for similar subqueries
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Multiple subqueries detected. If similar, consider derived table or CTE to avoid duplication".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for subquery that could be a simple JOIN
        if lower_line.contains("where") && lower_line.contains("in (select") &&
           lower_line.contains("where") && !lower_line.contains("group by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Simple IN subquery without aggregation. Consider INNER JOIN for potentially better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}