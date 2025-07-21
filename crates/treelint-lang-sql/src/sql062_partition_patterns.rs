use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct PartitionPatterns;

impl Rule for PartitionPatterns {
    fn id(&self) -> &'static str {
        "SQL062"
    }

    fn name(&self) -> &'static str {
        "partition-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce table partitioning best practices and patterns"
    }

    fn explanation(&self) -> &'static str {
        "Table partitioning can improve performance and maintenance for large tables.
        This rule checks for proper partitioning strategies and common partitioning issues."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_partition_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl PartitionPatterns {
    fn check_partition_patterns(
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
            
            // Check partition function definitions
            if lower_line.contains("create partition function") {
                self.check_partition_function(&lower_line, line_idx, node, violations);
            }
            
            // Check partition scheme definitions
            if lower_line.contains("create partition scheme") {
                self.check_partition_scheme(&lower_line, line_idx, node, violations);
            }
            
            // Check partitioned table patterns
            if lower_line.contains("on ") && (lower_line.contains("create table") || lower_line.contains("create index")) {
                self.check_partitioned_objects(&lower_line, line_idx, node, violations);
            }
            
            // Check for large table candidates
            self.check_partition_candidates(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_partition_patterns(child, source, violations);
            }
        }
    }
    
    fn check_partition_function(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for appropriate data types for partitioning
        let good_partition_types = ["int", "bigint", "datetime", "datetime2", "date"];
        let bad_partition_types = ["varchar", "nvarchar", "text", "ntext", "float", "real"];
        
        let mut has_good_type = false;
        let mut has_bad_type = false;
        
        for good_type in good_partition_types.iter() {
            if lower_line.contains(good_type) {
                has_good_type = true;
                break;
            }
        }
        
        for bad_type in bad_partition_types.iter() {
            if lower_line.contains(bad_type) {
                has_bad_type = true;
                break;
            }
        }
        
        if has_bad_type {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Partition function uses data type that may not be optimal for partitioning. Consider INT, BIGINT, or date types".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for too many partitions (management overhead)
        if lower_line.matches(",").count() > 100 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Partition function with many partitions. Consider if this level of granularity is necessary".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for static partition boundaries (maintenance consideration)
        if lower_line.contains("2023") || lower_line.contains("2024") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Partition function with hardcoded dates. Consider automated partition management for date-based partitioning".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_partition_scheme(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for all partitions on same filegroup (performance consideration)
        if lower_line.contains("to (") && !lower_line.contains(",") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Partition scheme maps all partitions to same filegroup. Consider multiple filegroups for I/O distribution".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for proper filegroup naming
        if lower_line.contains("primary") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Partition scheme uses PRIMARY filegroup. Consider dedicated filegroups for better management".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_partitioned_objects(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for partitioned tables
        if lower_line.contains("create table") {
            // Look for partition scheme reference
            if lower_line.contains("on ") && !lower_line.contains("on primary") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Partitioned table detected. Ensure partition column is part of primary key for optimal performance".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for partitioned indexes
        if lower_line.contains("create index") {
            if lower_line.contains("on ") && !lower_line.contains("on primary") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Partitioned index detected. Ensure index alignment with table partitioning for best performance".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for non-aligned indexes (performance issue)
        if lower_line.contains("create index") && lower_line.contains("on ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Index on partitioned table. Verify partition alignment to avoid performance penalties".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_partition_candidates(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for large table indicators
        if lower_line.contains("create table") {
            // Look for date/time columns that could be partition keys
            let time_columns = ["created_date", "created_at", "order_date", "transaction_date", "modified_date"];
            for column in time_columns.iter() {
                if lower_line.contains(column) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Table with '{}' column. Consider date-based partitioning for large tables with time-series data",
                            column
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
            
            // Look for tables that might benefit from partitioning
            let large_table_indicators = ["transaction", "log", "audit", "history", "archive"];
            for indicator in large_table_indicators.iter() {
                if lower_line.contains(indicator) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Table name contains '{}' - consider partitioning strategy for tables expected to grow large",
                            indicator
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
        
        // Check for queries that might benefit from partition elimination
        if lower_line.contains("where") && (lower_line.contains("date") || lower_line.contains("created") || lower_line.contains("id")) {
            if lower_line.contains("between") || lower_line.contains(">=") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Query with range condition on date/ID column. Ensure partition column is included for partition elimination".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
}