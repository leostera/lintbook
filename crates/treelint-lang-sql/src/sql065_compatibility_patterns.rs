use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct CompatibilityPatterns;

impl Rule for CompatibilityPatterns {
    fn id(&self) -> &'static str {
        "SQL065"
    }

    fn name(&self) -> &'static str {
        "compatibility-patterns"
    }

    fn description(&self) -> &'static str {
        "Ensure cross-database compatibility and identify vendor-specific features"
    }

    fn explanation(&self) -> &'static str {
        "When writing SQL that needs to work across multiple database platforms,
        certain features should be avoided or used carefully. This rule identifies portability issues."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_compatibility_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl CompatibilityPatterns {
    fn check_compatibility_patterns(
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
            
            // Check for vendor-specific data types
            self.check_vendor_specific_types(&lower_line, line_idx, node, violations);
            
            // Check for vendor-specific functions
            self.check_vendor_specific_functions(&lower_line, line_idx, node, violations);
            
            // Check for vendor-specific syntax
            self.check_vendor_specific_syntax(&lower_line, line_idx, node, violations);
            
            // Check for cross-platform compatibility issues
            self.check_cross_platform_issues(&lower_line, line, line_idx, node, violations);
            
            // Check for SQL standard compliance
            self.check_sql_standard_compliance(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_compatibility_patterns(child, source, violations);
            }
        }
    }
    
    fn check_vendor_specific_types(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // SQL Server specific types
        let sqlserver_types = [
            ("uniqueidentifier", "GUID type - not portable. Consider CHAR(36) for cross-platform compatibility"),
            ("money", "SQL Server specific. Use DECIMAL(19,4) for cross-platform compatibility"),
            ("smallmoney", "SQL Server specific. Use DECIMAL(10,4) for cross-platform compatibility"),
            ("datetime2", "SQL Server specific. Use TIMESTAMP or DATETIME for better portability"),
            ("datetimeoffset", "SQL Server specific. Consider separate date and timezone columns"),
            ("hierarchyid", "SQL Server specific hierarchical data type"),
            ("geography", "SQL Server spatial type - check if target databases support spatial data"),
            ("geometry", "SQL Server spatial type - check if target databases support spatial data"),
        ];
        
        for (type_name, message) in sqlserver_types.iter() {
            if lower_line.contains(type_name) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!("Vendor-specific type '{}': {}", type_name.to_uppercase(), message),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // MySQL specific types
        let mysql_types = [
            ("tinyint", "MySQL specific. Use SMALLINT for better portability"),
            ("mediumint", "MySQL specific. Use INT for better portability"),
            ("enum(", "MySQL specific. Consider lookup table for cross-platform compatibility"),
            ("set(", "MySQL specific. Consider separate junction table"),
        ];
        
        for (type_name, message) in mysql_types.iter() {
            if lower_line.contains(type_name) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!("MySQL-specific type '{}': {}", type_name.to_uppercase(), message),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_vendor_specific_functions(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // SQL Server specific functions
        let sqlserver_functions = [
            ("newid()", "SQL Server specific. Use UUID() in MySQL or random values in other databases"),
            ("getdate()", "SQL Server specific. Use NOW() (MySQL) or CURRENT_TIMESTAMP (standard)"),
            ("getutcdate()", "SQL Server specific. Use UTC_TIMESTAMP() in MySQL"),
            ("sysdatetime()", "SQL Server specific. Limited portability"),
            ("datediff(", "Function signature varies between databases. Check parameter order"),
            ("dateadd(", "SQL Server/Access specific. Use INTERVAL in other databases"),
            ("charindex(", "SQL Server specific. Use INSTR() or POSITION() for portability"),
            ("len(", "SQL Server specific. Use LENGTH() or CHAR_LENGTH() for portability"),
            ("stuff(", "SQL Server specific. Use REPLACE() or string concatenation"),
            ("patindex(", "SQL Server specific pattern matching function"),
        ];
        
        for (func_name, message) in sqlserver_functions.iter() {
            if lower_line.contains(func_name) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!("SQL Server function '{}': {}", func_name.to_uppercase(), message),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // MySQL specific functions
        let mysql_functions = [
            ("concat_ws(", "MySQL specific. Concatenate manually for cross-platform compatibility"),
            ("group_concat(", "MySQL specific. Use STRING_AGG() in SQL Server or manual aggregation"),
            ("ifnull(", "MySQL specific. Use ISNULL() (SQL Server) or COALESCE() (standard)"),
            ("unix_timestamp(", "MySQL specific timestamp function"),
            ("from_unixtime(", "MySQL specific timestamp conversion"),
        ];
        
        for (func_name, message) in mysql_functions.iter() {
            if lower_line.contains(func_name) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!("MySQL function '{}': {}", func_name.to_uppercase(), message),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_vendor_specific_syntax(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // SQL Server specific syntax
        if lower_line.contains("with (nolock)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "WITH (NOLOCK) is SQL Server specific. Consider transaction isolation levels for portability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("top ") && !lower_line.contains("select top") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "TOP clause syntax varies between databases. Use LIMIT for MySQL/PostgreSQL compatibility".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("go") && lower_line.trim() == "go" {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "GO is SQL Server Management Studio specific batch separator. Not portable to other databases".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // MySQL specific syntax
        if lower_line.contains("limit ") && lower_line.contains("offset") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "LIMIT...OFFSET syntax is MySQL/PostgreSQL specific. Use OFFSET...FETCH in SQL Server".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("auto_increment") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "AUTO_INCREMENT is MySQL specific. Use IDENTITY in SQL Server or SERIAL in PostgreSQL".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_cross_platform_issues(
        &self,
        lower_line: &str,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Case sensitivity issues
        if lower_line.contains("collate") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "COLLATE clause - collation names vary between database systems. Document platform requirements".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // String concatenation differences
        if lower_line.contains("||") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "|| string concatenation not supported in SQL Server. Use CONCAT() or + operator considerations".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Boolean literal differences
        if lower_line.contains(" true ") || lower_line.contains(" false ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Boolean literals TRUE/FALSE not supported in all databases. Use 1/0 for better compatibility".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Quote character differences
        if line.contains('`') {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Backtick quotes are MySQL specific. Use square brackets [SQL Server] or double quotes [Standard]".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_sql_standard_compliance(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for non-standard join syntax
        if lower_line.contains(" natural join ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "NATURAL JOIN is not widely supported. Use explicit join conditions for better portability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for non-standard window function syntax
        if lower_line.contains(" over ") && !lower_line.contains("partition by") && !lower_line.contains("order by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Window function without PARTITION BY or ORDER BY. Syntax support varies between databases".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for standard compliance recommendations
        if lower_line.contains("isnull(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ISNULL() function name varies. COALESCE() is SQL standard and more portable".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}