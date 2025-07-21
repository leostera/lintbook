use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct DistinctGroupByConflict;

impl Rule for DistinctGroupByConflict {
    fn id(&self) -> &'static str {
        "SQL017"
    }

    fn name(&self) -> &'static str {
        "distinct-group-by-conflict"
    }

    fn description(&self) -> &'static str {
        "DISTINCT and GROUP BY serve similar purposes and may be redundant"
    }

    fn explanation(&self) -> &'static str {
        "Using DISTINCT with GROUP BY can be redundant since GROUP BY already eliminates duplicates 
        for the grouped columns. Consider removing DISTINCT or restructuring the query."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_distinct_group_by(tree.root_node(), source, &mut violations);

        violations
    }
}

impl DistinctGroupByConflict {
    fn check_distinct_group_by(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lower_text = node_text.to_lowercase();

        // Check if query contains both DISTINCT and GROUP BY
        if lower_text.contains("select distinct") && lower_text.contains("group by") {
            let lines: Vec<&str> = node_text.split('\n').collect();

            for (line_idx, line) in lines.iter().enumerate() {
                let lower_line = line.to_lowercase();

                if lower_line.contains("select distinct") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Query uses both DISTINCT and GROUP BY, which may be redundant. Consider removing DISTINCT or restructuring the query".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                    break; // Only report once per query
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_distinct_group_by(child, source, violations);
            }
        }
    }
}
