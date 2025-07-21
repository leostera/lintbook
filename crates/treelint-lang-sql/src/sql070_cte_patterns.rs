use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct CtePatterns;

impl Rule for CtePatterns {
    fn id(&self) -> &'static str {
        "SQL070"
    }

    fn name(&self) -> &'static str {
        "cte-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce Common Table Expression (CTE) best practices"
    }

    fn explanation(&self) -> &'static str {
        "CTEs improve query readability and maintainability when used properly.
        This rule identifies CTE usage patterns and suggests improvements."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_cte_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl CtePatterns {
    fn check_cte_patterns(
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
            
            // Check CTE definitions
            if lower_line.contains("with ") && lower_line.contains(" as (") {
                self.check_cte_definition(&lower_line, line_idx, node, violations);
                self.check_cte_naming(line, line_idx, node, violations);
            }
            
            // Check CTE usage patterns
            self.check_cte_usage_patterns(&lower_line, line_idx, node, violations);
            
            // Check recursive CTE patterns
            if lower_line.contains("with recursive") || lower_line.contains("recursive") {
                self.check_recursive_cte_patterns(&lower_line, line_idx, node, violations);
            }
            
            // Check CTE vs alternatives
            self.check_cte_alternatives(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_cte_patterns(child, source, violations);
            }
        }
    }
    
    fn check_cte_definition(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for CTE without column list (when beneficial)
        if lower_line.contains("with ") && lower_line.contains(" as (") &&
           lower_line.contains("case ") && !lower_line.contains("(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTE with complex expressions should specify column list for clarity".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for multiple CTEs that could be combined
        let cte_count = lower_line.matches(" as (").count();
        if cte_count > 5 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Query with {} CTEs may be overly complex. Consider breaking into smaller queries or views",
                    cte_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for CTE that's used only once
        if lower_line.contains("with ") && lower_line.contains(" as (") &&
           !lower_line.contains(",") && lower_line.matches("select").count() == 2 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Single-use CTE might be unnecessary. Consider inline subquery if it doesn't improve readability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for CTE with SELECT *
        if lower_line.contains("with ") && lower_line.contains("select *") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTE with SELECT * reduces readability. Specify columns explicitly in CTEs".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_cte_naming(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();
        
        if lower_line.contains("with ") && lower_line.contains(" as (") {
            // Extract CTE name (simplified)
            if let Some(with_pos) = lower_line.find("with ") {
                let after_with = &line[with_pos + 5..].trim_start();
                if let Some(as_pos) = after_with.find(" as") {
                    let cte_name = after_with[..as_pos].trim();
                    
                    // Check for generic CTE names
                    let generic_names = ["cte", "temp", "tmp", "t1", "t2", "t3", "x", "y", "z"];
                    if generic_names.contains(&cte_name.to_lowercase().as_str()) {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + with_pos + 5,
                            message: format!(
                                "Generic CTE name '{}'. Use descriptive names that explain the CTE's purpose",
                                cte_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                    
                    // Check for very long CTE names
                    if cte_name.len() > 50 {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + with_pos + 5,
                            message: format!(
                                "CTE name '{}' is very long. Consider shorter, more concise naming",
                                cte_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                    
                    // Check for naming conventions
                    if !cte_name.contains("_") && cte_name.len() > 10 {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + with_pos + 5,
                            message: format!(
                                "Long CTE name '{}' without underscores. Consider snake_case for readability",
                                cte_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    fn check_cte_usage_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for CTE with aggregation that could benefit from indexing hints
        if lower_line.contains("with ") && (lower_line.contains("group by") || 
           lower_line.contains("sum(") || lower_line.contains("count(")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTE with aggregation. Ensure underlying tables have appropriate indexes for performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for CTE with window functions
        if lower_line.contains("with ") && lower_line.contains(" over (") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTE with window functions. Consider if the windowing logic can be optimized or needs partitioning".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for CTE chaining (one CTE referencing another)
        if lower_line.contains("with ") && lower_line.contains("from ") {
            // Simple heuristic for CTE references
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTE chaining detected. Ensure logical flow is clear and consider performance implications".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for CTE with UNION operations
        if lower_line.contains("with ") && lower_line.contains(" union ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTE with UNION operation. Consider if UNION ALL is sufficient or if separate CTEs would be clearer".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_recursive_cte_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for recursive CTE without termination condition
        if lower_line.contains("recursive") && !lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Recursive CTE without visible termination condition. Ensure proper WHERE clause prevents infinite recursion".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for recursive CTE without MAXRECURSION hint
        if lower_line.contains("recursive") && !lower_line.contains("maxrecursion") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Recursive CTE without MAXRECURSION option. Consider adding safety limit to prevent runaway recursion".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for complex recursive CTE
        if lower_line.contains("recursive") && (lower_line.contains("join") || 
           lower_line.contains("group by") || lower_line.contains("having")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex recursive CTE with joins/aggregations. Verify performance and consider alternative approaches".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for recursive CTE with large datasets
        if lower_line.contains("recursive") && !lower_line.contains("top ") && !lower_line.contains("limit") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Recursive CTE without result limiting. Consider adding TOP or LIMIT for performance safety".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_cte_alternatives(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check if CTE could be replaced with EXISTS
        if lower_line.contains("with ") && lower_line.contains("in (") &&
           lower_line.contains("select") && !lower_line.contains("group by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTE used for existence check. Consider using EXISTS clause for potentially better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check if CTE could be a view
        if lower_line.contains("with ") && lower_line.matches("with ").count() == 1 &&
           !lower_line.contains("@") && !lower_line.contains("parameter") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Static CTE without parameters. Consider creating a VIEW if this logic is reused".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check if CTE could be a temp table
        if lower_line.contains("with ") && (lower_line.contains("group by") || 
           lower_line.contains("order by")) && lower_line.contains("join") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex CTE with sorting/grouping. Consider temp table with indexes for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for CTE that could be a derived table
        if lower_line.contains("with ") && lower_line.contains(" as (") &&
           !lower_line.contains(",") && lower_line.len() < 200 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Simple single CTE. Consider if derived table (subquery) would be more appropriate".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}