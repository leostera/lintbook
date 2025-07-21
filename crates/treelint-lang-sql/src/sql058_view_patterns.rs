use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ViewPatterns;

impl Rule for ViewPatterns {
    fn id(&self) -> &'static str {
        "SQL058"
    }

    fn name(&self) -> &'static str {
        "view-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce view design best practices and patterns"
    }

    fn explanation(&self) -> &'static str {
        "Views should be designed for maintainability, performance, and proper abstraction.
        This includes appropriate naming, avoiding complex logic, and proper indexing considerations."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_view_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ViewPatterns {
    fn check_view_patterns(
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
            
            // Check view definitions
            if lower_line.contains("create view") {
                self.check_view_definition(&lower_line, line_idx, node, violations);
                self.check_view_naming(line, line_idx, node, violations);
            }
            
            // Check view complexity
            if lower_line.contains("create view") || lower_line.contains("alter view") {
                self.check_view_complexity(&lower_line, line_idx, node, violations);
            }
            
            // Check for problematic patterns in views
            self.check_view_anti_patterns(&lower_line, line_idx, node, violations);
            
            // Check materialized view patterns
            if lower_line.contains("materialized view") || lower_line.contains("indexed view") {
                self.check_materialized_view_patterns(&lower_line, line_idx, node, violations);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_view_patterns(child, source, violations);
            }
        }
    }
    
    fn check_view_definition(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for views with SELECT *
        if lower_line.contains("select *") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "View uses SELECT *. Specify explicit columns for better maintainability and performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for views without schema binding (SQL Server)
        if lower_line.contains("create view") && !lower_line.contains("with schemabinding") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "View without SCHEMABINDING. Consider WITH SCHEMABINDING for indexed views and dependency tracking".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for views with ORDER BY (not allowed without TOP)
        if lower_line.contains("order by") && !lower_line.contains("top ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ORDER BY in view without TOP clause. ORDER BY is ignored in views without TOP/OFFSET".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for updatable views complexity
        if lower_line.contains(" join ") && lower_line.contains("create view") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "View with JOINs may not be updatable. Document whether this view should support INSERT/UPDATE/DELETE".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_view_naming(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();
        
        if lower_line.contains("create view") {
            // Extract view name
            if let Some(view_start) = lower_line.find("create view") {
                let after_view = &line[view_start + 11..].trim_start();
                if let Some(space_pos) = after_view.find(' ') {
                    let view_name = &after_view[..space_pos].trim();
                    
                    // Check for view naming conventions
                    if !view_name.starts_with("v_") && !view_name.starts_with("view_") && !view_name.ends_with("_view") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + view_start + 11,
                            message: format!(
                                "View '{}' doesn't follow naming convention. Consider v_, view_ prefix or _view suffix",
                                view_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                    
                    // Check for table-like names (confusion)
                    if !view_name.contains("view") && !view_name.contains("v_") {
                        let table_indicators = ["users", "orders", "products", "customers", "items"];
                        for indicator in table_indicators.iter() {
                            if view_name.to_lowercase().contains(indicator) {
                                let start_pos = node.start_position();
                                violations.push(LintViolation {
                                    line: start_pos.row + line_idx + 1,
                                    column: start_pos.column + view_start + 11,
                                    message: format!(
                                        "View name '{}' looks like a table name. Use clear view naming to avoid confusion",
                                        view_name
                                    ),
                                    lint_name: self.name().to_string(),
                                    lint_id: self.id().to_string(),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn check_view_complexity(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Count joins in view
        let join_patterns = [" join ", " inner join ", " left join ", " right join ", " outer join "];
        let mut join_count = 0;
        
        for pattern in join_patterns.iter() {
            join_count += lower_line.matches(pattern).count();
        }
        
        if join_count >= 4 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "View with {} joins may be complex. Consider breaking into smaller views or using CTEs",
                    join_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for subqueries in views
        if lower_line.contains("(select") || lower_line.contains("( select") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Subquery in view. Consider using JOINs or breaking into separate views for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for aggregations in views
        let aggregates = ["sum(", "count(", "avg(", "max(", "min("];
        let mut has_aggregates = false;
        for agg in aggregates.iter() {
            if lower_line.contains(agg) {
                has_aggregates = true;
                break;
            }
        }
        
        if has_aggregates && join_count > 0 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "View with both JOINs and aggregations. Consider materialized view for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_view_anti_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for views calling functions (performance impact)
        let functions = ["getdate()", "newid()", "rand()", "user_name()", "current_user"];
        for func in functions.iter() {
            if lower_line.contains(func) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "View calls function {}. Non-deterministic functions can prevent indexing and optimization",
                        func.to_uppercase()
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for views with UNION (performance consideration)
        if lower_line.contains(" union ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "View with UNION operation. Consider performance implications and index usage".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for views with CASE statements (complexity)
        if lower_line.contains(" case ") && lower_line.matches(" case ").count() > 2 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Multiple CASE statements in view. Consider moving complex logic to stored procedures".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for views referencing other views (nested views)
        if lower_line.contains(" from v_") || lower_line.contains(" join v_") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "View referencing another view. Avoid deep view nesting for better performance and maintainability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_materialized_view_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for materialized views without refresh strategy
        if lower_line.contains("materialized view") && !lower_line.contains("refresh") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Materialized view without refresh strategy. Specify refresh method and schedule".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for indexed views without proper requirements
        if lower_line.contains("with schemabinding") && lower_line.contains("create view") {
            if !lower_line.contains("count_big") && lower_line.contains("count(") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Indexed view with COUNT(*). Use COUNT_BIG(*) for indexed views to avoid overflow".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for materialized view complexity
        if lower_line.contains("materialized view") {
            let join_count = lower_line.matches(" join ").count();
            if join_count > 5 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Complex materialized view with {} joins. Consider refresh performance impact",
                        join_count
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
}