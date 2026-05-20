use std::collections::{HashMap, HashSet};
use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct UnusedTableAliases;

impl Rule for UnusedTableAliases {
    fn id(&self) -> &'static str {
        "SQL019"
    }

    fn name(&self) -> &'static str {
        "unused-table-aliases"
    }

    fn description(&self) -> &'static str {
        "Remove unused table aliases to improve code clarity"
    }

    fn explanation(&self) -> &'static str {
        "Table aliases that are defined but never referenced make queries harder to read.
        Remove unused aliases or use them consistently for column references."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_alias_usage(tree.root_node(), source, &mut violations);

        violations
    }
}

impl UnusedTableAliases {
    fn check_alias_usage(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];

        // Extract aliases and their usage
        let mut declared_aliases: HashMap<String, (usize, usize)> = HashMap::new();
        let mut used_aliases: HashSet<String> = HashSet::new();

        self.extract_declared_aliases(node_text, &mut declared_aliases, node);
        self.extract_used_aliases(node_text, &mut used_aliases);

        // Find unused aliases
        for (alias, (line, col)) in declared_aliases {
            if !used_aliases.contains(&alias) {
                violations.push(LintViolation {
                    line,
                    column: col,
                    message: format!(
                        "Table alias '{}' is declared but never used. Consider removing it or using it for column references",
                        alias
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_alias_usage(child, source, violations);
            }
        }
    }

    fn extract_declared_aliases(
        &self,
        query: &str,
        aliases: &mut HashMap<String, (usize, usize)>,
        node: tree_sitter::Node,
    ) {
        let lines: Vec<&str> = query.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            if lower_line.contains("from ") || lower_line.contains("join ") {
                self.extract_aliases_from_line(line, line_idx, aliases, node);
            }
        }
    }

    fn extract_aliases_from_line(
        &self,
        line: &str,
        line_idx: usize,
        aliases: &mut HashMap<String, (usize, usize)>,
        node: tree_sitter::Node,
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;

        while i < words.len() {
            let word = words[i].to_lowercase();

            if matches!(
                word.as_str(),
                "from" | "join" | "inner" | "left" | "right" | "full" | "cross"
            ) {
                i += 1;
                if i < words.len() {
                    let table_name = words[i];
                    i += 1;

                    // Check for alias
                    if i < words.len() {
                        let next_word = words[i];
                        let alias = if next_word.to_lowercase() == "as" {
                            i += 1;
                            if i < words.len() {
                                words[i]
                            } else {
                                i += 1;
                                continue;
                            }
                        } else if !matches!(
                            next_word.to_lowercase().as_str(),
                            "join"
                                | "inner"
                                | "left"
                                | "right"
                                | "full"
                                | "outer"
                                | "cross"
                                | "on"
                                | "where"
                                | "group"
                                | "order"
                                | "having"
                                | "limit"
                        ) {
                            next_word
                        } else {
                            i += 1;
                            continue;
                        };

                        let clean_alias =
                            alias.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');

                        if !clean_alias.is_empty() && clean_alias != table_name {
                            let start_pos = node.start_position();
                            aliases.insert(
                                clean_alias.to_string(),
                                (start_pos.row + line_idx + 1, start_pos.column + 1),
                            );
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    fn extract_used_aliases(&self, query: &str, used_aliases: &mut HashSet<String>) {
        let lines: Vec<&str> = query.split('\n').collect();

        for line in lines {
            // Look for table.column references
            let words: Vec<&str> = line.split_whitespace().collect();

            for word in words {
                if word.contains('.') && !word.starts_with('(') {
                    let parts: Vec<&str> = word.split('.').collect();
                    if parts.len() >= 2 {
                        let table_ref =
                            parts[0].trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                        if !table_ref.is_empty() {
                            // Skip common non-table prefixes
                            let non_table_prefixes = ["extract", "date", "time"];
                            if !non_table_prefixes.contains(&table_ref.to_lowercase().as_str()) {
                                used_aliases.insert(table_ref.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}
