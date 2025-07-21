use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ReplicationPatterns;

impl Rule for ReplicationPatterns {
    fn id(&self) -> &'static str {
        "SQL063"
    }

    fn name(&self) -> &'static str {
        "replication-patterns"
    }

    fn description(&self) -> &'static str {
        "Detect patterns that may impact replication and high availability"
    }

    fn explanation(&self) -> &'static str {
        "Database replication requires careful consideration of operations that may not replicate
        properly or could cause replication delays. This rule identifies potential replication issues."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_replication_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ReplicationPatterns {
    fn check_replication_patterns(
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
            
            // Check for non-deterministic functions
            self.check_non_deterministic_functions(&lower_line, line_idx, node, violations);
            
            // Check for operations that don't replicate well
            self.check_non_replicable_operations(&lower_line, line_idx, node, violations);
            
            // Check for large transactions (replication impact)
            self.check_large_transaction_patterns(&lower_line, line_idx, node, violations);
            
            // Check for identity column issues
            self.check_identity_replication_issues(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_replication_patterns(child, source, violations);
            }
        }
    }
    
    fn check_non_deterministic_functions(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Functions that return different values on different servers
        let non_deterministic_functions = [
            "newid()", "rand()", "getdate()", "current_timestamp", "sysdatetime()",
            "getutcdate()", "sysutcdatetime()", "user_name()", "system_user",
            "suser_name()", "host_name()", "@@servername", "@@spid",
        ];
        
        for func in non_deterministic_functions.iter() {
            if lower_line.contains(func) {
                // Check if it's in a default constraint or computed column
                if lower_line.contains("default") || lower_line.contains("as ") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Non-deterministic function '{}' in default/computed column may cause replication issues",
                            func.to_uppercase()
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
                
                // Check in DML operations
                if lower_line.contains("insert") || lower_line.contains("update") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Non-deterministic function '{}' in DML operation. Values may differ between primary and replica",
                            func.to_uppercase()
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
    
    fn check_non_replicable_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Operations that don't replicate or have special considerations
        let problematic_operations = [
            ("truncate table", "TRUNCATE TABLE may not replicate in all scenarios. Consider DELETE for replicated tables"),
            ("bulk insert", "BULK INSERT may not replicate. Use INSERT statements or configure for replication"),
            ("bcp ", "BCP operations don't replicate. Use INSERT statements for replicated data"),
            ("select into", "SELECT INTO doesn't replicate. Use CREATE TABLE + INSERT for replicated environments"),
            ("create index", "Index creation may not replicate to subscribers. Check replication settings"),
            ("drop index", "Index drops may not replicate to subscribers. Check replication settings"),
        ];
        
        for (operation, message) in problematic_operations.iter() {
            if lower_line.contains(operation) {
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
        
        // Check for temp table operations (scope issues in replication)
        if lower_line.contains("#") && (lower_line.contains("create table") || lower_line.contains("insert into")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Temporary table operation. Temp tables have different scope in replication contexts".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for cross-database queries (may not work on subscribers)
        if lower_line.contains("..") || (lower_line.contains(".") && lower_line.contains("dbo")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Cross-database reference detected. Ensure referenced databases exist on all replicas".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_large_transaction_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for operations that might create large transactions
        if lower_line.contains("update") && !lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UPDATE without WHERE clause creates large transaction. May cause replication latency".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        if lower_line.contains("delete") && !lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DELETE without WHERE clause creates large transaction. May cause replication latency".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for bulk operations
        if lower_line.contains("insert") && lower_line.contains("select") && !lower_line.contains("top ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Large INSERT...SELECT operation. Consider batching to reduce replication latency".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for ALTER TABLE operations (schema changes)
        if lower_line.contains("alter table") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ALTER TABLE operation. Schema changes may require special handling in replication environments".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_identity_replication_issues(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for identity column operations
        if lower_line.contains("identity(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "IDENTITY column detected. Configure identity range management for replication environments".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for SET IDENTITY_INSERT (replication considerations)
        if lower_line.contains("set identity_insert") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "SET IDENTITY_INSERT operation. May cause conflicts in replication scenarios".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for explicit identity values
        if (lower_line.contains("insert") && lower_line.contains("values")) || 
           (lower_line.contains("insert") && lower_line.contains("select")) {
            if lower_line.contains("1,") || lower_line.contains("2,") || lower_line.contains("(1)") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Possible explicit identity value insertion. Ensure IDENTITY_INSERT is properly managed".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for GUID columns (better for replication)
        if lower_line.contains("uniqueidentifier") && lower_line.contains("default") && lower_line.contains("newid") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UNIQUEIDENTIFIER with NEWID() default. Good choice for replication scenarios".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}