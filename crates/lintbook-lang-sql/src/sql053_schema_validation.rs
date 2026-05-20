use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct SchemaValidation;

impl Rule for SchemaValidation {
    fn id(&self) -> &'static str {
        "SQL053"
    }

    fn name(&self) -> &'static str {
        "schema-validation"
    }

    fn description(&self) -> &'static str {
        "Validate schema design patterns and conventions"
    }

    fn explanation(&self) -> &'static str {
        "Enforce schema design best practices: proper naming conventions, constraints usage,
        normalization patterns, and common schema anti-patterns to avoid."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_schema_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl SchemaValidation {
    fn check_schema_patterns(
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

            if lower_line.contains("create table") {
                self.check_table_design(&lower_line, line_idx, node, violations);
                self.check_naming_conventions(line, line_idx, node, violations);
                self.check_column_constraints(&lower_line, line_idx, node, violations);
                self.check_primary_key_patterns(&lower_line, line_idx, node, violations);
            }

            if lower_line.contains("foreign key") || lower_line.contains("references") {
                self.check_foreign_key_patterns(&lower_line, line_idx, node, violations);
            }

            if lower_line.contains("create index") {
                self.check_index_naming(&lower_line, line_idx, node, violations);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_schema_patterns(child, source, violations);
            }
        }
    }

    fn check_table_design(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for tables without explicit primary key
        if !lower_line.contains("primary key") && !lower_line.contains("pk") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Table created without explicit primary key. Every table should have a primary key".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for very wide tables (many columns)
        let column_indicators = ["varchar", "int", "decimal", "datetime", "bit", "text"];
        let mut column_count = 0;
        for indicator in column_indicators.iter() {
            column_count += lower_line.matches(indicator).count();
        }

        if column_count > 15 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Table appears to have {} columns. Consider normalizing wide tables or using vertical partitioning",
                    column_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_naming_conventions(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();

        // Extract table name from CREATE TABLE statement
        if lower_line.contains("create table") {
            if let Some(table_start) = lower_line.find("create table") {
                let after_table = &line[table_start + 12..].trim_start();
                if let Some(space_pos) = after_table.find(' ') {
                    let table_name = &after_table[..space_pos].trim();

                    // Check for plural table names (controversial but common convention)
                    if !table_name.ends_with('s') && !table_name.is_empty() {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + table_start + 12,
                            message: format!(
                                "Table name '{}' is singular. Consider using plural form for table names",
                                table_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }

                    // Check for Hungarian notation or prefixes
                    if table_name.starts_with("tbl") || table_name.starts_with("t_") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + table_start + 12,
                            message: format!(
                                "Table name '{}' uses Hungarian notation prefix. Use descriptive names without type prefixes",
                                table_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }

                    // Check for reserved words as table names
                    let reserved_words = ["user", "order", "group", "index", "table", "column"];
                    for word in reserved_words.iter() {
                        if table_name.to_lowercase() == *word {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + table_start + 12,
                                message: format!(
                                    "Table name '{}' is a reserved word. Use a different name or quote the identifier",
                                    table_name
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

    fn check_column_constraints(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for missing NOT NULL on important columns
        let important_columns = ["email", "username", "name", "status", "created_at"];
        for column in important_columns.iter() {
            if lower_line.contains(column) && !lower_line.contains("not null") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Column '{}' should probably be NOT NULL for data integrity",
                        column
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for missing default values on boolean columns
        if lower_line.contains(" bit ") || lower_line.contains(" boolean ") {
            if !lower_line.contains("default") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message:
                        "Boolean column without default value. Consider adding DEFAULT constraint"
                            .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for audit columns
        let has_created_at =
            lower_line.contains("created_at") || lower_line.contains("created_date");
        let has_updated_at =
            lower_line.contains("updated_at") || lower_line.contains("modified_date");

        if !has_created_at && !has_updated_at {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Consider adding audit columns (created_at, updated_at) for tracking record changes".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_primary_key_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for composite primary keys (sometimes anti-pattern)
        if lower_line.contains("primary key") {
            if let Some(pk_start) = lower_line.find("primary key") {
                let pk_part = &lower_line[pk_start..];
                if pk_part.contains(",") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pk_start,
                        message: "Composite primary key detected. Consider using surrogate key with unique constraint instead".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Check for natural vs surrogate key patterns
        if lower_line.contains(" varchar") && lower_line.contains("primary key") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "String primary key detected. Consider using integer surrogate key for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_foreign_key_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for foreign key naming convention
        if lower_line.contains("foreign key") || lower_line.contains("references") {
            // Simple check for _id suffix
            if !lower_line.contains("_id") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message:
                        "Foreign key column should follow naming convention (e.g., table_name_id)"
                            .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for cascade options
        if lower_line.contains("references")
            && !lower_line.contains("cascade")
            && !lower_line.contains("restrict")
        {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Foreign key without explicit CASCADE/RESTRICT option. Specify referential action explicitly".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_index_naming(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if lower_line.contains("create index") {
            // Extract index name
            if let Some(index_start) = lower_line.find("create index") {
                let after_index = &lower_line[index_start + 12..].trim_start();
                if let Some(space_pos) = after_index.find(' ') {
                    let index_name = &after_index[..space_pos].trim();

                    // Check for meaningful index names
                    if index_name.starts_with("ix_") || index_name.starts_with("idx_") {
                        // Good practice
                    } else {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + index_start + 12,
                            message: format!(
                                "Index name '{}' doesn't follow naming convention. Consider ix_ or idx_ prefix",
                                index_name
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
