use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ExplicitUnionAll;

impl Rule for ExplicitUnionAll {
    fn id(&self) -> &'static str {
        "SQL008"
    }

    fn name(&self) -> &'static str {
        "explicit-union-all"
    }

    fn description(&self) -> &'static str {
        "Use explicit UNION ALL or UNION DISTINCT instead of plain UNION"
    }

    fn explanation(&self) -> &'static str {
        "Plain UNION implicitly removes duplicates (like UNION DISTINCT), which can be expensive.
        Make the behavior explicit by using UNION ALL (faster, keeps duplicates) or UNION DISTINCT."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_union_statements(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ExplicitUnionAll {
    fn check_union_statements(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Look for standalone UNION (not followed by ALL or DISTINCT)
            if let Some(union_pos) = lower_line.find("union") {
                // Get text after UNION keyword
                let after_union = &lower_line[union_pos + 5..].trim_start();

                // Check if UNION is not followed by ALL or DISTINCT
                if !after_union.starts_with("all") && !after_union.starts_with("distinct") {
                    // Make sure it's actually the UNION keyword and not part of another word
                    let before_union = if union_pos > 0 {
                        &lower_line[..union_pos]
                    } else {
                        ""
                    };

                    // Check if UNION is a standalone word
                    let is_standalone = (before_union.is_empty()
                        || before_union.ends_with(|c: char| !c.is_alphanumeric()))
                        && (after_union.is_empty()
                            || after_union.starts_with(|c: char| !c.is_alphanumeric()));

                    if is_standalone {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + union_pos + 1,
                            message: "Use explicit 'UNION ALL' or 'UNION DISTINCT' instead of plain 'UNION'".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_union_statements(child, source, violations);
            }
        }
    }
}
