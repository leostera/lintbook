use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct UnnecessaryDistinct;

impl Rule for UnnecessaryDistinct {
    fn id(&self) -> &'static str {
        "SQL046"
    }

    fn name(&self) -> &'static str {
        "unnecessary-distinct"
    }

    fn description(&self) -> &'static str {
        "Avoid unnecessary DISTINCT when results are already unique"
    }

    fn explanation(&self) -> &'static str {
        "DISTINCT can be expensive and is often unnecessary. Remove DISTINCT when:
        1) Selecting from a primary key or unique constraint
        2) Using GROUP BY which already ensures uniqueness
        3) Results are inherently unique due to query structure"
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_distinct_usage(tree.root_node(), source, &mut violations);

        violations
    }
}

impl UnnecessaryDistinct {
    fn check_distinct_usage(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut query_context = QueryContext::new();
        let mut current_query_lines = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Start of new query
            if lower_line.trim().starts_with("select ") {
                if !current_query_lines.is_empty() {
                    self.analyze_query(&current_query_lines, &query_context, node, violations);
                }
                current_query_lines.clear();
                query_context = QueryContext::new();
            }

            current_query_lines.push((line_idx, *line));

            // Track query components
            if lower_line.contains("select distinct") {
                query_context.has_distinct = true;
                query_context.distinct_line = Some(line_idx);
            }

            if lower_line.contains(" group by ") {
                query_context.has_group_by = true;
            }

            if lower_line.contains(" from ") {
                query_context.extract_tables(&lower_line);
            }

            // Track unique constraints (simplified detection)
            self.track_unique_columns(line, &mut query_context);
        }

        // Don't forget the last query
        if !current_query_lines.is_empty() {
            self.analyze_query(&current_query_lines, &query_context, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_distinct_usage(child, source, violations);
            }
        }
    }

    fn analyze_query(
        &self,
        query_lines: &[(usize, &str)],
        context: &QueryContext,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if !context.has_distinct {
            return;
        }

        let mut reasons = Vec::new();

        // Check if DISTINCT is unnecessary due to GROUP BY
        if context.has_group_by {
            reasons.push("Query uses GROUP BY which already ensures uniqueness");
        }

        // Check if selecting primary key or ID columns
        if self.selects_unique_columns(query_lines) {
            reasons.push("Query selects primary key or unique columns");
        }

        // Check if it's a simple aggregation
        if self.is_simple_aggregation(query_lines) {
            reasons.push("Aggregation functions already produce unique results");
        }

        if !reasons.is_empty() {
            if let Some(distinct_line) = context.distinct_line {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + distinct_line + 1,
                    column: start_pos.column + 1,
                    message: format!("DISTINCT is unnecessary: {}", reasons.join(", ")),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn selects_unique_columns(&self, query_lines: &[(usize, &str)]) -> bool {
        // Look for common ID/key patterns in SELECT
        for (_, line) in query_lines {
            let lower_line = line.to_lowercase();
            if lower_line.contains("select ") {
                // Extract selected columns
                if let Some(select_pos) = lower_line.find("select ") {
                    let after_select = &line[select_pos + 7..];
                    if let Some(from_pos) = after_select.to_lowercase().find(" from ") {
                        let columns_part = &after_select[..from_pos];

                        // Check for ID patterns
                        let id_patterns = ["id", "pk", "primary_key", "uuid"];
                        for pattern in id_patterns.iter() {
                            if columns_part.to_lowercase().contains(pattern) {
                                // Make sure it's not just part of another word
                                if self.is_standalone_column(columns_part, pattern) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn is_simple_aggregation(&self, query_lines: &[(usize, &str)]) -> bool {
        // Check if query only contains aggregation functions
        let mut has_aggregation = false;
        let mut has_non_agg_columns = false;

        for (_, line) in query_lines {
            let lower_line = line.to_lowercase();
            if lower_line.contains("select ") {
                // Check for aggregation functions
                let agg_functions = ["count(", "sum(", "avg(", "max(", "min(", "group_concat("];
                for func in agg_functions.iter() {
                    if lower_line.contains(func) {
                        has_aggregation = true;
                        break;
                    }
                }

                // Simple check for non-aggregated columns
                if lower_line.contains("select ") && !lower_line.contains("*") {
                    if let Some(select_pos) = lower_line.find("select ") {
                        let after_select = &line[select_pos + 7..];
                        if let Some(from_pos) = after_select.to_lowercase().find(" from ") {
                            let columns_part = &after_select[..from_pos];
                            // If there are regular column names that don't look like functions
                            if columns_part.contains(",")
                                || (!columns_part.contains("(") && columns_part.trim() != "*")
                            {
                                has_non_agg_columns = true;
                            }
                        }
                    }
                }
            }
        }

        has_aggregation && !has_non_agg_columns
    }

    fn is_standalone_column(&self, text: &str, pattern: &str) -> bool {
        // Simple check to see if pattern appears as a standalone word
        let words: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .collect();
        words.iter().any(|word| word.to_lowercase() == pattern)
    }

    fn track_unique_columns(&self, line: &str, context: &mut QueryContext) {
        // Track if we see hints about unique constraints
        let lower_line = line.to_lowercase();
        if lower_line.contains("primary key")
            || lower_line.contains("unique")
            || lower_line.contains("constraint")
        {
            context.has_unique_hints = true;
        }
    }
}

#[derive(Debug)]
struct QueryContext {
    has_distinct: bool,
    has_group_by: bool,
    distinct_line: Option<usize>,
    tables: Vec<String>,
    has_unique_hints: bool,
}

impl QueryContext {
    fn new() -> Self {
        Self {
            has_distinct: false,
            has_group_by: false,
            distinct_line: None,
            tables: Vec::new(),
            has_unique_hints: false,
        }
    }

    fn extract_tables(&mut self, line: &str) {
        // Simple table extraction from FROM clause
        if let Some(from_pos) = line.find(" from ") {
            let after_from = &line[from_pos + 6..];
            let words: Vec<&str> = after_from.split_whitespace().collect();
            if !words.is_empty() {
                self.tables.push(words[0].to_string());
            }
        }
    }
}
