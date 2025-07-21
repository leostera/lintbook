use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct TriggerPatterns;

impl Rule for TriggerPatterns {
    fn id(&self) -> &'static str {
        "SQL059"
    }

    fn name(&self) -> &'static str {
        "trigger-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce trigger design best practices and identify potential issues"
    }

    fn explanation(&self) -> &'static str {
        "Triggers can impact performance and create complex dependencies. This rule checks for
        proper trigger design, performance considerations, and common anti-patterns."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_trigger_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl TriggerPatterns {
    fn check_trigger_patterns(
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
            
            // Check trigger definitions
            if lower_line.contains("create trigger") {
                self.check_trigger_definition(&lower_line, line_idx, node, violations);
                self.check_trigger_naming(line, line_idx, node, violations);
            }
            
            // Check trigger events and timing
            if lower_line.contains("after insert") || lower_line.contains("before insert") ||
               lower_line.contains("after update") || lower_line.contains("before update") ||
               lower_line.contains("after delete") || lower_line.contains("before delete") {
                self.check_trigger_events(&lower_line, line_idx, node, violations);
            }
            
            // Check for problematic operations in triggers
            self.check_trigger_operations(&lower_line, line_idx, node, violations);
            
            // Check for trigger recursion risks
            self.check_recursion_risks(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_trigger_patterns(child, source, violations);
            }
        }
    }
    
    fn check_trigger_definition(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for triggers without proper error handling
        if lower_line.contains("create trigger") && !lower_line.contains("try") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Trigger without error handling. Consider TRY-CATCH blocks to prevent transaction rollback".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for triggers on system tables (generally not recommended)
        let system_table_patterns = ["sys.", "information_schema.", "msdb.", "master.", "tempdb."];
        for pattern in system_table_patterns.iter() {
            if lower_line.contains(pattern) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Trigger on system table/view '{}'. This is generally not recommended and may cause issues",
                        pattern
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for disabled triggers (maintenance issue)
        if lower_line.contains("disable trigger") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Trigger being disabled. Ensure this is temporary and document the reason".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_trigger_naming(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();
        
        if lower_line.contains("create trigger") {
            // Extract trigger name
            if let Some(trigger_start) = lower_line.find("create trigger") {
                let after_trigger = &line[trigger_start + 14..].trim_start();
                if let Some(space_pos) = after_trigger.find(' ') {
                    let trigger_name = &after_trigger[..space_pos].trim();
                    
                    // Check for trigger naming conventions
                    if !trigger_name.starts_with("tr_") && !trigger_name.starts_with("trig_") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + trigger_start + 14,
                            message: format!(
                                "Trigger '{}' doesn't follow naming convention. Consider tr_ or trig_ prefix",
                                trigger_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                    
                    // Check for descriptive naming
                    if !trigger_name.contains("audit") && !trigger_name.contains("log") && 
                       !trigger_name.contains("validate") && !trigger_name.contains("update") &&
                       !trigger_name.contains("insert") && !trigger_name.contains("delete") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + trigger_start + 14,
                            message: format!(
                                "Trigger name '{}' should indicate its purpose (audit, validate, etc.)",
                                trigger_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    fn check_trigger_events(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for triggers on multiple events (complexity)
        let event_count = ["insert", "update", "delete"].iter()
            .filter(|&event| lower_line.contains(event))
            .count();
            
        if event_count > 1 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Trigger handles {} events. Consider separate triggers for each event for better maintainability",
                    event_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for INSTEAD OF triggers (updatable views)
        if lower_line.contains("instead of") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "INSTEAD OF trigger detected. Ensure this is on a view and properly handles all affected base tables".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for AFTER DELETE triggers (cascade considerations)
        if lower_line.contains("after delete") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "AFTER DELETE trigger. Ensure this doesn't conflict with foreign key cascades".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_trigger_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for DDL operations in triggers (generally bad practice)
        let ddl_operations = ["create table", "drop table", "alter table", "create index", "drop index"];
        for operation in ddl_operations.iter() {
            if lower_line.contains(operation) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "DDL operation '{}' in trigger. This can cause performance issues and unexpected behavior",
                        operation.to_uppercase()
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for ROLLBACK in triggers (can cause issues)
        if lower_line.contains("rollback") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ROLLBACK in trigger. This will rollback the entire transaction and may cause application errors".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for expensive operations in triggers
        let expensive_operations = ["waitfor", "backup", "restore", "bulk insert"];
        for operation in expensive_operations.iter() {
            if lower_line.contains(operation) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Expensive operation '{}' in trigger. This will block the triggering transaction",
                        operation.to_uppercase()
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for cursors in triggers (performance impact)
        if lower_line.contains("declare") && lower_line.contains("cursor") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Cursor in trigger. This can significantly impact performance. Consider set-based operations".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for external calls (CLR, xp_cmdshell, etc.)
        let external_calls = ["xp_cmdshell", "openrowset", "opendatasource"];
        for call in external_calls.iter() {
            if lower_line.contains(call) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "External call '{}' in trigger. This can cause security and performance issues",
                        call.to_uppercase()
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn check_recursion_risks(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for INSERT/UPDATE/DELETE on same table (recursion risk)
        if lower_line.contains("insert into") || lower_line.contains("update ") || lower_line.contains("delete from") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DML operation in trigger. Check for potential recursion if this modifies the same table".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for trigger_nestlevel() usage (recursion detection)
        if lower_line.contains("trigger_nestlevel") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "TRIGGER_NESTLEVEL() detected. Good practice for preventing infinite recursion".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for @@ROWCOUNT usage (important for performance)
        if lower_line.contains("@@rowcount") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "@@ROWCOUNT usage in trigger. Ensure trigger logic handles multiple rows properly".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for EXISTS with INSERTED/DELETED (good practice)
        if (lower_line.contains("inserted") || lower_line.contains("deleted")) && !lower_line.contains("exists") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "INSERTED/DELETED table usage. Consider IF EXISTS check for better performance with no affected rows".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}