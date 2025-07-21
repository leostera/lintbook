use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct MemoryOptimization;

impl Rule for MemoryOptimization {
    fn id(&self) -> &'static str {
        "SQL067"
    }

    fn name(&self) -> &'static str {
        "memory-optimization"
    }

    fn description(&self) -> &'static str {
        "Identify patterns that may cause excessive memory usage"
    }

    fn explanation(&self) -> &'static str {
        "Database operations can consume significant memory, especially with large datasets.
        This rule identifies patterns that may lead to memory pressure or inefficient memory usage."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_memory_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl MemoryOptimization {
    fn check_memory_patterns(
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
            
            // Check for memory-intensive operations
            self.check_memory_intensive_operations(&lower_line, line_idx, node, violations);
            
            // Check for sort operations
            self.check_sort_operations(&lower_line, line_idx, node, violations);
            
            // Check for large result sets
            self.check_large_result_sets(&lower_line, line_idx, node, violations);
            
            // Check for memory-inefficient data types
            self.check_memory_inefficient_types(&lower_line, line_idx, node, violations);
            
            // Check for temp object usage
            self.check_temp_object_usage(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_memory_patterns(child, source, violations);
            }
        }
    }
    
    fn check_memory_intensive_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for operations that can consume lots of memory
        if lower_line.contains("distinct") && lower_line.contains("order by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DISTINCT with ORDER BY can require significant memory for sorting unique values".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for hash operations
        if lower_line.contains("group by") && !lower_line.contains("order by") {
            // Large GROUP BY operations can use hash tables
            let group_columns = lower_line.matches(",").count() + 1;
            if group_columns > 5 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "GROUP BY with {} columns may require large hash tables. Consider reducing grouping columns",
                        group_columns
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for window functions (can require buffering)
        if lower_line.contains(" over ") && (lower_line.contains("partition by") || lower_line.contains("order by")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Window functions may require buffering large amounts of data. Monitor memory usage with large datasets".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for recursive CTEs (can grow exponentially)
        if lower_line.contains("with") && lower_line.contains("recursive") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Recursive CTE can consume exponential memory. Ensure termination conditions and consider MAXRECURSION".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for string aggregation (can build large strings)
        if lower_line.contains("string_agg") || lower_line.contains("group_concat") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "String aggregation can build very large strings. Consider limiting output size or using alternative approaches".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_sort_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for multiple ORDER BY clauses in subqueries
        if lower_line.contains("order by") && lower_line.contains("(select") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ORDER BY in subquery may require additional sorting. Consider if ordering is necessary at this level".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for sort on large columns
        if lower_line.contains("order by") && (lower_line.contains("varchar(max)") || lower_line.contains("text")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ORDER BY on large text columns can be memory-intensive. Consider computed columns or indexing strategies".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for union with order by
        if lower_line.contains("union") && lower_line.contains("order by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UNION with ORDER BY requires sorting the combined result set. Consider performance impact with large datasets".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_large_result_sets(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for SELECT * without limits
        if lower_line.contains("select *") && !lower_line.contains("top ") && 
           !lower_line.contains("limit ") && !lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "SELECT * without WHERE or LIMIT can return entire tables. Consider adding constraints to limit memory usage".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for large joins without limits
        let join_count = lower_line.matches(" join ").count();
        if join_count >= 3 && !lower_line.contains("top ") && !lower_line.contains("limit ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Query with {} joins without result limiting. Large result sets may consume excessive memory",
                    join_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for cross joins (cartesian products)
        if lower_line.contains("cross join") || (lower_line.contains(",") && !lower_line.contains("where")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Cross join or cartesian product can generate enormous result sets. Ensure this is intentional and bounded".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_memory_inefficient_types(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for oversized data types
        if lower_line.contains("varchar(max)") || lower_line.contains("nvarchar(max)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "MAX-sized varchar columns can consume large amounts of memory. Use specific sizes when possible".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for large fixed-length types
        if lower_line.contains("char(") {
            // Extract size if possible (simplified check)
            if let Some(start) = lower_line.find("char(") {
                if let Some(end) = lower_line[start..].find(")") {
                    let size_part = &lower_line[start + 5..start + end];
                    if let Ok(size) = size_part.parse::<u32>() {
                        if size > 100 {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "Large CHAR({}) column always allocates full size. Consider VARCHAR for variable-length data",
                                    size
                                ),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        // Check for binary large objects
        if lower_line.contains("varbinary(max)") || lower_line.contains("image") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Large binary columns can consume significant memory. Consider filestream or external storage for large files".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for XML columns (can be large)
        if lower_line.contains(" xml ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "XML columns can consume significant memory. Consider size limitations and indexing strategies".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_temp_object_usage(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for temp table creation
        if lower_line.contains("create table #") || lower_line.contains("create table ##") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Temporary tables use tempdb memory. Monitor tempdb usage and consider table variables for small datasets".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for table variables with large datasets
        if lower_line.contains("declare @") && lower_line.contains("table") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Table variables are memory-resident. For large datasets, consider temp tables which can spill to disk".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for large variable declarations
        if lower_line.contains("declare @") && (lower_line.contains("varchar(max)") || lower_line.contains("text")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Large string variables can consume significant memory. Consider streaming or chunked processing for large data".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for CTE that might materialize large datasets
        if lower_line.contains("with ") && lower_line.contains(" as (") && 
           !lower_line.contains("recursive") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CTEs may be materialized in memory. For large datasets, consider breaking into temp tables if performance is poor".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}