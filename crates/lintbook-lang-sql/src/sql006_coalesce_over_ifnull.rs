use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct CoalesceOverIfnull;

impl Rule for CoalesceOverIfnull {
    fn id(&self) -> &'static str {
        "SQL006"
    }

    fn name(&self) -> &'static str {
        "coalesce-over-ifnull"
    }

    fn description(&self) -> &'static str {
        "Use COALESCE instead of IFNULL or NVL for better SQL standard compliance"
    }

    fn explanation(&self) -> &'static str {
        "COALESCE is part of the SQL standard and works across all databases,
        while IFNULL (MySQL) and NVL (Oracle) are database-specific functions."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_null_functions(tree.root_node(), source, &mut violations);

        violations
    }
}

impl CoalesceOverIfnull {
    fn check_null_functions(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Check for IFNULL function
            if let Some(pos) = lower_line.find("ifnull(") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + pos + 1,
                    message:
                        "Use 'COALESCE' instead of 'IFNULL' for better SQL standard compliance"
                            .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Check for NVL function
            if let Some(pos) = lower_line.find("nvl(") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + pos + 1,
                    message: "Use 'COALESCE' instead of 'NVL' for better SQL standard compliance"
                        .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Check for ISNULL function (SQL Server)
            if let Some(pos) = lower_line.find("isnull(") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + pos + 1,
                    message:
                        "Use 'COALESCE' instead of 'ISNULL' for better SQL standard compliance"
                            .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_null_functions(child, source, violations);
            }
        }
    }
}
