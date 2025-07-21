use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct TrailingCommas;

impl Rule for TrailingCommas {
    fn id(&self) -> &'static str {
        "SQL018"
    }

    fn name(&self) -> &'static str {
        "trailing-commas"
    }

    fn description(&self) -> &'static str {
        "Use consistent trailing comma style in SELECT clauses"
    }

    fn explanation(&self) -> &'static str {
        "Trailing commas in SELECT clauses can make version control diffs cleaner 
        but may not be supported by all SQL dialects. Choose one style consistently."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_trailing_comma_style(tree.root_node(), source, &mut violations);

        violations
    }
}

impl TrailingCommas {
    fn check_trailing_comma_style(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut in_select = false;
        let mut select_lines = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            let _trimmed = line.trim();

            // Start of SELECT clause
            if lower_line.contains("select ") && !lower_line.contains("*") {
                in_select = true;
                select_lines.clear();
                select_lines.push((line_idx, *line));
                continue;
            }

            // End of SELECT clause
            if in_select
                && (lower_line.contains(" from ")
                    || lower_line.contains(" where ")
                    || lower_line.contains(" group ")
                    || lower_line.contains(" order ")
                    || lower_line.contains(" having "))
            {
                self.check_select_block(&select_lines, violations, node);
                in_select = false;
                select_lines.clear();
            }

            // Continue collecting SELECT lines
            if in_select {
                select_lines.push((line_idx, *line));
            }
        }

        // Check final SELECT block if query doesn't end with FROM/WHERE etc.
        if in_select && !select_lines.is_empty() {
            self.check_select_block(&select_lines, violations, node);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_trailing_comma_style(child, source, violations);
            }
        }
    }

    fn check_select_block(
        &self,
        select_lines: &[(usize, &str)],
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        if select_lines.len() < 2 {
            return; // Single line SELECT, no trailing comma possible
        }

        let mut has_trailing_commas = false;
        let mut has_leading_commas = false;
        let mut inconsistent_style = false;

        for (_line_idx, line) in select_lines.iter().skip(1) {
            // Skip first line with SELECT keyword
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue; // Skip empty lines and comments
            }

            // Check for trailing comma (line ends with comma)
            if trimmed.ends_with(',') {
                has_trailing_commas = true;
            }

            // Check for leading comma (line starts with comma)
            if trimmed.starts_with(',') {
                has_leading_commas = true;
            }

            // Check for inconsistent style within the same SELECT
            if has_trailing_commas && has_leading_commas {
                inconsistent_style = true;
                break;
            }
        }

        if inconsistent_style {
            let (first_line_idx, _) = select_lines[0];
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + first_line_idx + 1,
                column: start_pos.column + 1,
                message: "Inconsistent comma style in SELECT clause. Use either all trailing commas or all leading commas".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}
