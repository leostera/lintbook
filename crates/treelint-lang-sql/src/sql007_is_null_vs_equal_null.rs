use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct IsNullVsEqualNull;

impl Rule for IsNullVsEqualNull {
    fn id(&self) -> &'static str {
        "SQL007"
    }

    fn name(&self) -> &'static str {
        "is-null-vs-equal-null"
    }

    fn description(&self) -> &'static str {
        "Use IS NULL instead of = NULL for null comparisons"
    }

    fn explanation(&self) -> &'static str {
        "In SQL, comparing with NULL using = or != will always return NULL (unknown), not TRUE or FALSE. 
        Use IS NULL or IS NOT NULL for proper null comparisons."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_null_comparisons(tree.root_node(), source, &mut violations);

        violations
    }
}

impl IsNullVsEqualNull {
    fn check_null_comparisons(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Check for = NULL pattern
            if let Some(pos) = lower_line.find("= null") {
                if !lower_line[..pos].ends_with("!") {
                    // Skip != NULL (handled separately)
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pos + 1,
                        message: "Use 'IS NULL' instead of '= NULL' for null comparison"
                            .to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }

            // Check for != NULL pattern
            if lower_line.contains("!= null") {
                if let Some(pos) = lower_line.find("!= null") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pos + 1,
                        message: "Use 'IS NOT NULL' instead of '!= NULL' for null comparison"
                            .to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }

            // Check for <> NULL pattern
            if lower_line.contains("<> null") {
                if let Some(pos) = lower_line.find("<> null") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pos + 1,
                        message: "Use 'IS NOT NULL' instead of '<> NULL' for null comparison"
                            .to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_null_comparisons(child, source, violations);
            }
        }
    }
}
