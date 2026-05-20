use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct NotEqualOperatorConsistency;

impl Rule for NotEqualOperatorConsistency {
    fn id(&self) -> &'static str {
        "SQL005"
    }

    fn name(&self) -> &'static str {
        "not-equal-operator-consistency"
    }

    fn description(&self) -> &'static str {
        "Use consistent not-equal operators throughout the codebase"
    }

    fn explanation(&self) -> &'static str {
        "SQL supports both != and <> for not-equal comparisons.
        Choose one style and use it consistently. Standard SQL prefers <> but != is widely supported."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut found_operators = Vec::new();

        // First pass: collect all not-equal operators
        self.collect_operators(tree.root_node(), source, &mut found_operators);

        // Determine the preferred operator (the first one found or most common)
        if found_operators.is_empty() {
            return violations;
        }

        let preferred_op = if found_operators
            .iter()
            .filter(|op| op.operator == "!=")
            .count()
            > found_operators
                .iter()
                .filter(|op| op.operator == "<>")
                .count()
        {
            "!="
        } else {
            "<>"
        };

        // Second pass: report inconsistencies
        for op_info in found_operators {
            if op_info.operator != preferred_op {
                violations.push(LintViolation {
                    line: op_info.line,
                    column: op_info.column,
                    message: format!(
                        "Inconsistent not-equal operator '{}'. Use '{}' for consistency",
                        op_info.operator, preferred_op
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        violations
    }
}

#[derive(Debug)]
struct OperatorInfo {
    operator: String,
    line: usize,
    column: usize,
}

impl NotEqualOperatorConsistency {
    fn collect_operators(
        &self,
        node: tree_sitter::Node,
        source: &str,
        operators: &mut Vec<OperatorInfo>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Find != operators
            let mut pos = 0;
            while let Some(found_pos) = line[pos..].find("!=") {
                let actual_pos = pos + found_pos;
                let start_pos = node.start_position();
                operators.push(OperatorInfo {
                    operator: "!=".to_string(),
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + actual_pos + 1,
                });
                pos = actual_pos + 2;
            }

            // Find <> operators
            let mut pos = 0;
            while let Some(found_pos) = line[pos..].find("<>") {
                let actual_pos = pos + found_pos;
                let start_pos = node.start_position();
                operators.push(OperatorInfo {
                    operator: "<>".to_string(),
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + actual_pos + 1,
                });
                pos = actual_pos + 2;
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_operators(child, source, operators);
            }
        }
    }
}
