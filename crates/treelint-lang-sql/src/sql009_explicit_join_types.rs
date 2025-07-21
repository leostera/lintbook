use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ExplicitJoinTypes;

impl Rule for ExplicitJoinTypes {
    fn id(&self) -> &'static str {
        "SQL009"
    }

    fn name(&self) -> &'static str {
        "explicit-join-types"
    }

    fn description(&self) -> &'static str {
        "Use explicit JOIN types (INNER JOIN, LEFT JOIN) instead of implicit joins"
    }

    fn explanation(&self) -> &'static str {
        "Explicit JOIN syntax is clearer and less error-prone than implicit joins using comma separation.
        Use INNER JOIN, LEFT JOIN, RIGHT JOIN, or FULL OUTER JOIN instead of listing tables with commas."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_implicit_joins(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ExplicitJoinTypes {
    fn check_implicit_joins(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Look for FROM clause with comma-separated tables (implicit join)
            if let Some(from_pos) = lower_line.find("from ") {
                let from_clause = &lower_line[from_pos..];

                // Find the end of the FROM clause (before WHERE, GROUP BY, ORDER BY, etc.)
                let clause_end = from_clause
                    .find(" where ")
                    .or_else(|| from_clause.find(" group "))
                    .or_else(|| from_clause.find(" order "))
                    .or_else(|| from_clause.find(" having "))
                    .or_else(|| from_clause.find(" limit "))
                    .unwrap_or(from_clause.len());

                let from_clause = &from_clause[..clause_end];

                // Check if there are commas in the FROM clause (but not in function calls)
                if from_clause.contains(',') && !from_clause.contains("join") {
                    // Count parentheses to avoid flagging commas inside function calls
                    let mut paren_count = 0;
                    let mut has_implicit_join = false;

                    for ch in from_clause.chars() {
                        match ch {
                            '(' => paren_count += 1,
                            ')' => paren_count -= 1,
                            ',' if paren_count == 0 => {
                                has_implicit_join = true;
                                break;
                            }
                            _ => {}
                        }
                    }

                    if has_implicit_join {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + from_pos + 1,
                            message: "Use explicit JOIN syntax instead of comma-separated tables in FROM clause".to_string(),
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
                self.check_implicit_joins(child, source, violations);
            }
        }
    }
}
