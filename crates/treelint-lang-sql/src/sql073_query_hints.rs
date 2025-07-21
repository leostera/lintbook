use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct QueryHints;

impl Rule for QueryHints {
    fn id(&self) -> &'static str {
        "SQL073"
    }

    fn name(&self) -> &'static str {
        "query-hints"
    }

    fn description(&self) -> &'static str {
        "Analyze query hints and optimizer directives"
    }

    fn explanation(&self) -> &'static str {
        "Query hints can override query optimizer decisions. This rule identifies hint usage
        and suggests when hints might be unnecessary or potentially harmful."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_hint_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl QueryHints {
    fn check_hint_patterns(
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
            
            // Check join hints
            self.check_join_hints(&lower_line, line_idx, node, violations);
            
            // Check index hints
            self.check_index_hints(&lower_line, line_idx, node, violations);
            
            // Check query execution hints
            self.check_execution_hints(&lower_line, line_idx, node, violations);
            
            // Check optimization hints
            self.check_optimization_hints(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_hint_patterns(child, source, violations);
            }
        }
    }
    
    fn check_join_hints(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for specific join algorithm hints
        let join_hints = [
            ("hash join", "HASH JOIN hint forces hash join algorithm"),
            ("merge join", "MERGE JOIN hint forces merge join algorithm"),
            ("loop join", "LOOP JOIN hint forces nested loop join"),
            ("force order", "FORCE ORDER hint disables join reordering optimization"),
        ];
        
        for (hint, description) in join_hints.iter() {
            if lower_line.contains(hint) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "{}: {}. Verify this is necessary and monitor for plan regression",
                        hint.to_uppercase(), description
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for multiple join hints (conflicting)
        let hint_count = ["hash join", "merge join", "loop join"].iter()
            .filter(|&hint| lower_line.contains(hint))
            .count();
            
        if hint_count > 1 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Multiple join algorithm hints detected. Conflicting hints may cause unexpected behavior".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_index_hints(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for index-related hints
        if lower_line.contains("index(") || lower_line.contains("forceseek") || 
           lower_line.contains("forcescan") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Index hint detected. Document why optimizer's choice was overridden and review periodically".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for FORCESEEK on inappropriate queries
        if lower_line.contains("forceseek") && 
           (lower_line.contains("group by") || lower_line.contains("order by") || lower_line.contains("distinct")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "FORCESEEK with GROUP BY/ORDER BY/DISTINCT. Seek operations may not be optimal for these patterns".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for FORCESCAN with small result sets
        if lower_line.contains("forcescan") && lower_line.contains("top ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "FORCESCAN with TOP clause. Scan may be inefficient for small result sets".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for index hints with parameterized queries
        if (lower_line.contains("index(") || lower_line.contains("forceseek")) &&
           lower_line.contains("@") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Index hint in parameterized query. Hints may not be optimal for all parameter values".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_execution_hints(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for parallelism hints
        if lower_line.contains("maxdop") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "MAXDOP hint overrides server/database setting. Ensure this is appropriate for the query workload".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for memory grants
        if lower_line.contains("max_grant_percent") || lower_line.contains("min_grant_percent") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Memory grant hint detected. Monitor for memory pressure and verify necessity".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for recompile hints
        if lower_line.contains("recompile") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "RECOMPILE hint prevents plan caching. Use sparingly and only when parameter sniffing is problematic".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for keepfixed plan hint
        if lower_line.contains("keepfixed plan") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "KEEPFIXED PLAN prevents plan updates. Ensure statistics don't change significantly for this query".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for optimize for unknown
        if lower_line.contains("optimize for unknown") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "OPTIMIZE FOR UNKNOWN disables parameter sniffing. Good for highly variable parameters".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
    
    fn check_optimization_hints(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for fast hint usage
        if lower_line.contains("fast ") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "FAST hint prioritizes returning first N rows quickly. May impact overall query performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for expand views hint
        if lower_line.contains("expand views") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "EXPAND VIEWS hint inlines view definitions. May affect query plan and performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for robust plan hint
        if lower_line.contains("robust plan") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "ROBUST PLAN hint trades performance for plan stability. Monitor for performance impact".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for concat union hint
        if lower_line.contains("concat union") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "CONCAT UNION hint affects OR clause evaluation. Verify this improves performance for your data distribution".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for multiple optimization hints (potential conflicts)
        let opt_hints = ["fast ", "expand views", "robust plan", "concat union", "hash union", "merge union"];
        let opt_hint_count = opt_hints.iter().filter(|&hint| lower_line.contains(hint)).count();
        
        if opt_hint_count > 2 {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: format!(
                    "Multiple optimization hints ({}) detected. Verify they don't conflict and are all necessary",
                    opt_hint_count
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for hints in views (generally not recommended)
        if (lower_line.contains("create view") || lower_line.contains("alter view")) &&
           (lower_line.contains("with (") || lower_line.contains("option (")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Query hints in view definition. Hints in views affect all queries using the view".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
        
        // Check for hints in functions
        if (lower_line.contains("create function") || lower_line.contains("alter function")) &&
           (lower_line.contains("with (") || lower_line.contains("option (")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Query hints in function definition. Consider if hints are appropriate for all function usage contexts".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}