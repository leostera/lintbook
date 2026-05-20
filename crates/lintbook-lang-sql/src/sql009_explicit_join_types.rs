use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

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

            let mut search_start = 0;
            while let Some(relative_pos) = lower_line[search_start..].find("join") {
                let join_pos = search_start + relative_pos;
                let before = &lower_line[..join_pos];
                let after = &lower_line[join_pos + 4..];
                let is_keyword = (before.is_empty()
                    || before.ends_with(|c: char| !c.is_alphanumeric()))
                    && (after.is_empty() || after.starts_with(|c: char| !c.is_alphanumeric()));

                if is_keyword {
                    let previous_word = before.split_whitespace().last().unwrap_or("");
                    let is_explicit = matches!(
                        previous_word,
                        "inner" | "left" | "right" | "full" | "cross" | "outer"
                    );

                    if !is_explicit {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + join_pos + 1,
                            message: "Use explicit JOIN type such as INNER JOIN or LEFT JOIN instead of plain JOIN".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }

                search_start = join_pos + 4;
            }
        }
    }
}
