use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct NestedSubqueries;

impl Rule for NestedSubqueries {
    fn id(&self) -> &'static str {
        "SQL038"
    }

    fn name(&self) -> &'static str {
        "nested-subqueries"
    }

    fn description(&self) -> &'static str {
        "Avoid deeply nested subqueries"
    }

    fn explanation(&self) -> &'static str {
        "Deeply nested subqueries are hard to read and maintain. Consider refactoring
        into CTEs (Common Table Expressions) or joins. Maximum recommended nesting
        level is 2. CTEs improve readability and can be reused within the query."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_subquery_nesting(tree.root_node(), source, &mut violations);

        violations
    }
}

impl NestedSubqueries {
    fn check_subquery_nesting(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];

        // Track parentheses depth and SELECT depth
        let mut paren_depth = 0;
        let mut select_depth = 0;
        let mut max_select_depth = 0;
        let mut select_positions = Vec::new();

        let chars: Vec<char> = node_text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Skip string literals
            if chars[i] == '\'' || chars[i] == '"' {
                let quote = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                i += 1;
                continue;
            }

            // Skip comments
            if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }

            // Track parentheses
            if chars[i] == '(' {
                paren_depth += 1;

                // Check if this starts a subquery (SELECT after parenthesis)
                let remaining = chars[i + 1..]
                    .iter()
                    .collect::<String>()
                    .trim_start()
                    .to_lowercase();
                if remaining.starts_with("select ") {
                    select_depth += 1;
                    if select_depth > max_select_depth {
                        max_select_depth = select_depth;
                    }

                    // Find line number for this position
                    let mut line_num = 0;
                    let mut char_count = 0;
                    for ch in chars[..=i].iter() {
                        if *ch == '\n' {
                            line_num += 1;
                        }
                        char_count += 1;
                    }

                    select_positions.push((line_num, select_depth, char_count));

                    // Report violation if too deep
                    if select_depth > 2 {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_num + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "Subquery nested {} levels deep. Maximum recommended nesting is 2 levels. Consider using CTEs instead",
                                select_depth
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            } else if chars[i] == ')' {
                paren_depth -= 1;

                // Check if we're exiting a subquery level
                if !select_positions.is_empty() {
                    let last_select = select_positions.last().unwrap();
                    if paren_depth < last_select.1 {
                        select_depth -= 1;
                        select_positions.pop();
                    }
                }
            }

            i += 1;
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_subquery_nesting(child, source, violations);
            }
        }
    }
}
