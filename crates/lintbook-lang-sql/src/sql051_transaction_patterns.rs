use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct TransactionPatterns;

impl Rule for TransactionPatterns {
    fn id(&self) -> &'static str {
        "SQL051"
    }

    fn name(&self) -> &'static str {
        "transaction-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce proper transaction handling patterns"
    }

    fn explanation(&self) -> &'static str {
        "Ensure transactions are properly managed with explicit BEGIN/COMMIT/ROLLBACK,
        avoid nested transactions, and include proper error handling patterns."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_transaction_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl TransactionPatterns {
    fn check_transaction_patterns(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut in_transaction = false;
        let mut begin_line = 0;
        let mut has_commit_or_rollback = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }

            // Check for transaction start
            if lower_line.contains("begin transaction")
                || lower_line.contains("begin tran")
                || lower_line.trim() == "begin"
            {
                if in_transaction {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Nested transaction detected. Avoid nested transactions which can cause unexpected behavior".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
                in_transaction = true;
                begin_line = line_idx;
                has_commit_or_rollback = false;
            }

            // Check for transaction end
            if lower_line.contains("commit") || lower_line.contains("rollback") {
                has_commit_or_rollback = true;
                if lower_line.contains("commit") && !in_transaction {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "COMMIT without corresponding BEGIN TRANSACTION".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
                if lower_line.contains("rollback") && !in_transaction {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "ROLLBACK without corresponding BEGIN TRANSACTION".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
                in_transaction = false;
            }

            // Check for autocommit-sensitive statements
            if lower_line.contains("insert ")
                || lower_line.contains("update ")
                || lower_line.contains("delete ")
                || lower_line.contains("create ")
                || lower_line.contains("drop ")
                || lower_line.contains("alter ")
            {
                if !in_transaction && !self.is_single_row_operation(&lower_line) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Data modification outside transaction. Consider wrapping in explicit transaction for better control".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }

            // Check for dangerous operations without transaction
            if lower_line.contains("truncate ") || lower_line.contains("drop table") {
                if !in_transaction {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Destructive operation outside transaction. Use explicit transaction for safety".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Check for unclosed transactions
        if in_transaction && !has_commit_or_rollback {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + begin_line + 1,
                column: start_pos.column + 1,
                message: "Transaction started but never committed or rolled back".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_transaction_patterns(child, source, violations);
            }
        }
    }

    fn is_single_row_operation(&self, line: &str) -> bool {
        // Simple heuristic for single-row operations
        line.contains(" where ")
            && (line.contains("= ") || line.contains("in ("))
            && !line.contains(" or ")
            && line.matches("=").count() <= 2
    }
}
