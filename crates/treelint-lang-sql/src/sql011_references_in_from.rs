use std::collections::HashSet;
use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ReferencesInFrom;

impl Rule for ReferencesInFrom {
    fn id(&self) -> &'static str {
        "SQL011"
    }

    fn name(&self) -> &'static str {
        "references-in-from"
    }

    fn description(&self) -> &'static str {
        "All table references must be defined in FROM clause"
    }

    fn explanation(&self) -> &'static str {
        "Column references using table aliases must correspond to tables that are actually listed in the FROM clause.
        This prevents errors from typos in table aliases or missing table declarations."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_table_references(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ReferencesInFrom {
    fn check_table_references(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];

        // Extract tables from FROM clause
        let mut declared_tables = HashSet::new();
        self.extract_from_tables(node_text, &mut declared_tables);

        // Check for table references in SELECT, WHERE, etc.
        self.check_table_refs_in_query(node_text, &declared_tables, violations, node);

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_table_references(child, source, violations);
            }
        }
    }

    fn extract_from_tables(&self, query: &str, tables: &mut HashSet<String>) {
        let lower_query = query.to_lowercase();

        if let Some(from_pos) = lower_query.find("from ") {
            let from_part = &lower_query[from_pos + 5..];

            // Find the end of FROM clause
            let end_pos = from_part
                .find(" where ")
                .or_else(|| from_part.find(" group "))
                .or_else(|| from_part.find(" order "))
                .or_else(|| from_part.find(" having "))
                .or_else(|| from_part.find(" limit "))
                .unwrap_or(from_part.len());

            let from_clause = &from_part[..end_pos];

            // Parse table names and aliases
            let parts: Vec<&str> = from_clause.split_whitespace().collect();
            let mut i = 0;

            while i < parts.len() {
                let part = parts[i];

                // Skip JOIN keywords
                if matches!(
                    part,
                    "join" | "inner" | "left" | "right" | "full" | "outer" | "cross" | "on"
                ) {
                    i += 1;
                    continue;
                }

                // Table name found
                if !part.contains('(') && !part.contains(')') {
                    tables.insert(part.to_string());

                    // Check for alias
                    if i + 1 < parts.len() {
                        let next_part = parts[i + 1];
                        if next_part == "as" && i + 2 < parts.len() {
                            // Explicit alias: table AS alias
                            tables.insert(parts[i + 2].to_string());
                            i += 3;
                        } else if !matches!(
                            next_part,
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
                        ) {
                            // Implicit alias: table alias
                            tables.insert(next_part.to_string());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    fn check_table_refs_in_query(
        &self,
        query: &str,
        declared_tables: &HashSet<String>,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        let lines: Vec<&str> = query.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let words: Vec<&str> = line.split_whitespace().collect();

            for word in words {
                // Look for table.column references
                if word.contains('.') && !word.starts_with('(') {
                    let parts: Vec<&str> = word.split('.').collect();
                    if parts.len() >= 2 {
                        let table_ref =
                            parts[0].trim_matches(|c: char| !c.is_alphanumeric() && c != '_');

                        // Check if table reference exists in declared tables
                        if !table_ref.is_empty() && !declared_tables.contains(table_ref) {
                            // Skip common SQL functions and literals
                            let sql_functions =
                                ["count", "sum", "avg", "max", "min", "extract", "date"];
                            if !sql_functions.contains(&table_ref) {
                                let start_pos = node.start_position();
                                violations.push(LintViolation {
                                    line: start_pos.row + line_idx + 1,
                                    column: start_pos.column + 1,
                                    message: format!(
                                        "Table reference '{}' not found in FROM clause",
                                        table_ref
                                    ),
                                    lint_name: self.name().to_string(),
                                    lint_id: self.id().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
