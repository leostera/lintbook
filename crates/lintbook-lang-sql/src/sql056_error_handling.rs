use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct ErrorHandling;

impl Rule for ErrorHandling {
    fn id(&self) -> &'static str {
        "SQL056"
    }

    fn name(&self) -> &'static str {
        "error-handling"
    }

    fn description(&self) -> &'static str {
        "Enforce proper error handling and exception management patterns"
    }

    fn explanation(&self) -> &'static str {
        "Proper error handling is essential for robust database operations. This rule checks for
        try-catch blocks, transaction rollback patterns, and proper error propagation."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_error_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ErrorHandling {
    fn check_error_patterns(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut in_transaction = false;
        let mut has_try_catch = false;
        let mut transaction_line = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }

            // Track transaction state
            if lower_line.contains("begin transaction") || lower_line.contains("begin tran") {
                in_transaction = true;
                transaction_line = line_idx;
                has_try_catch = false;
            }

            if lower_line.contains("commit") || lower_line.contains("rollback") {
                in_transaction = false;
            }

            // Check for try-catch patterns
            self.check_try_catch_patterns(&lower_line, line_idx, node, violations);

            // Check for error raising patterns
            self.check_error_raising(&lower_line, line_idx, node, violations);

            // Check for transaction error handling
            if in_transaction {
                if lower_line.contains("try") || lower_line.contains("catch") {
                    has_try_catch = true;
                }

                self.check_transaction_error_handling(
                    &lower_line,
                    line_idx,
                    node,
                    violations,
                    has_try_catch,
                );
            }

            // Check for silent failures
            self.check_silent_failures(&lower_line, line_idx, node, violations);

            // Check for proper exception information
            self.check_exception_info(&lower_line, line_idx, node, violations);

            // Check for resource cleanup
            self.check_resource_cleanup(&lower_line, line_idx, node, violations);
        }

        // Check for transactions without error handling
        if in_transaction && !has_try_catch {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + transaction_line + 1,
                column: start_pos.column + 1,
                message: "Transaction without error handling. Consider wrapping in TRY-CATCH block"
                    .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_error_patterns(child, source, violations);
            }
        }
    }

    fn check_try_catch_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for empty catch blocks
        if lower_line.trim() == "catch" || lower_line.contains("catch") {
            // This is a simplified check - would need better parsing for real implementation
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CATCH block detected. Ensure it handles errors appropriately and doesn't swallow exceptions silently".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for try without catch
        if lower_line.contains("begin try") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "TRY block found. Ensure there's a corresponding CATCH block for proper error handling".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for nested try-catch (potential anti-pattern)
        if lower_line.contains("try") && lower_line.matches("try").count() > 1 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Nested TRY blocks detected. Consider simplifying error handling logic"
                    .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_error_raising(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for proper error raising
        if lower_line.contains("raiserror") || lower_line.contains("throw") {
            // Check for meaningful error messages
            if lower_line.contains("'error'") || lower_line.contains("\"error\"") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Generic error message 'error'. Use descriptive error messages that help diagnose the issue".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Check for error codes
            if !lower_line.contains("5") && !lower_line.contains("1") && !lower_line.contains("0") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Error raised without severity level. Specify appropriate severity for RAISERROR".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for RETHROW usage
        if lower_line.contains("throw;") || lower_line.trim() == "throw" {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "THROW without parameters re-raises current exception. Ensure this preserves necessary error context".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_transaction_error_handling(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
        has_try_catch: bool,
    ) {
        // Check for transaction operations without error handling
        if lower_line.contains("insert ")
            || lower_line.contains("update ")
            || lower_line.contains("delete ")
        {
            if !has_try_catch {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Data modification in transaction without TRY-CATCH. Consider error handling for transaction safety".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for rollback in catch blocks
        if lower_line.contains("catch") && !lower_line.contains("rollback") {
            // This is a simplified check
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CATCH block in transaction context. Ensure ROLLBACK is called on error"
                    .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_silent_failures(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for @@ERROR usage without handling
        if lower_line.contains("@@error") {
            if !lower_line.contains("if") && !lower_line.contains("when") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "@@ERROR referenced but not used in conditional. Ensure error checking is performed".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for @@ROWCOUNT without validation
        if lower_line.contains("@@rowcount") {
            if !lower_line.contains("if") && !lower_line.contains("=") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "@@ROWCOUNT referenced but not validated. Consider checking if expected number of rows were affected".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for SET NOCOUNT ON without error considerations
        if lower_line.contains("set nocount on") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "SET NOCOUNT ON suppresses row count messages. Ensure this doesn't hide important feedback".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_exception_info(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for error functions in catch blocks
        if lower_line.contains("catch") {
            let error_functions = [
                "error_message()",
                "error_number()",
                "error_severity()",
                "error_state()",
            ];
            let mut has_error_info = false;

            for func in error_functions.iter() {
                if lower_line.contains(func) {
                    has_error_info = true;
                    break;
                }
            }

            if !has_error_info {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "CATCH block without error information functions. Consider using ERROR_MESSAGE(), ERROR_NUMBER() for debugging".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for logging in error handlers
        if lower_line.contains("catch") || lower_line.contains("raiserror") {
            if !lower_line.contains("log")
                && !lower_line.contains("print")
                && !lower_line.contains("insert")
            {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Error handling without logging. Consider logging errors for troubleshooting and monitoring".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_resource_cleanup(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for cursor cleanup
        if lower_line.contains("declare") && lower_line.contains("cursor") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Cursor declared. Ensure proper cleanup with CLOSE and DEALLOCATE in error handling".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for temp table cleanup
        if lower_line.contains("create table #") || lower_line.contains("create table ##") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Temporary table created. Consider cleanup in error handling or use table variables for automatic cleanup".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for connection management
        if lower_line.contains("openquery") || lower_line.contains("linked server") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Remote query detected. Ensure connection timeouts and error handling for network issues".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}
