use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ConcurrencyPatterns;

impl Rule for ConcurrencyPatterns {
    fn id(&self) -> &'static str {
        "SQL066"
    }

    fn name(&self) -> &'static str {
        "concurrency-patterns"
    }

    fn description(&self) -> &'static str {
        "Detect concurrency issues and suggest proper isolation level usage"
    }

    fn explanation(&self) -> &'static str {
        "Concurrent database access requires careful consideration of locking, isolation levels,
        and deadlock prevention. This rule identifies potential concurrency issues."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_concurrency_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ConcurrencyPatterns {
    fn check_concurrency_patterns(
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
            
            // Check isolation level settings
            self.check_isolation_levels(&lower_line, line_idx, node, violations);
            
            // Check for locking hints
            self.check_locking_hints(&lower_line, line_idx, node, violations);
            
            // Check for deadlock-prone patterns
            self.check_deadlock_patterns(&lower_line, line_idx, node, violations);
            
            // Check for long-running transactions
            self.check_long_transactions(&lower_line, line_idx, node, violations);
            
            // Check for blocking operations
            self.check_blocking_operations(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_concurrency_patterns(child, source, violations);
            }
        }
    }
    
    fn check_isolation_levels(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for isolation level changes
        if lower_line.contains("set transaction isolation level") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Transaction isolation level change. Document the reason and ensure it's session-scoped when needed".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
            
            // Check for specific problematic isolation levels
            if lower_line.contains("read uncommitted") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "READ UNCOMMITTED isolation allows dirty reads. Consider if this is acceptable for data consistency".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
            
            if lower_line.contains("serializable") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "SERIALIZABLE isolation level can cause significant blocking. Monitor for performance impact".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_locking_hints(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for various locking hints
        let locking_hints = [
            ("with (nolock)", "NOLOCK hint bypasses locking but may return inconsistent data"),
            ("with (readuncommitted)", "READUNCOMMITTED allows dirty reads"),
            ("with (updlock)", "UPDLOCK can cause blocking. Ensure it's necessary"),
            ("with (holdlock)", "HOLDLOCK holds shared locks until transaction end"),
            ("with (serializable)", "SERIALIZABLE hint can cause significant blocking"),
            ("with (readpast)", "READPAST skips locked rows. Ensure this behavior is intended"),
            ("with (rowlock)", "ROWLOCK hint forces row-level locking"),
            ("with (paglock)", "PAGLOCK hint forces page-level locking"),
            ("with (tablock)", "TABLOCK hint forces table-level locking"),
            ("with (tablockx)", "TABLOCKX hint forces exclusive table lock"),
        ];
        
        for (hint, message) in locking_hints.iter() {
            if lower_line.contains(hint) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!("Locking hint '{}': {}", hint.to_uppercase(), message),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_deadlock_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for operations that commonly cause deadlocks
        if lower_line.contains("select") && lower_line.contains("for update") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "SELECT FOR UPDATE can cause deadlocks. Consider using NOWAIT or shorter transactions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for multiple table updates (deadlock risk)
        if lower_line.contains("update") && lower_line.contains("join") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UPDATE with JOIN can increase deadlock risk. Consider accessing tables in consistent order".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for foreign key operations (can cause blocking)
        if (lower_line.contains("insert") || lower_line.contains("update") || lower_line.contains("delete")) &&
           lower_line.contains("foreign key") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DML on tables with foreign keys can cause blocking on parent tables. Monitor for deadlocks".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for index maintenance during transactions
        if lower_line.contains("create index") || lower_line.contains("drop index") ||
           lower_line.contains("alter index") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Index operations can block DML operations. Consider online index operations or maintenance windows".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_long_transactions(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for operations that might create long transactions
        if lower_line.contains("backup database") || lower_line.contains("restore database") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Backup/restore operations can block other transactions. Schedule during low-activity periods".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("bulk insert") || lower_line.contains("bcp") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Bulk operations can create long-running transactions. Consider batching or using minimal logging".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("alter table") && lower_line.contains("add column") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ALTER TABLE operations can block table access. Consider online operations or maintenance windows".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_blocking_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for potentially blocking DDL operations
        if lower_line.contains("create table") && lower_line.contains("as select") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CREATE TABLE AS SELECT can block source tables. Consider if locks on source are acceptable".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for TRUNCATE (can be blocked by open transactions)
        if lower_line.contains("truncate table") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "TRUNCATE TABLE requires exclusive access and can be blocked by active transactions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for statistics updates
        if lower_line.contains("update statistics") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UPDATE STATISTICS can block queries. Consider using sampling or scheduling during low activity".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for DBCC operations that can block
        if lower_line.contains("dbcc checkdb") || lower_line.contains("dbcc checktable") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DBCC integrity checks can impact performance and block operations. Schedule appropriately".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}