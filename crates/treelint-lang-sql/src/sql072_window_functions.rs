use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct WindowFunctions;

impl Rule for WindowFunctions {
    fn id(&self) -> &'static str {
        "SQL072"
    }

    fn name(&self) -> &'static str {
        "window-functions"
    }

    fn description(&self) -> &'static str {
        "Optimize window function usage and patterns"
    }

    fn explanation(&self) -> &'static str {
        "Window functions are powerful but can be resource-intensive. This rule identifies
        optimization opportunities and best practices for window function usage."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_window_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl WindowFunctions {
    fn check_window_patterns(
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
            
            // Check window function definitions
            if lower_line.contains(" over (") {
                self.check_window_function_usage(&lower_line, line_idx, node, violations);
                self.check_window_partitioning(&lower_line, line_idx, node, violations);
                self.check_window_ordering(&lower_line, line_idx, node, violations);
                self.check_window_frames(&lower_line, line_idx, node, violations);
            }
            
            // Check for window function alternatives
            self.check_window_alternatives(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_window_patterns(child, source, violations);
            }
        }
    }
    
    fn check_window_function_usage(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Count window functions in the query
        let window_count = lower_line.matches(" over (").count();
        if window_count > 5 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Query with {} window functions may impact performance. Consider breaking into multiple queries",
                    window_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for duplicate window specifications
        if lower_line.contains("partition by") && lower_line.matches("partition by").count() > 2 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Multiple similar PARTITION BY clauses. Consider using WINDOW clause to define reusable window specifications".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for window functions without PARTITION BY (full table scan)
        if lower_line.contains(" over ()") || 
           (lower_line.contains(" over (") && !lower_line.contains("partition by")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Window function without PARTITION BY processes entire result set. Consider if partitioning would improve performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for expensive window functions
        let expensive_functions = ["dense_rank()", "percent_rank()", "cume_dist()", "ntile("];
        for func in expensive_functions.iter() {
            if lower_line.contains(func) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Expensive window function '{}' detected. Monitor performance with large datasets",
                        func.trim_end_matches('(').to_uppercase()
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_window_partitioning(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for partitioning on low-cardinality columns
        let low_cardinality_columns = ["status", "type", "category", "gender", "active"];
        if lower_line.contains("partition by") {
            for column in low_cardinality_columns.iter() {
                if lower_line.contains(column) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "PARTITION BY on low-cardinality column '{}'. May not provide significant performance benefit",
                            column
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
        
        // Check for complex partitioning expressions
        if lower_line.contains("partition by") && 
           (lower_line.contains("case ") || lower_line.contains("substring") || lower_line.contains("year(")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex expression in PARTITION BY. Consider computed columns or simpler partitioning for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for multiple column partitioning
        if lower_line.contains("partition by") && lower_line.matches(",").count() > 2 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "PARTITION BY with many columns. Consider if fewer partitioning columns would be sufficient".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_window_ordering(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for ordering without indexes
        if lower_line.contains("order by") && lower_line.contains(" over (") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Window function with ORDER BY. Ensure appropriate indexes exist for sorting columns".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for ordering on text columns
        if lower_line.contains("order by") && 
           (lower_line.contains("name") || lower_line.contains("description") || lower_line.contains("title")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ORDER BY on text column in window function. Consider impact on sort performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for complex ordering expressions
        if lower_line.contains("order by") && 
           (lower_line.contains("case ") || lower_line.contains("substring") || lower_line.contains("len(")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex expression in ORDER BY within window function. Consider computed columns for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_window_frames(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for unbounded frames (can be memory intensive)
        if lower_line.contains("rows unbounded preceding") || 
           lower_line.contains("range unbounded preceding") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Unbounded window frame. May require processing all rows in partition - monitor memory usage".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for inappropriate frame specifications
        if (lower_line.contains("row_number()") || lower_line.contains("rank()")) && 
           lower_line.contains("rows between") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ROW_NUMBER/RANK with frame specification. These functions typically don't need explicit frames".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for large frame windows
        if lower_line.contains("rows between") && 
           (lower_line.contains("100") || lower_line.contains("1000")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Large window frame size. Consider performance impact and if a smaller frame would suffice".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for RANGE frames with ORDER BY on non-unique columns
        if lower_line.contains("range between") && lower_line.contains("order by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "RANGE frame with ORDER BY. Ensure ORDER BY column has sufficient uniqueness to avoid unexpected results".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_window_alternatives(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Suggest GROUP BY for simple aggregations
        if lower_line.contains("sum(") && lower_line.contains(" over (") &&
           lower_line.contains("partition by") && !lower_line.contains("order by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Simple SUM with PARTITION BY. Consider if GROUP BY with aggregate would be more efficient".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Suggest EXISTS for window-based existence checks
        if lower_line.contains("count(") && lower_line.contains(" over (") &&
           lower_line.contains(" > 0") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "COUNT window function for existence check. Consider EXISTS subquery for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Suggest FIRST_VALUE/LAST_VALUE for MIN/MAX alternatives
        if (lower_line.contains("min(") || lower_line.contains("max(")) && 
           lower_line.contains(" over (") && lower_line.contains("order by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "MIN/MAX with ORDER BY in window function. Consider FIRST_VALUE/LAST_VALUE for potentially better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Suggest CTE for complex window expressions
        if lower_line.contains(" over (") && lower_line.contains("case ") &&
           lower_line.contains("when") && lower_line.contains("then") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex CASE expression in window function. Consider CTE to separate logic for better readability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Suggest self-join for LAG/LEAD alternatives (sometimes)
        if (lower_line.contains("lag(") || lower_line.contains("lead(")) && 
           lower_line.contains("1") && !lower_line.contains("partition by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "LAG/LEAD without PARTITION BY. Consider if self-join might be more efficient for small datasets".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}