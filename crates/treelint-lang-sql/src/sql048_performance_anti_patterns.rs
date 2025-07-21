use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct PerformanceAntiPatterns;

impl Rule for PerformanceAntiPatterns {
    fn id(&self) -> &'static str {
        "SQL048"
    }

    fn name(&self) -> &'static str {
        "performance-anti-patterns"
    }

    fn description(&self) -> &'static str {
        "Detect common SQL performance anti-patterns"
    }

    fn explanation(&self) -> &'static str {
        "Avoid SQL patterns that can hurt performance: LIKE with leading wildcards, 
        functions in WHERE clauses on indexed columns, NOT IN with possible NULLs, 
        and other common performance pitfalls."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_performance_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl PerformanceAntiPatterns {
    fn check_performance_patterns(
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
            
            // Check for LIKE with leading wildcard
            self.check_leading_wildcard_like(&lower_line, line_idx, node, violations);
            
            // Check for functions in WHERE clause
            self.check_functions_in_where(&lower_line, line_idx, node, violations);
            
            // Check for NOT IN with potential NULL issues
            self.check_not_in_nulls(&lower_line, line_idx, node, violations);
            
            // Check for OR in WHERE that might prevent index usage
            self.check_or_conditions(&lower_line, line_idx, node, violations);
            
            // Check for implicit type conversions
            self.check_implicit_conversions(line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_performance_patterns(child, source, violations);
            }
        }
    }
    
    fn check_leading_wildcard_like(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" like ") {
            // Look for patterns like LIKE '%something'
            if let Some(like_pos) = lower_line.find(" like ") {
                let after_like = &lower_line[like_pos + 6..].trim_start();
                if after_like.starts_with("'%") || after_like.starts_with("\"%") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + like_pos + 1,
                        message: "LIKE with leading wildcard (%) cannot use indexes efficiently. Consider full-text search or restructuring the query".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
    
    fn check_functions_in_where(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" where ") {
            // Common functions that prevent index usage when applied to columns
            let problem_functions = [
                "upper(",
                "lower(",
                "trim(",
                "substring(",
                "left(",
                "right(",
                "year(",
                "month(",
                "day(",
                "datepart(",
                "extract(",
                "convert(",
                "cast(",
                "isnull(",
                "coalesce(",
            ];
            
            for func in problem_functions.iter() {
                if lower_line.contains(func) {
                    // Check if it's in WHERE clause context
                    if let Some(where_pos) = lower_line.find(" where ") {
                        if let Some(func_pos) = lower_line.find(func) {
                            if func_pos > where_pos {
                                let start_pos = node.start_position();
                                violations.push(LintViolation {
                                    line: start_pos.row + line_idx + 1,
                                    column: start_pos.column + func_pos + 1,
                                    message: format!(
                                        "Function {} in WHERE clause may prevent index usage. Consider computed columns or function-based indexes",
                                        func.trim_end_matches('(').to_uppercase()
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
    
    fn check_not_in_nulls(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" not in (") {
            // Check if there's a subquery that might return NULLs
            if lower_line.contains("select ") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "NOT IN with subquery can behave unexpectedly with NULL values. Consider NOT EXISTS or add NULL checks".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_or_conditions(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains(" where ") && lower_line.contains(" or ") {
            // Count OR conditions
            let or_count = lower_line.matches(" or ").count();
            if or_count >= 3 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Multiple OR conditions ({} found) may prevent efficient index usage. Consider UNION ALL or IN clause",
                        or_count
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_implicit_conversions(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();
        
        // Look for numeric columns compared to strings (common conversion issue)
        if lower_line.contains(" where ") || lower_line.contains(" and ") || lower_line.contains(" or ") {
            // Simple pattern: id = 'number' or status_id = 'value'
            let id_patterns = ["id = '", "id='", "_id = '", "_id='"];
            
            for pattern in id_patterns.iter() {
                if lower_line.contains(pattern) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Possible implicit type conversion: comparing numeric ID to string. Use numeric literal for better performance".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}