use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct KeywordCapitalization;

impl Rule for KeywordCapitalization {
    fn id(&self) -> &'static str {
        "SQL003"
    }

    fn name(&self) -> &'static str {
        "keyword-capitalization"
    }

    fn description(&self) -> &'static str {
        "SQL keywords should be consistently capitalized"
    }

    fn explanation(&self) -> &'static str {
        "SQL keywords should follow consistent capitalization policy.
        By default, keywords should be uppercase (SELECT, FROM, WHERE, etc.)"
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_keywords(tree.root_node(), source, &mut violations);

        violations
    }
}

impl KeywordCapitalization {
    fn check_keywords(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let sql_keywords = [
            "select",
            "from",
            "where",
            "join",
            "inner",
            "left",
            "right",
            "outer",
            "full",
            "on",
            "group",
            "by",
            "order",
            "having",
            "union",
            "all",
            "distinct",
            "as",
            "insert",
            "into",
            "values",
            "update",
            "set",
            "delete",
            "create",
            "table",
            "alter",
            "drop",
            "index",
            "view",
            "database",
            "schema",
            "primary",
            "key",
            "foreign",
            "references",
            "constraint",
            "null",
            "not",
            "and",
            "or",
            "in",
            "between",
            "like",
            "exists",
            "case",
            "when",
            "then",
            "else",
            "end",
            "if",
            "count",
            "sum",
            "avg",
            "max",
            "min",
            "coalesce",
            "ifnull",
            "nvl",
            "cast",
            "convert",
            "substring",
            "concat",
            "trim",
            "upper",
            "lower",
            "length",
            "is",
            "true",
            "false",
        ];

        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let words: Vec<&str> = line.split_whitespace().collect();

            for (word_idx, word) in words.iter().enumerate() {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                let lower_word = clean_word.to_lowercase();

                if sql_keywords.contains(&lower_word.as_str())
                    && clean_word != clean_word.to_uppercase()
                {
                    // Calculate column position
                    let line_before_word: String = words[..word_idx].join(" ");
                    let col_position = if word_idx == 0 {
                        0
                    } else {
                        line_before_word.len() + 1
                    };

                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + col_position + 1,
                        message: format!(
                            "SQL keyword '{}' should be uppercase: '{}'",
                            clean_word,
                            clean_word.to_uppercase()
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_keywords(child, source, violations);
            }
        }
    }
}
