use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct DataTypeBestPractices;

impl Rule for DataTypeBestPractices {
    fn id(&self) -> &'static str {
        "SQL050"
    }

    fn name(&self) -> &'static str {
        "data-type-best-practices"
    }

    fn description(&self) -> &'static str {
        "Enforce data type best practices and conventions"
    }

    fn explanation(&self) -> &'static str {
        "Use appropriate data types for better performance, storage efficiency, and data integrity. 
        Avoid deprecated types, oversized types, and prefer standard types for better portability."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_data_types(tree.root_node(), source, &mut violations);

        violations
    }
}

impl DataTypeBestPractices {
    fn check_data_types(
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
            
            // Only check CREATE TABLE and ALTER TABLE statements
            if lower_line.contains("create table") || lower_line.contains("alter table") {
                self.check_deprecated_types(&lower_line, line_idx, node, violations);
                self.check_oversized_types(&lower_line, line_idx, node, violations);
                self.check_varchar_without_length(&lower_line, line_idx, node, violations);
                self.check_float_precision(&lower_line, line_idx, node, violations);
                self.check_money_types(&lower_line, line_idx, node, violations);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_data_types(child, source, violations);
            }
        }
    }
    
    fn check_deprecated_types(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let deprecated_types = [
            ("text", "Use VARCHAR(MAX) or CLOB for large text"),
            ("ntext", "Use NVARCHAR(MAX) for large Unicode text"),
            ("image", "Use VARBINARY(MAX) or BLOB for binary data"),
            ("timestamp", "Use ROWVERSION for SQL Server or consider DATETIME2"),
            ("datetime", "Consider DATETIME2 for better precision (SQL Server)"),
            ("smalldatetime", "Consider DATETIME2 for better range and precision"),
        ];
        
        for (deprecated_type, suggestion) in deprecated_types.iter() {
            if lower_line.contains(&format!(" {} ", deprecated_type)) ||
               lower_line.contains(&format!(" {}\t", deprecated_type)) ||
               lower_line.contains(&format!(" {},", deprecated_type)) ||
               lower_line.contains(&format!(" {});", deprecated_type)) {
                
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Deprecated data type '{}' detected. {}",
                        deprecated_type.to_uppercase(), suggestion
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_oversized_types(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for excessively large VARCHAR/CHAR declarations
        let varchar_patterns = [
            ("varchar(8000)", "VARCHAR(8000) is at the limit, consider VARCHAR(MAX) or review if this size is necessary"),
            ("varchar(max)", "VARCHAR(MAX) can impact performance. Use specific length if possible"),
            ("char(255)", "CHAR(255) may waste space. Consider VARCHAR if length varies"),
            ("nvarchar(4000)", "NVARCHAR(4000) is at the limit, consider NVARCHAR(MAX) or review size"),
        ];
        
        for (pattern, message) in varchar_patterns.iter() {
            if lower_line.contains(pattern) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: message.to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for very large CHAR types
        if lower_line.contains("char(") {
            if let Some(start) = lower_line.find("char(") {
                if let Some(end) = lower_line[start..].find(")") {
                    let size_str = &lower_line[start + 5..start + end];
                    if let Ok(size) = size_str.parse::<u32>() {
                        if size > 50 {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "CHAR({}) may waste space with padding. Consider VARCHAR({}) if length varies",
                                    size, size
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
    
    fn check_varchar_without_length(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for VARCHAR without explicit length (some databases default to 1)
        if lower_line.contains(" varchar ") || lower_line.contains(" varchar,") || lower_line.contains(" varchar)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "VARCHAR without explicit length may default to 1. Specify length: VARCHAR(n)".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains(" nvarchar ") || lower_line.contains(" nvarchar,") || lower_line.contains(" nvarchar)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "NVARCHAR without explicit length may default to 1. Specify length: NVARCHAR(n)".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_float_precision(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for FLOAT without precision (can be imprecise)
        if lower_line.contains(" float ") || lower_line.contains(" float,") || lower_line.contains(" float)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "FLOAT without precision can be imprecise. Consider DECIMAL/NUMERIC for exact values or specify FLOAT precision".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for REAL (usually 32-bit float)
        if lower_line.contains(" real ") || lower_line.contains(" real,") || lower_line.contains(" real)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "REAL has limited precision. Consider DECIMAL/NUMERIC for exact values or DOUBLE PRECISION for more precision".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_money_types(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for MONEY type (not portable)
        if lower_line.contains(" money ") || lower_line.contains(" money,") || lower_line.contains(" money)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "MONEY type is not portable across databases. Use DECIMAL(19,4) or NUMERIC(19,4) for currency".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains(" smallmoney ") || lower_line.contains(" smallmoney,") || lower_line.contains(" smallmoney)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "SMALLMONEY type is not portable. Use DECIMAL(10,4) or NUMERIC(10,4) for smaller currency values".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}