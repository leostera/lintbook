use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct QueryComplexity;

impl Rule for QueryComplexity {
    fn id(&self) -> &'static str {
        "SQL054"
    }

    fn name(&self) -> &'static str {
        "query-complexity"
    }

    fn description(&self) -> &'static str {
        "Monitor and limit query complexity to maintain performance"
    }

    fn explanation(&self) -> &'static str {
        "Complex queries can impact performance and maintainability. This rule detects queries
        with high complexity: many joins, deep nesting, large UNION operations, and complex expressions."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_complexity_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl QueryComplexity {
    fn check_complexity_patterns(
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

            // Check for excessive joins
            self.check_join_count(&lower_line, line_idx, node, violations);

            // Check for deep subquery nesting
            self.check_subquery_nesting(&lower_line, line_idx, node, violations);

            // Check for large UNION operations
            self.check_union_complexity(&lower_line, line_idx, node, violations);

            // Check for complex WHERE clauses
            self.check_where_complexity(&lower_line, line_idx, node, violations);

            // Check for excessive aggregations
            self.check_aggregation_complexity(&lower_line, line_idx, node, violations);

            // Check for cartesian products
            self.check_cartesian_product_risk(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_complexity_patterns(child, source, violations);
            }
        }
    }

    fn check_join_count(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Count different types of joins
        let join_patterns = [
            " join ",
            " inner join ",
            " left join ",
            " right join ",
            " outer join ",
            " cross join ",
        ];
        let mut total_joins = 0;

        for pattern in join_patterns.iter() {
            total_joins += lower_line.matches(pattern).count();
        }

        if total_joins >= 5 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Query has {} joins. Consider breaking into smaller queries or using views for better maintainability",
                    total_joins
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        } else if total_joins >= 3 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Query has {} joins. Monitor performance and consider query optimization",
                    total_joins
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_subquery_nesting(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Count nested parentheses as a proxy for subquery depth
        let mut max_depth = 0;
        let mut current_depth: i32 = 0;

        for ch in lower_line.chars() {
            match ch {
                '(' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                ')' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        // Check if this looks like a SELECT with subqueries
        if lower_line.contains("select") && max_depth >= 3 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Deep subquery nesting detected (depth: {}). Consider using CTEs or temporary tables",
                    max_depth
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for correlated subqueries (EXISTS, IN with SELECT)
        if lower_line.contains(" exists ") || lower_line.contains(" in (select") {
            if lower_line.matches("select").count() > 1 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Correlated subquery detected. Consider rewriting as JOIN for better performance".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_union_complexity(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let union_count =
            lower_line.matches(" union ").count() + lower_line.matches(" union all ").count();

        if union_count >= 4 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Large UNION operation with {} branches. Consider table redesign or alternative approaches",
                    union_count + 1
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for UNION without ALL (performance impact)
        if lower_line.contains(" union ") && !lower_line.contains(" union all ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UNION without ALL requires duplicate elimination. Use UNION ALL if duplicates are acceptable".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_where_complexity(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" where ") {
            // Count logical operators in WHERE clause
            let and_count = lower_line.matches(" and ").count();
            let or_count = lower_line.matches(" or ").count();

            if and_count + or_count >= 5 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Complex WHERE clause with {} conditions. Consider breaking into smaller queries or using temporary filters",
                        and_count + or_count + 1
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Check for complex expressions in WHERE
            if lower_line.contains(" case ") && lower_line.contains(" where ") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "CASE expression in WHERE clause. Consider moving complex logic to SELECT or using computed columns".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_aggregation_complexity(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Count aggregate functions
        let aggregates = [
            "count(",
            "sum(",
            "avg(",
            "min(",
            "max(",
            "stddev(",
            "variance(",
        ];
        let mut agg_count = 0;

        for agg in aggregates.iter() {
            agg_count += lower_line.matches(agg).count();
        }

        if agg_count >= 4 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Query with {} aggregate functions. Consider breaking into multiple queries or using materialized views",
                    agg_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for aggregates with DISTINCT
        if lower_line.contains("count(distinct") || lower_line.contains("sum(distinct") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Aggregate with DISTINCT can be expensive. Consider alternative approaches or ensure proper indexing".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_cartesian_product_risk(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for potential cartesian products
        if lower_line.contains(" from ") && lower_line.contains(",") {
            // Old-style joins with comma
            if !lower_line.contains(" where ") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Comma-separated tables without WHERE clause may create cartesian product. Use explicit JOIN syntax".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for CROSS JOIN (intentional cartesian product)
        if lower_line.contains(" cross join ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CROSS JOIN creates cartesian product. Ensure this is intentional and result set will be manageable".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}
