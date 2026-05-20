use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct CursorPatterns;

impl Rule for CursorPatterns {
    fn id(&self) -> &'static str {
        "SQL060"
    }

    fn name(&self) -> &'static str {
        "cursor-patterns"
    }

    fn description(&self) -> &'static str {
        "Detect cursor usage and suggest set-based alternatives"
    }

    fn explanation(&self) -> &'static str {
        "Cursors are often inefficient compared to set-based operations. This rule identifies
        cursor usage patterns and suggests alternatives for better performance."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_cursor_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl CursorPatterns {
    fn check_cursor_patterns(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut cursor_declared = false;
        let mut cursor_deallocated = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }

            // Track cursor lifecycle
            if lower_line.contains("declare") && lower_line.contains("cursor") {
                cursor_declared = true;
                self.check_cursor_declaration(&lower_line, line_idx, node, violations);
            }

            if lower_line.contains("deallocate") {
                cursor_deallocated = true;
            }

            // Check cursor operations
            if lower_line.contains("fetch") {
                self.check_fetch_operations(&lower_line, line_idx, node, violations);
            }

            // Check for potential set-based alternatives
            if cursor_declared {
                self.check_set_based_alternatives(&lower_line, line_idx, node, violations);
            }
        }

        // Check for incomplete cursor lifecycle
        if cursor_declared && !cursor_deallocated {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + 1,
                column: start_pos.column + 1,
                message:
                    "Cursor declared but not deallocated. Ensure proper cleanup with DEALLOCATE"
                        .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_cursor_patterns(child, source, violations);
            }
        }
    }

    fn check_cursor_declaration(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // General cursor usage warning
        let start_pos = node.start_position();
        violations.push(LintViolation {
            line: start_pos.row + line_idx + 1,
            column: start_pos.column + 1,
            message: "Cursor detected. Consider set-based operations for better performance"
                .to_string(),
            lint_name: self.name().to_string(),
            lint_id: self.id().to_string(),
        });

        // Check cursor options
        if !lower_line.contains("fast_forward") && !lower_line.contains("forward_only") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Cursor without FAST_FORWARD or FORWARD_ONLY options. Use fastest cursor type when possible".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        if lower_line.contains("scroll") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "SCROLL cursor detected. Scrollable cursors have higher overhead than forward-only cursors".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        if lower_line.contains("dynamic") || lower_line.contains("keyset") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DYNAMIC or KEYSET cursor detected. These have higher overhead than STATIC or FAST_FORWARD cursors".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_fetch_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for proper fetch loop structure
        if lower_line.contains("fetch") && !lower_line.contains("@@fetch_status") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message:
                    "FETCH operation without @@FETCH_STATUS check. Ensure proper loop termination"
                        .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for fetch direction
        if lower_line.contains("fetch last")
            || lower_line.contains("fetch prior")
            || lower_line.contains("fetch first")
            || lower_line.contains("fetch absolute")
        {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Non-sequential FETCH operation. This requires SCROLL cursor and has performance implications".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_set_based_alternatives(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Suggest alternatives for common cursor patterns
        if lower_line.contains("update") && lower_line.contains("where current of") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "UPDATE with WHERE CURRENT OF cursor. Consider set-based UPDATE with proper WHERE clause".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        if lower_line.contains("delete") && lower_line.contains("where current of") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "DELETE with WHERE CURRENT OF cursor. Consider set-based DELETE with proper WHERE clause".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for aggregation patterns
        if lower_line.contains("sum") || lower_line.contains("count") || lower_line.contains("avg")
        {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Aggregation in cursor loop. Consider GROUP BY with aggregate functions for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for row-by-row processing patterns
        if lower_line.contains("insert into") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "INSERT in cursor loop. Consider INSERT...SELECT or bulk operations for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}
