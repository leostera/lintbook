use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct NestedCaseStatements;

impl Rule for NestedCaseStatements {
    fn id(&self) -> &'static str {
        "SQL033"
    }

    fn name(&self) -> &'static str {
        "nested-case-statements"
    }

    fn description(&self) -> &'static str {
        "Avoid deeply nested CASE statements"
    }

    fn explanation(&self) -> &'static str {
        "Deeply nested CASE statements are hard to read and maintain. Consider refactoring
        complex nested CASE logic into CTEs, derived tables, or separate columns.
        Maximum recommended nesting level is 2."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_case_nesting(tree.root_node(), source, &mut violations);

        violations
    }
}

impl NestedCaseStatements {
    fn check_case_nesting(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut case_stack = Vec::new();
        let mut max_nesting = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Count CASE keywords (start of CASE statement)
            let case_count = self.count_keyword_occurrences(&lower_line, "case");
            for _ in 0..case_count {
                case_stack.push(line_idx);
                if case_stack.len() > max_nesting {
                    max_nesting = case_stack.len();
                }
            }

            // Count END keywords (end of CASE statement)
            let end_count = self.count_keyword_occurrences(&lower_line, "end");
            for _ in 0..end_count {
                if !case_stack.is_empty() {
                    case_stack.pop();
                }
            }

            // Check for violation (more than 2 levels of nesting)
            if case_stack.len() > 2 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "CASE statement nested {} levels deep. Maximum recommended nesting is 2 levels",
                        case_stack.len()
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });

                // Only report once per deeply nested section
                while case_stack.len() > 2 {
                    case_stack.pop();
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_case_nesting(child, source, violations);
            }
        }
    }

    fn count_keyword_occurrences(&self, line: &str, keyword: &str) -> usize {
        let mut count = 0;

        // Split by word boundaries and count occurrences
        let words: Vec<&str> = line.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            // Check if this word starts with the keyword
            if word.to_lowercase().starts_with(keyword) {
                // Make sure it's not part of another word (e.g., "CASE" vs "CASEWHEN")
                let after_keyword = &word[keyword.len()..];
                if after_keyword.is_empty()
                    || after_keyword.starts_with(' ')
                    || after_keyword.starts_with('(')
                    || after_keyword.starts_with('\t')
                    || after_keyword.starts_with('\n')
                {
                    // Also check it's not in a string literal or identifier
                    if !self.is_in_string_or_identifier(&words, i) {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    fn is_in_string_or_identifier(&self, words: &[&str], word_index: usize) -> bool {
        // Simple heuristic: check if the previous word ends with a quote
        // or if we're inside quotes
        if word_index > 0 {
            let prev_word = words[word_index - 1];
            if prev_word.ends_with("'") || prev_word.ends_with("\"") {
                return true;
            }
        }

        // Check if the word itself is quoted
        let word = words[word_index];
        if word.starts_with("'") || word.starts_with("\"") {
            return true;
        }

        false
    }
}
