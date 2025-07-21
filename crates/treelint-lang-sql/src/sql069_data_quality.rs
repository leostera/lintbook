use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct DataQuality;

impl Rule for DataQuality {
    fn id(&self) -> &'static str {
        "SQL069"
    }

    fn name(&self) -> &'static str {
        "data-quality"
    }

    fn description(&self) -> &'static str {
        "Enforce data quality and integrity patterns"
    }

    fn explanation(&self) -> &'static str {
        "Data quality is fundamental to reliable applications. This rule identifies patterns
        that may compromise data integrity and suggests improvements for data validation."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_data_quality_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl DataQuality {
    fn check_data_quality_patterns(
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
            
            // Check constraint patterns
            self.check_constraint_patterns(&lower_line, line_idx, node, violations);
            
            // Check data validation patterns
            self.check_validation_patterns(&lower_line, line_idx, node, violations);
            
            // Check null handling
            self.check_null_handling(&lower_line, line_idx, node, violations);
            
            // Check data type consistency
            self.check_data_type_consistency(&lower_line, line_idx, node, violations);
            
            // Check business rule enforcement
            self.check_business_rules(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_data_quality_patterns(child, source, violations);
            }
        }
    }
    
    fn check_constraint_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for tables without primary keys
        if lower_line.contains("create table") && !lower_line.contains("primary key") &&
           !lower_line.contains("constraint") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Table without primary key constraint. Every table should have a primary key for data integrity".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for missing check constraints on enum-like columns
        let enum_columns = ["status", "type", "category", "state", "priority"];
        for column in enum_columns.iter() {
            if lower_line.contains(column) && lower_line.contains("varchar") &&
               !lower_line.contains("check") && !lower_line.contains("foreign key") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Column '{}' appears to be enum-like but lacks CHECK constraint. Consider adding valid value constraints",
                        column
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for missing foreign key constraints
        if lower_line.contains("_id ") && lower_line.contains("int") &&
           !lower_line.contains("foreign key") && !lower_line.contains("references") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Column ending in '_id' without foreign key constraint. Consider adding referential integrity".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for missing unique constraints
        let unique_candidates = ["email", "username", "ssn", "phone", "license"];
        for column in unique_candidates.iter() {
            if lower_line.contains(column) && lower_line.contains("varchar") &&
               !lower_line.contains("unique") && !lower_line.contains("primary key") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Column '{}' should likely have UNIQUE constraint to prevent duplicates",
                        column
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_validation_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for missing input validation in procedures
        if (lower_line.contains("create procedure") || lower_line.contains("alter procedure")) &&
           lower_line.contains("@") && !lower_line.contains("if") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Stored procedure with parameters but no visible validation. Consider parameter validation".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for date range validation
        if lower_line.contains("datetime") || lower_line.contains("date") {
            if !lower_line.contains("check") && lower_line.contains("create table") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Date column without range validation. Consider CHECK constraints for reasonable date ranges".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for numeric range validation
        if (lower_line.contains("price") || lower_line.contains("amount") || 
            lower_line.contains("quantity") || lower_line.contains("count")) &&
           (lower_line.contains("decimal") || lower_line.contains("money") || lower_line.contains("int")) {
            if !lower_line.contains("check") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Numeric column that should have positive value constraint. Consider CHECK (column >= 0)".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for email validation
        if lower_line.contains("email") && lower_line.contains("varchar") &&
           !lower_line.contains("check") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Email column without format validation. Consider CHECK constraint with email pattern".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_null_handling(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for important columns that should be NOT NULL
        let important_columns = ["name", "title", "description", "status", "created_date", "created_at"];
        for column in important_columns.iter() {
            if lower_line.contains(column) && !lower_line.contains("not null") &&
               !lower_line.contains("primary key") && lower_line.contains("create table") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Column '{}' should likely be NOT NULL for data integrity",
                        column
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for NULL comparisons (should use IS NULL)
        if lower_line.contains("= null") || lower_line.contains("!= null") || lower_line.contains("<> null") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Direct NULL comparison detected. Use IS NULL or IS NOT NULL for proper NULL handling".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for ISNULL usage without default handling
        if lower_line.contains("isnull(") && !lower_line.contains(",") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ISNULL function without default value. Provide meaningful default for NULL handling".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for potential NULL propagation in calculations
        if (lower_line.contains("+") || lower_line.contains("-") || lower_line.contains("*")) &&
           !lower_line.contains("isnull") && !lower_line.contains("coalesce") {
            if lower_line.contains("select") && lower_line.contains("where") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Arithmetic operation without NULL handling. NULLs propagate through calculations".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_data_type_consistency(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for inconsistent ID column types
        if lower_line.contains("_id ") {
            if lower_line.contains("varchar") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "ID column using VARCHAR. Consider consistent numeric or GUID types for IDs across schema".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for storing numbers as strings
        if (lower_line.contains("phone") || lower_line.contains("zip") || 
            lower_line.contains("postal")) && lower_line.contains("varchar") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Numeric data stored as VARCHAR. Consider if this affects sorting, calculations, or storage efficiency".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for boolean representation inconsistency
        if lower_line.contains("is_") || lower_line.contains("has_") || lower_line.contains("can_") {
            if lower_line.contains("varchar") || lower_line.contains("char") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Boolean-like column using text type. Consider BIT or TINYINT for consistency and efficiency".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for date/time inconsistency
        if (lower_line.contains("created") || lower_line.contains("modified") || 
            lower_line.contains("updated")) && lower_line.contains("varchar") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Date/time data stored as VARCHAR. Use proper temporal types for data integrity and querying".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_business_rules(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for age-related business rules
        if lower_line.contains("age") && lower_line.contains("int") && !lower_line.contains("check") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Age column without range validation. Consider CHECK constraint for reasonable age ranges".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for percentage columns
        if (lower_line.contains("percent") || lower_line.contains("rate")) && 
           (lower_line.contains("decimal") || lower_line.contains("float")) &&
           !lower_line.contains("check") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Percentage/rate column without range validation. Consider CHECK constraint for 0-100% or 0-1 range".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for audit trail completeness
        if lower_line.contains("create table") {
            let has_created_date = lower_line.contains("created_date") || lower_line.contains("created_at");
            let has_created_by = lower_line.contains("created_by") || lower_line.contains("creator");
            
            if !has_created_date && !has_created_by {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Table without audit trail columns. Consider adding created_date and created_by for data lineage".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for soft delete pattern
        if lower_line.contains("delete from") && !lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Hard delete operation. Consider soft delete pattern with deleted_date/is_deleted for data recovery".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for versioning in data tables
        if lower_line.contains("create table") && 
           (lower_line.contains("document") || lower_line.contains("article") || 
            lower_line.contains("contract") || lower_line.contains("policy")) &&
           !lower_line.contains("version") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Document-like table without versioning. Consider version tracking for change management".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}