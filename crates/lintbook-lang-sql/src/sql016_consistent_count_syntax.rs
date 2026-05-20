use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct ConsistentCountSyntax;

impl Rule for ConsistentCountSyntax {
    fn id(&self) -> &'static str {
        "SQL016"
    }

    fn name(&self) -> &'static str {
        "consistent-count-syntax"
    }

    fn description(&self) -> &'static str {
        "Use consistent COUNT syntax throughout the codebase"
    }

    fn explanation(&self) -> &'static str {
        "COUNT(*), COUNT(1), and COUNT(0) all count rows, but using consistent syntax improves readability.
        COUNT(*) is generally preferred as it clearly expresses the intent to count rows."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut count_patterns = Vec::new();

        // First pass: collect all COUNT patterns
        self.collect_count_patterns(tree.root_node(), source, &mut count_patterns);

        // Determine preferred pattern (most common)
        if count_patterns.is_empty() {
            return violations;
        }

        let mut pattern_counts = std::collections::HashMap::new();
        for pattern in &count_patterns {
            *pattern_counts.entry(pattern.pattern.clone()).or_insert(0) += 1;
        }

        let preferred_pattern = pattern_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(pattern, _)| pattern.clone())
            .unwrap_or_else(|| "count(*)".to_string());

        // Second pass: report inconsistencies
        for pattern_info in count_patterns {
            if pattern_info.pattern != preferred_pattern {
                violations.push(LintViolation {
                    line: pattern_info.line,
                    column: pattern_info.column,
                    message: format!(
                        "Inconsistent COUNT syntax '{}'. Use '{}' for consistency",
                        pattern_info.pattern.to_uppercase(),
                        preferred_pattern.to_uppercase()
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
struct CountPattern {
    pattern: String,
    line: usize,
    column: usize,
}

impl ConsistentCountSyntax {
    fn collect_count_patterns(
        &self,
        node: tree_sitter::Node,
        source: &str,
        patterns: &mut Vec<CountPattern>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Find COUNT patterns
            let mut pos = 0;
            while let Some(count_pos) = lower_line[pos..].find("count(") {
                let actual_pos = pos + count_pos;

                // Extract the full COUNT expression
                if let Some(close_paren) = lower_line[actual_pos..].find(')') {
                    let count_expr = &lower_line[actual_pos..actual_pos + close_paren + 1];

                    // Check for common COUNT patterns
                    if matches!(count_expr, "count(*)" | "count(1)" | "count(0)") {
                        let start_pos = node.start_position();
                        patterns.push(CountPattern {
                            pattern: count_expr.to_string(),
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + actual_pos + 1,
                        });
                    }

                    pos = actual_pos + close_paren + 1;
                } else {
                    break;
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_count_patterns(child, source, patterns);
            }
        }
    }
}
