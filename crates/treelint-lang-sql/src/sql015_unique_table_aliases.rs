use std::collections::HashMap;
use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct UniqueTableAliases;

impl Rule for UniqueTableAliases {
    fn id(&self) -> &'static str {
        "SQL015"
    }

    fn name(&self) -> &'static str {
        "unique-table-aliases"
    }

    fn description(&self) -> &'static str {
        "Table aliases must be unique within a query"
    }

    fn explanation(&self) -> &'static str {
        "Each table alias in a query must be unique to avoid ambiguity in column references. 
        Duplicate aliases can cause confusion and potential errors."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_query_aliases(tree.root_node(), source, &mut violations);

        violations
    }
}

impl UniqueTableAliases {
    fn check_query_aliases(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];

        // Extract aliases from the query
        let mut alias_locations: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        self.extract_aliases(node_text, &mut alias_locations, node);

        // Check for duplicates
        for (alias, locations) in alias_locations {
            if locations.len() > 1 {
                for (line, col) in locations {
                    violations.push(LintViolation {
                        line,
                        column: col,
                        message: format!(
                            "Duplicate table alias '{}'. Table aliases must be unique within a query",
                            alias
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Recursively check child nodes for nested queries
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_query_aliases(child, source, violations);
            }
        }
    }

    fn extract_aliases(
        &self,
        query: &str,
        aliases: &mut HashMap<String, Vec<(usize, usize)>>,
        node: tree_sitter::Node,
    ) {
        let lines: Vec<&str> = query.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Look for FROM clause and JOINs
            if lower_line.contains("from ") || lower_line.contains("join ") {
                self.extract_aliases_from_line(line, line_idx, aliases, node);
            }
        }
    }

    fn extract_aliases_from_line(
        &self,
        line: &str,
        line_idx: usize,
        aliases: &mut HashMap<String, Vec<(usize, usize)>>,
        node: tree_sitter::Node,
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;

        while i < words.len() {
            let word = words[i].to_lowercase();

            // Skip to table name after FROM or JOIN
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
                            // Explicit alias: table AS alias
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
                            // Implicit alias: table alias
                            next_word
                        } else {
                            i += 1;
                            continue;
                        };

                        // Clean up alias (remove punctuation)
                        let clean_alias =
                            alias.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');

                        if !clean_alias.is_empty() && clean_alias != table_name {
                            let start_pos = node.start_position();
                            aliases
                                .entry(clean_alias.to_lowercase())
                                .or_insert_with(Vec::new)
                                .push((start_pos.row + line_idx + 1, start_pos.column + 1));
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
    }
}
