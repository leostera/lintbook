use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct WildcardColumnAmbiguity;

impl Rule for WildcardColumnAmbiguity {
    fn id(&self) -> &'static str {
        "SQL024"
    }

    fn name(&self) -> &'static str {
        "wildcard-column-ambiguity"
    }

    fn description(&self) -> &'static str {
        "Avoid SELECT * in joins or when column count matters"
    }

    fn explanation(&self) -> &'static str {
        "SELECT * can be ambiguous when used with JOINs as it may return duplicate columns 
        or unexpected column counts. Be explicit about which columns you need."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_wildcard_usage(tree.root_node(), source, &mut violations);

        violations
    }
}

impl WildcardColumnAmbiguity {
    fn check_wildcard_usage(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lower_text = node_text.to_lowercase();

        // Check if query has both SELECT * and JOINs
        if lower_text.contains("select *")
            && (lower_text.contains(" join ")
                || lower_text.contains(" inner ")
                || lower_text.contains(" left ")
                || lower_text.contains(" right ")
                || lower_text.contains(" full ")
                || lower_text.contains(" cross "))
        {
            let lines: Vec<&str> = node_text.split('\n').collect();

            for (line_idx, line) in lines.iter().enumerate() {
                let lower_line = line.to_lowercase();

                if lower_line.contains("select *") {
                    if let Some(pos) = lower_line.find("select *") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + pos + 8, // Position at the *
                            message: "Avoid SELECT * in queries with JOINs. Specify explicit column names to prevent ambiguity".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }

        // Check for qualified wildcards that might still be ambiguous
        let lines: Vec<&str> = node_text.split('\n').collect();
        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains(".*") && lower_text.contains(" join ") {
                // Find table.* patterns
                let words: Vec<&str> = line.split_whitespace().collect();
                for word in words {
                    if word.contains(".*") && word.len() > 2 {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "Consider being explicit about columns instead of using '{}' in JOIN queries",
                                word
                            ),
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
                self.check_wildcard_usage(child, source, violations);
            }
        }
    }
}
