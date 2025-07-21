use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};
use std::collections::HashSet;

pub struct UndefinedFunctions;

impl Rule for UndefinedFunctions {
    fn id(&self) -> &'static str {
        "SQL047"
    }

    fn name(&self) -> &'static str {
        "undefined-functions"
    }

    fn description(&self) -> &'static str {
        "Check for potentially undefined or database-specific functions"
    }

    fn explanation(&self) -> &'static str {
        "Use standard SQL functions when possible for better portability. Some functions 
        are database-specific and may not work across different SQL platforms. Consider 
        using ANSI SQL equivalents or add comments explaining database dependencies."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_function_usage(tree.root_node(), source, &mut violations);

        violations
    }
}

impl UndefinedFunctions {
    fn check_function_usage(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        let standard_functions = self.get_standard_sql_functions();
        let db_specific_functions = self.get_database_specific_functions();

        for (line_idx, line) in lines.iter().enumerate() {
            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }
            
            // Find function calls
            let functions = self.extract_function_calls(line);
            
            for (func_name, pos) in functions {
                let lower_func = func_name.to_lowercase();
                
                // Check if it's a known standard function
                if standard_functions.contains(&lower_func) {
                    continue;
                }
                
                // Check if it's a known database-specific function
                if let Some(db_info) = db_specific_functions.get(&lower_func) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pos + 1,
                        message: format!(
                            "Function '{}' is {}-specific. {}",
                            func_name, db_info.database, db_info.suggestion
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                    continue;
                }
                
                // Check if it looks like a user-defined function (common patterns)
                if self.looks_like_user_function(&func_name) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pos + 1,
                        message: format!(
                            "Function '{}' appears to be user-defined. Ensure it exists in target database",
                            func_name
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_function_usage(child, source, violations);
            }
        }
    }
    
    fn extract_function_calls(&self, line: &str) -> Vec<(String, usize)> {
        let mut functions = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            // Skip string literals
            if chars[i] == '\'' || chars[i] == '"' {
                let quote = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                i += 1;
                continue;
            }
            
            // Look for function pattern: identifier followed by (
            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                
                // Extract identifier
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                
                // Skip whitespace
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                
                // Check if followed by opening parenthesis
                if j < chars.len() && chars[j] == '(' {
                    let func_name: String = chars[start..i].iter().collect();
                    functions.push((func_name, start));
                    i = j + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        functions
    }
    
    fn get_standard_sql_functions(&self) -> HashSet<String> {
        let functions = [
            // Aggregate functions
            "count", "sum", "avg", "min", "max",
            // String functions
            "upper", "lower", "trim", "ltrim", "rtrim", "length", "substring", "concat",
            "left", "right", "replace", "reverse", "charindex", "position",
            // Date functions (standard)
            "current_date", "current_time", "current_timestamp", "extract",
            // Math functions
            "abs", "ceil", "ceiling", "floor", "round", "sqrt", "power", "exp", "log",
            "sin", "cos", "tan", "asin", "acos", "atan", "pi", "degrees", "radians",
            // Conditional
            "case", "coalesce", "nullif", "isnull",
            // Type conversion
            "cast", "convert",
            // Window functions
            "row_number", "rank", "dense_rank", "ntile", "lag", "lead",
            "first_value", "last_value",
        ];
        
        functions.iter().map(|s| s.to_string()).collect()
    }
    
    fn get_database_specific_functions(&self) -> std::collections::HashMap<String, DatabaseFunction> {
        let mut functions = std::collections::HashMap::new();
        
        // MySQL specific
        functions.insert("group_concat".to_string(), DatabaseFunction {
            database: "MySQL",
            suggestion: "Consider using STRING_AGG (PostgreSQL/SQL Server) or LISTAGG (Oracle)"
        });
        functions.insert("ifnull".to_string(), DatabaseFunction {
            database: "MySQL",
            suggestion: "Use COALESCE for better portability"
        });
        functions.insert("unix_timestamp".to_string(), DatabaseFunction {
            database: "MySQL",
            suggestion: "Use EXTRACT(EPOCH FROM timestamp) for PostgreSQL"
        });
        
        // PostgreSQL specific
        functions.insert("string_agg".to_string(), DatabaseFunction {
            database: "PostgreSQL",
            suggestion: "Use GROUP_CONCAT (MySQL) or STRING_AGG (SQL Server)"
        });
        functions.insert("array_agg".to_string(), DatabaseFunction {
            database: "PostgreSQL",
            suggestion: "Database-specific array aggregation function"
        });
        
        // SQL Server specific
        functions.insert("getdate".to_string(), DatabaseFunction {
            database: "SQL Server",
            suggestion: "Use CURRENT_TIMESTAMP for better portability"
        });
        functions.insert("datediff".to_string(), DatabaseFunction {
            database: "SQL Server",
            suggestion: "Consider using standard date arithmetic"
        });
        functions.insert("iif".to_string(), DatabaseFunction {
            database: "SQL Server",
            suggestion: "Use CASE statement for better portability"
        });
        
        // Oracle specific
        functions.insert("sysdate".to_string(), DatabaseFunction {
            database: "Oracle",
            suggestion: "Use CURRENT_TIMESTAMP for better portability"
        });
        functions.insert("nvl".to_string(), DatabaseFunction {
            database: "Oracle",
            suggestion: "Use COALESCE for better portability"
        });
        functions.insert("listagg".to_string(), DatabaseFunction {
            database: "Oracle",
            suggestion: "Use STRING_AGG (PostgreSQL/SQL Server) or GROUP_CONCAT (MySQL)"
        });
        
        // SQLite specific
        functions.insert("datetime".to_string(), DatabaseFunction {
            database: "SQLite",
            suggestion: "Use CURRENT_TIMESTAMP for standard SQL"
        });
        functions.insert("julianday".to_string(), DatabaseFunction {
            database: "SQLite",
            suggestion: "SQLite-specific date function"
        });
        
        functions
    }
    
    fn looks_like_user_function(&self, func_name: &str) -> bool {
        let lower_name = func_name.to_lowercase();
        
        // Skip if it's too short (likely not a UDF)
        if lower_name.len() < 3 {
            return false;
        }
        
        // Common UDF patterns
        let udf_prefixes = ["fn_", "func_", "sp_", "proc_", "get_", "calc_", "format_"];
        for prefix in udf_prefixes.iter() {
            if lower_name.starts_with(prefix) {
                return true;
            }
        }
        
        // Check for camelCase or underscore patterns that suggest custom functions
        if func_name.chars().any(|c| c.is_uppercase()) && func_name.len() > 6 {
            return true;
        }
        
        // Check for multiple underscores (common in custom functions)
        if lower_name.matches('_').count() >= 2 {
            return true;
        }
        
        false
    }
}

struct DatabaseFunction {
    database: &'static str,
    suggestion: &'static str,
}