use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct RemoveRedundantElseNull;

impl Rule for RemoveRedundantElseNull {
    fn id(&self) -> &'static str {
        "SQL010"
    }

    fn name(&self) -> &'static str {
        "remove-redundant-else-null"
    }

    fn description(&self) -> &'static str {
        "Remove redundant ELSE NULL from CASE statements"
    }

    fn explanation(&self) -> &'static str {
        "CASE statements implicitly return NULL when no conditions match,
        so explicitly adding ELSE NULL is redundant and can be removed for cleaner code."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_case_statements(tree.root_node(), source, &mut violations);

        violations
    }
}

impl RemoveRedundantElseNull {
    fn check_case_statements(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Look for CASE statements with ELSE NULL
            if lower_line.contains("case") {
                self.check_case_in_line(line, line_idx, violations, node);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_case_statements(child, source, violations);
            }
        }
    }

    fn check_case_in_line(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        let lower_line = line.to_lowercase();

        // Look for "ELSE NULL END" pattern
        if lower_line.contains("else null end") {
            if let Some(pos) = lower_line.find("else null end") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + pos + 1,
                    message: "Remove redundant 'ELSE NULL' from CASE statement".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Look for "ELSE NULL" followed by END on next line or later
        if lower_line.contains("else null") && !lower_line.contains("end") {
            if let Some(pos) = lower_line.find("else null") {
                // Check if this is just "ELSE NULL" without other value
                let after_else_null = &lower_line[pos + 9..].trim();
                if after_else_null.is_empty() || after_else_null.starts_with("--") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pos + 1,
                        message: "Remove redundant 'ELSE NULL' from CASE statement".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}
