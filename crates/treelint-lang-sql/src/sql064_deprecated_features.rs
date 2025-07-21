use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct DeprecatedFeatures;

impl Rule for DeprecatedFeatures {
    fn id(&self) -> &'static str {
        "SQL064"
    }

    fn name(&self) -> &'static str {
        "deprecated-features"
    }

    fn description(&self) -> &'static str {
        "Identify usage of deprecated SQL features and suggest modern alternatives"
    }

    fn explanation(&self) -> &'static str {
        "SQL Server and other databases regularly deprecate features in favor of newer,
        better alternatives. This rule identifies deprecated features and suggests replacements."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_deprecated_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl DeprecatedFeatures {
    fn check_deprecated_patterns(
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
            
            // Check deprecated data types
            self.check_deprecated_data_types(&lower_line, line_idx, node, violations);
            
            // Check deprecated functions
            self.check_deprecated_functions(&lower_line, line_idx, node, violations);
            
            // Check deprecated syntax
            self.check_deprecated_syntax(&lower_line, line_idx, node, violations);
            
            // Check deprecated system objects
            self.check_deprecated_system_objects(&lower_line, line_idx, node, violations);
            
            // Check deprecated join syntax
            self.check_deprecated_join_syntax(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_deprecated_patterns(child, source, violations);
            }
        }
    }
    
    fn check_deprecated_data_types(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let deprecated_types = [
            ("text", "Use VARCHAR(MAX) instead of TEXT for large text data"),
            ("ntext", "Use NVARCHAR(MAX) instead of NTEXT for large Unicode text"),
            ("image", "Use VARBINARY(MAX) instead of IMAGE for binary data"),
            ("timestamp", "Use ROWVERSION instead of TIMESTAMP for versioning"),
            ("sql_variant", "Consider specific data types instead of SQL_VARIANT when possible"),
        ];
        
        for (deprecated_type, suggestion) in deprecated_types.iter() {
            if lower_line.contains(&format!(" {} ", deprecated_type)) ||
               lower_line.contains(&format!(" {}\t", deprecated_type)) ||
               lower_line.contains(&format!(" {},", deprecated_type)) ||
               lower_line.contains(&format!(" {})", deprecated_type)) {
                
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Deprecated data type '{}'. {}",
                        deprecated_type.to_uppercase(), suggestion
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_deprecated_functions(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let deprecated_functions = [
            ("datalength(", "Consider LEN() for string length or use specific type functions"),
            ("getdate()", "Consider SYSDATETIME() for higher precision or GETUTCDATE() for UTC"),
            ("host_id()", "Function deprecated - no direct replacement"),
            ("is_member(", "Use IS_ROLEMEMBER() for role membership checks"),
            ("textptr(", "Function deprecated with TEXT/IMAGE types"),
            ("textvalid(", "Function deprecated with TEXT/IMAGE types"),
            ("readtext ", "Use SELECT with VARCHAR(MAX) instead"),
            ("writetext ", "Use UPDATE with VARCHAR(MAX) instead"),
            ("updatetext ", "Use UPDATE with VARCHAR(MAX) instead"),
        ];
        
        for (deprecated_func, suggestion) in deprecated_functions.iter() {
            if lower_line.contains(deprecated_func) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Deprecated function '{}'. {}",
                        deprecated_func.trim_end_matches('(').to_uppercase(), suggestion
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_deprecated_syntax(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for old-style outer joins
        if lower_line.contains("*=") || lower_line.contains("=*") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Deprecated outer join syntax (*= or =*). Use ANSI JOIN syntax (LEFT JOIN, RIGHT JOIN)".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for SET ROWCOUNT (deprecated for DML)
        if lower_line.contains("set rowcount") && !lower_line.contains("set rowcount 0") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "SET ROWCOUNT is deprecated for DML statements. Use TOP clause instead".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for old DBCC commands
        let deprecated_dbcc = [
            ("dbcc dbreindex", "Use ALTER INDEX REBUILD instead"),
            ("dbcc indexdefrag", "Use ALTER INDEX REORGANIZE instead"),
            ("dbcc showcontig", "Use sys.dm_db_index_physical_stats instead"),
            ("dbcc show_statistics", "Use DBCC SHOW_STATISTICS with new syntax"),
        ];
        
        for (deprecated_cmd, suggestion) in deprecated_dbcc.iter() {
            if lower_line.contains(deprecated_cmd) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Deprecated command '{}'. {}",
                        deprecated_cmd.to_uppercase(), suggestion
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for COMPUTE BY clause
        if lower_line.contains("compute by") || lower_line.contains("compute ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "COMPUTE BY clause is deprecated. Use GROUP BY with ROLLUP/CUBE or window functions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_deprecated_system_objects(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for deprecated system tables/views
        let deprecated_objects = [
            ("syscolumns", "Use sys.columns catalog view"),
            ("sysindexes", "Use sys.indexes and sys.index_columns"),
            ("sysobjects", "Use sys.objects catalog view"),
            ("systypes", "Use sys.types catalog view"),
            ("sysusers", "Use sys.database_principals"),
            ("sysdatabases", "Use sys.databases catalog view"),
            ("sysprocesses", "Use sys.dm_exec_sessions and sys.dm_exec_requests"),
            ("syslocks", "Use sys.dm_tran_locks"),
            ("information_schema.columns", "Consider sys.columns for better performance"),
        ];
        
        for (deprecated_obj, suggestion) in deprecated_objects.iter() {
            if lower_line.contains(deprecated_obj) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Deprecated system object '{}'. {}",
                        deprecated_obj, suggestion
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for deprecated compatibility views
        if lower_line.contains("sys.sql_dependencies") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "sys.sql_dependencies is deprecated. Use sys.sql_expression_dependencies".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_deprecated_join_syntax(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for comma-separated joins (old style)
        if lower_line.contains(" from ") && lower_line.contains(",") && 
           !lower_line.contains(" join ") && lower_line.contains(" where ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Old-style comma join syntax. Use explicit INNER JOIN for better readability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for implicit joins without proper conditions
        if lower_line.contains(" from ") && lower_line.contains(",") && 
           !lower_line.contains(" where ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Comma-separated tables without WHERE clause creates cartesian product. Use explicit JOIN syntax".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}