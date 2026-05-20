use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct LeadingWhitespace;

impl Rule for LeadingWhitespace {
    fn id(&self) -> &'static str {
        "SQL031"
    }

    fn name(&self) -> &'static str {
        "leading-whitespace"
    }

    fn description(&self) -> &'static str {
        "Lines should not have unnecessary leading whitespace"
    }

    fn explanation(&self) -> &'static str {
        "Excessive or inconsistent leading whitespace makes SQL queries harder to read.
        Use consistent indentation (spaces, not tabs) and avoid unnecessary blank spaces
        at the beginning of lines."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_leading_whitespace(tree.root_node(), source, &mut violations);

        violations
    }
}

impl LeadingWhitespace {
    fn check_leading_whitespace(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let lines: Vec<&str> = source.split('\n').collect();
        let mut in_query = false;
        let mut base_indent = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Skip comments
            if line.trim_start().starts_with("--") {
                continue;
            }

            let trimmed = line.trim_start();
            let leading_spaces = line.len() - trimmed.len();

            // Detect start of query
            let lower_trimmed = trimmed.to_lowercase();
            if lower_trimmed.starts_with("select")
                || lower_trimmed.starts_with("insert")
                || lower_trimmed.starts_with("update")
                || lower_trimmed.starts_with("delete")
                || lower_trimmed.starts_with("with")
                || lower_trimmed.starts_with("create")
                || lower_trimmed.starts_with("alter")
                || lower_trimmed.starts_with("drop")
            {
                in_query = true;
                base_indent = leading_spaces;
            }

            // Check for tabs
            if line.contains('\t') {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: 1,
                    message: "Use spaces for indentation, not tabs".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Check for inconsistent indentation
            if in_query && leading_spaces > 0 {
                // Common SQL keywords that should be at the same level
                if matches!(
                    lower_trimmed.split_whitespace().next(),
                    Some("from")
                        | Some("where")
                        | Some("group")
                        | Some("having")
                        | Some("order")
                        | Some("limit")
                        | Some("union")
                ) {
                    if leading_spaces != base_indent {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: 1,
                            message: format!(
                                "Inconsistent indentation: expected {} spaces, found {}",
                                base_indent, leading_spaces
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }

                // Check for odd number of spaces (should use even numbers: 2, 4, 6, etc.)
                if leading_spaces % 2 != 0 {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: 1,
                        message: format!(
                            "Use even number of spaces for indentation (found {} spaces)",
                            leading_spaces
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }

            // End of query
            if trimmed.ends_with(';') {
                in_query = false;
            }
        }

        // Note: We're not recursing into child nodes as we're checking the entire source
    }
}
