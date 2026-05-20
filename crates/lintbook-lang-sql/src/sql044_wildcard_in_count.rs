use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct WildcardInCount;

impl Rule for WildcardInCount {
    fn id(&self) -> &'static str {
        "SQL044"
    }

    fn name(&self) -> &'static str {
        "wildcard-in-count"
    }

    fn description(&self) -> &'static str {
        "Use COUNT(*) instead of COUNT(column) when counting all rows"
    }

    fn explanation(&self) -> &'static str {
        "When counting all rows, use COUNT(*) rather than COUNT(column_name). COUNT(*)
        counts all rows including those with NULL values, while COUNT(column) only counts
        non-NULL values. COUNT(*) is also often more performant and clearly expresses intent."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_count_usage(tree.root_node(), source, &mut violations);

        violations
    }
}

impl WildcardInCount {
    fn check_count_usage(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Look for COUNT functions
            let mut search_start = 0;
            while let Some(count_pos) = lower_line[search_start..].find("count(") {
                let absolute_pos = search_start + count_pos;

                // Extract the content inside COUNT()
                if let Some(content) = self.extract_count_content(line, absolute_pos + 6) {
                    self.analyze_count_usage(&content, line_idx, absolute_pos, node, violations);
                }

                search_start = absolute_pos + 6;
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_count_usage(child, source, violations);
            }
        }
    }

    fn extract_count_content(&self, line: &str, start_pos: usize) -> Option<String> {
        let chars: Vec<char> = line.chars().collect();
        let mut content = String::new();
        let mut paren_depth = 1; // We start after the opening parenthesis
        let mut i = start_pos;

        while i < chars.len() && paren_depth > 0 {
            match chars[i] {
                '(' => {
                    paren_depth += 1;
                    content.push(chars[i]);
                }
                ')' => {
                    paren_depth -= 1;
                    if paren_depth > 0 {
                        content.push(chars[i]);
                    }
                }
                _ => {
                    content.push(chars[i]);
                }
            }
            i += 1;
        }

        if paren_depth == 0 {
            Some(content.trim().to_string())
        } else {
            None
        }
    }

    fn analyze_count_usage(
        &self,
        content: &str,
        line_idx: usize,
        pos: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let trimmed = content.trim();

        // Skip if it's already COUNT(*)
        if trimmed == "*" {
            return;
        }

        // Skip if it's COUNT(DISTINCT ...)
        if trimmed.to_lowercase().starts_with("distinct ") {
            return;
        }

        // Check if it's a simple column reference that might be better as COUNT(*)
        if self.is_simple_column_reference(trimmed) {
            // Check context to see if this is likely meant to count all rows
            let suggestion = self.analyze_context_for_count_suggestion(trimmed);

            if suggestion.is_some() {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + pos + 1,
                    message: format!(
                        "Consider using COUNT(*) instead of COUNT({}). {}",
                        trimmed,
                        suggestion.unwrap_or_default()
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for COUNT(1) or COUNT(0) which should be COUNT(*)
        if trimmed == "1" || trimmed == "0" {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + pos + 1,
                message: format!(
                    "Use COUNT(*) instead of COUNT({}). COUNT(*) is clearer and standard",
                    trimmed
                ),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn is_simple_column_reference(&self, content: &str) -> bool {
        // Check if it's a simple column reference (not a complex expression)
        !content.contains('(')
            && !content.contains('+')
            && !content.contains('-')
            && !content.contains('*')
            && !content.contains('/')
            && !content.contains(" case ")
            && !content.to_lowercase().contains(" when ")
    }

    fn analyze_context_for_count_suggestion(&self, column: &str) -> Option<String> {
        // If it's a common primary key or ID column, suggest COUNT(*)
        let id_patterns = ["id", "pk", "key"];
        let lower_column = column.to_lowercase();

        for pattern in id_patterns.iter() {
            if lower_column.contains(pattern) {
                return Some(format!(
                    "If counting all rows, COUNT(*) is clearer. If checking non-NULL {} values, this is correct",
                    column
                ));
            }
        }

        // For other columns, provide general guidance
        Some(format!(
            "If counting total rows, use COUNT(*). If counting non-NULL {} values, this is correct",
            column
        ))
    }
}
