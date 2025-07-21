use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ConsistentOrderByDirections;

impl Rule for ConsistentOrderByDirections {
    fn id(&self) -> &'static str {
        "SQL021"
    }

    fn name(&self) -> &'static str {
        "consistent-order-by-directions"
    }

    fn description(&self) -> &'static str {
        "Use explicit ASC/DESC in ORDER BY clauses for consistency"
    }

    fn explanation(&self) -> &'static str {
        "While ASC is the default for ORDER BY, explicitly specifying ASC or DESC 
        for all columns improves readability and makes sort intentions clear."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_order_by_clauses(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ConsistentOrderByDirections {
    fn check_order_by_clauses(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            if lower_line.contains("order by") {
                self.check_order_by_line(line, line_idx, violations, node);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_order_by_clauses(child, source, violations);
            }
        }
    }

    fn check_order_by_line(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        let lower_line = line.to_lowercase();

        if let Some(order_by_pos) = lower_line.find("order by") {
            let after_order_by = &line[order_by_pos + 8..]; // Skip "ORDER BY"

            // Extract the ORDER BY clause (until next SQL keyword)
            let end_pos = after_order_by
                .to_lowercase()
                .find(" limit ")
                .or_else(|| after_order_by.to_lowercase().find(" offset "))
                .or_else(|| after_order_by.to_lowercase().find(" union "))
                .or_else(|| after_order_by.to_lowercase().find(" except "))
                .or_else(|| after_order_by.to_lowercase().find(" intersect "))
                .unwrap_or(after_order_by.len());

            let order_clause = &after_order_by[..end_pos].trim();

            // Split by comma to get individual column specifications
            let columns: Vec<&str> = order_clause.split(',').collect();

            let mut has_explicit_direction = false;
            let mut has_implicit_direction = false;

            for column in columns {
                let trimmed = column.trim();
                let words: Vec<&str> = trimmed.split_whitespace().collect();

                if words.is_empty() {
                    continue;
                }

                // Check if last word is ASC or DESC
                if let Some(last_word) = words.last() {
                    let lower_last = last_word.to_lowercase();
                    if matches!(lower_last.as_str(), "asc" | "desc") {
                        has_explicit_direction = true;
                    } else {
                        has_implicit_direction = true;
                    }
                }
            }

            // Report inconsistency if we have both explicit and implicit directions
            if has_explicit_direction && has_implicit_direction {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + order_by_pos + 1,
                    message: "ORDER BY clause has inconsistent direction specification. Either specify ASC/DESC for all columns or none".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Optionally report if no explicit directions are used (uncomment if you want to enforce explicit directions)
            /*
            if has_implicit_direction && !has_explicit_direction {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + order_by_pos + 1,
                    message: "ORDER BY clause should explicitly specify ASC or DESC for all columns".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
            */
        }
    }
}
