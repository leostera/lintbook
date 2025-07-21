use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct SecurityPatterns;

impl Rule for SecurityPatterns {
    fn id(&self) -> &'static str {
        "SQL049"
    }

    fn name(&self) -> &'static str {
        "security-patterns"
    }

    fn description(&self) -> &'static str {
        "Detect potential SQL security issues and anti-patterns"
    }

    fn explanation(&self) -> &'static str {
        "Avoid SQL patterns that may indicate security vulnerabilities: dynamic SQL 
        construction, missing input validation patterns, overly permissive permissions, 
        and other security-related concerns."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_security_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl SecurityPatterns {
    fn check_security_patterns(
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
            
            // Check for dynamic SQL construction patterns
            self.check_dynamic_sql_construction(&lower_line, line_idx, node, violations);
            
            // Check for missing WHERE clauses in UPDATE/DELETE
            self.check_missing_where_clauses(&lower_line, line_idx, node, violations);
            
            // Check for overly broad permissions
            self.check_permission_patterns(&lower_line, line_idx, node, violations);
            
            // Check for potential SQL injection patterns
            self.check_injection_patterns(line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_security_patterns(child, source, violations);
            }
        }
    }
    
    fn check_dynamic_sql_construction(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for common dynamic SQL patterns
        let dynamic_patterns = [
            "exec (", "execute (", "sp_executesql", "exec @", "execute @",
            "concat(", "||", "+'", "+ '", "+@", "+ @"
        ];
        
        for pattern in dynamic_patterns.iter() {
            if lower_line.contains(pattern) {
                // Additional context check for SQL construction
                if lower_line.contains("select ") || 
                   lower_line.contains("insert ") || 
                   lower_line.contains("update ") || 
                   lower_line.contains("delete ") ||
                   lower_line.contains("create ") ||
                   lower_line.contains("drop ") {
                    
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Dynamic SQL construction detected with '{}'. Use parameterized queries to prevent SQL injection",
                            pattern
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
    
    fn check_missing_where_clauses(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for UPDATE without WHERE
        if lower_line.trim().starts_with("update ") && !lower_line.contains(" where ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UPDATE statement without WHERE clause will affect all rows. Add WHERE clause or use explicit confirmation".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for DELETE without WHERE
        if lower_line.trim().starts_with("delete ") && !lower_line.contains(" where ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DELETE statement without WHERE clause will delete all rows. Add WHERE clause or use TRUNCATE if intentional".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_permission_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for overly broad grants
        if lower_line.contains("grant ") {
            if lower_line.contains("grant all") || lower_line.contains("grant *") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Granting ALL privileges is overly permissive. Grant only specific privileges needed".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
            
            if lower_line.contains("to public") || lower_line.contains("to everyone") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Granting privileges to PUBLIC/EVERYONE is overly permissive. Grant to specific users/roles".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for weak password patterns in CREATE USER
        if lower_line.contains("create user") || lower_line.contains("alter user") {
            if lower_line.contains("password '") {
                let weak_passwords = ["password", "123", "admin", "user", "test"];
                for weak in weak_passwords.iter() {
                    if lower_line.contains(&format!("password '{}'", weak)) ||
                       lower_line.contains(&format!("password \"{}\"", weak)) {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!("Weak password '{}' detected. Use strong passwords for database users", weak),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    fn check_injection_patterns(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for potential SQL injection patterns in string literals
        if line.contains("'") || line.contains("\"") {
            // Extract string literals
            let string_literals = self.extract_string_literals(line);
            
            for literal in string_literals {
                // Check for SQL keywords in string literals that might indicate injection
                let lower_literal = literal.to_lowercase();
                let sql_keywords = [
                    "union select", "'; drop", "\"; drop", "' or '1'='1", "\" or \"1\"=\"1\"",
                    "' or 1=1", "\" or 1=1", "; exec", "; execute", "' union", "\" union"
                ];
                
                for pattern in sql_keywords.iter() {
                    if lower_literal.contains(pattern) {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "Potential SQL injection pattern '{}' detected in string literal. Review for security implications",
                                pattern
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    fn extract_string_literals(&self, line: &str) -> Vec<String> {
        let mut literals = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == '\'' || chars[i] == '"' {
                let quote = chars[i];
                let start = i;
                i += 1;
                
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                
                if i < chars.len() {
                    let literal = chars[start + 1..i].iter().collect::<String>();
                    literals.push(literal);
                    i += 1;
                } else {
                    break;
                }
            } else {
                i += 1;
            }
        }
        
        literals
    }
}