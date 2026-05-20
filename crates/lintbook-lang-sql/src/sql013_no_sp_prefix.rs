use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct NoSpPrefix;

impl Rule for NoSpPrefix {
    fn id(&self) -> &'static str {
        "SQL013"
    }

    fn name(&self) -> &'static str {
        "no-sp-prefix"
    }

    fn description(&self) -> &'static str {
        "Avoid SP_ prefix for user-defined stored procedures (T-SQL)"
    }

    fn explanation(&self) -> &'static str {
        "In SQL Server, the SP_ prefix is reserved for system stored procedures.
        Using this prefix for user-defined procedures can cause performance issues and naming conflicts."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_procedure_names(tree.root_node(), source, &mut violations);

        violations
    }
}

impl NoSpPrefix {
    fn check_procedure_names(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Check for CREATE PROCEDURE with SP_ prefix
            if lower_line.contains("create procedure") || lower_line.contains("create proc") {
                let words: Vec<&str> = line.split_whitespace().collect();

                for (word_idx, word) in words.iter().enumerate() {
                    // Look for procedure name after CREATE PROCEDURE
                    if word_idx > 1
                        && (words[word_idx - 2].to_lowercase() == "create"
                            && (words[word_idx - 1].to_lowercase() == "procedure"
                                || words[word_idx - 1].to_lowercase() == "proc"))
                    {
                        let proc_name =
                            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');

                        if proc_name.to_lowercase().starts_with("sp_") {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "Avoid 'SP_' prefix for user-defined stored procedure '{}'. This prefix is reserved for system procedures",
                                    proc_name
                                ),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
                    }
                }
            }

            // Check for ALTER PROCEDURE with SP_ prefix
            if lower_line.contains("alter procedure") || lower_line.contains("alter proc") {
                let words: Vec<&str> = line.split_whitespace().collect();

                for (word_idx, word) in words.iter().enumerate() {
                    if word_idx > 1
                        && (words[word_idx - 2].to_lowercase() == "alter"
                            && (words[word_idx - 1].to_lowercase() == "procedure"
                                || words[word_idx - 1].to_lowercase() == "proc"))
                    {
                        let proc_name =
                            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');

                        if proc_name.to_lowercase().starts_with("sp_") {
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "Avoid 'SP_' prefix for user-defined stored procedure '{}'. This prefix is reserved for system procedures",
                                    proc_name
                                ),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_procedure_names(child, source, violations);
            }
        }
    }
}
