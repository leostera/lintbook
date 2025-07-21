use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct LoggingMonitoring;

impl Rule for LoggingMonitoring {
    fn id(&self) -> &'static str {
        "SQL068"
    }

    fn name(&self) -> &'static str {
        "logging-monitoring"
    }

    fn description(&self) -> &'static str {
        "Ensure proper logging and monitoring practices in SQL code"
    }

    fn explanation(&self) -> &'static str {
        "Proper logging and monitoring are essential for troubleshooting and performance tuning.
        This rule identifies missing logging opportunities and monitoring best practices."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_logging_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl LoggingMonitoring {
    fn check_logging_patterns(
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
            
            // Check for error logging patterns
            self.check_error_logging(&lower_line, line_idx, node, violations);
            
            // Check for performance monitoring
            self.check_performance_monitoring(&lower_line, line_idx, node, violations);
            
            // Check for audit logging opportunities
            self.check_audit_logging(&lower_line, line_idx, node, violations);
            
            // Check for debugging aids
            self.check_debugging_patterns(&lower_line, line_idx, node, violations);
            
            // Check for monitoring best practices
            self.check_monitoring_best_practices(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_logging_patterns(child, source, violations);
            }
        }
    }
    
    fn check_error_logging(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for error handling without logging
        if lower_line.contains("catch") && !lower_line.contains("log") && 
           !lower_line.contains("print") && !lower_line.contains("raiserror") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CATCH block without logging. Consider logging error details for troubleshooting".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for generic error messages
        if lower_line.contains("raiserror") && lower_line.contains("'error'") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Generic error message. Use specific, actionable error messages that aid in troubleshooting".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for error logging without context
        if (lower_line.contains("error_message()") || lower_line.contains("error_number()")) &&
           !lower_line.contains("error_procedure()") && !lower_line.contains("error_line()") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Error logging without full context. Include ERROR_PROCEDURE() and ERROR_LINE() for better debugging".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for silent error handling
        if lower_line.contains("catch") && lower_line.contains("null") && 
           !lower_line.contains("log") && !lower_line.contains("print") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Silent error handling detected. Errors should be logged even if handled gracefully".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_performance_monitoring(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for long-running operations without progress reporting
        if lower_line.contains("backup database") && !lower_line.contains("stats") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Long-running backup without progress monitoring. Consider adding STATS option".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for bulk operations without monitoring
        if (lower_line.contains("bulk insert") || lower_line.contains("bcp")) &&
           !lower_line.contains("batchsize") && !lower_line.contains("rows_per_batch") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Bulk operation without batch size monitoring. Consider adding progress tracking".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for procedures without execution time logging
        if lower_line.contains("create procedure") || lower_line.contains("alter procedure") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Stored procedure without execution time logging. Consider adding performance monitoring".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for cursor operations (should be monitored)
        if lower_line.contains("declare") && lower_line.contains("cursor") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Cursor usage detected. Consider logging row counts and execution time for performance monitoring".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_audit_logging(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for data modification without audit logging
        if lower_line.contains("delete ") && !lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Mass DELETE operation without audit logging. Consider logging affected row counts and user context".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("update ") && !lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Mass UPDATE operation without audit logging. Consider logging affected row counts and user context".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for sensitive operations
        if lower_line.contains("drop table") || lower_line.contains("truncate table") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Destructive operation detected. Consider audit logging with user context and timestamp".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for security-related operations
        if lower_line.contains("grant ") || lower_line.contains("revoke ") || 
           lower_line.contains("alter login") || lower_line.contains("create user") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Security operation detected. Ensure proper audit logging for compliance and security monitoring".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for schema changes
        if lower_line.contains("alter table") || lower_line.contains("create table") ||
           lower_line.contains("drop index") || lower_line.contains("create index") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Schema change detected. Consider logging for change tracking and rollback procedures".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_debugging_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for debugging statements that should be removed
        if lower_line.contains("print 'debug") || lower_line.contains("print 'test") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Debug PRINT statement detected. Remove debug statements before production deployment".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for SELECT without purpose (debugging)
        if lower_line.trim().starts_with("select @") && lower_line.ends_with(";") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Variable SELECT statement. If for debugging, consider removing; if for output, document purpose".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for temporary debugging tables
        if lower_line.contains("create table debug") || lower_line.contains("create table test") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Temporary debugging table detected. Ensure cleanup and consider if this should be in production code".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_monitoring_best_practices(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for @@ROWCOUNT usage for monitoring
        if (lower_line.contains("insert") || lower_line.contains("update") || 
            lower_line.contains("delete")) && !lower_line.contains("@@rowcount") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DML operation without @@ROWCOUNT check. Consider logging affected row counts for monitoring".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for transaction logging
        if lower_line.contains("begin transaction") && !lower_line.contains("log") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Transaction without logging. Consider logging transaction start/end for monitoring long-running transactions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for dynamic SQL without logging
        if lower_line.contains("exec (") || lower_line.contains("execute (") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Dynamic SQL execution. Consider logging the executed SQL for security monitoring and debugging".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for parameter validation logging
        if lower_line.contains("if @") && lower_line.contains(" is null") &&
           !lower_line.contains("raiserror") && !lower_line.contains("log") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Parameter validation without error logging. Log validation failures for troubleshooting".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for business logic without logging
        if lower_line.contains("if ") && lower_line.contains("exists") &&
           !lower_line.contains("log") && !lower_line.contains("print") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Business logic condition without logging. Consider logging for business intelligence and debugging".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}