use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ColumnNamesInGroupOrderBy;

impl Rule for ColumnNamesInGroupOrderBy {
    fn id(&self) -> &'static str {
        "SQL028"
    }

    fn name(&self) -> &'static str {
        "column-names-in-group-order-by"
    }

    fn description(&self) -> &'static str {
        "Use column names instead of positional references in GROUP BY/ORDER BY"
    }

    fn explanation(&self) -> &'static str {
        "Using positional references (1, 2, 3) in GROUP BY and ORDER BY clauses is fragile 
        and harder to understand. Use explicit column names or aliases for better readability 
        and maintainability."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_positional_references(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ColumnNamesInGroupOrderBy {
    fn check_positional_references(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        let mut in_group_by = false;
        let mut in_order_by = false;
        let mut clause_lines = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            let trimmed = line.trim();
            
            // Start of GROUP BY clause
            if lower_line.contains(" group by ") {
                in_group_by = true;
                in_order_by = false;
                clause_lines.clear();
                // Extract content after GROUP BY on same line
                if let Some(pos) = lower_line.find(" group by ") {
                    let content_after = &line[pos + 10..];
                    if !content_after.trim().is_empty() {
                        clause_lines.push((line_idx, content_after));
                    }
                }
                continue;
            }
            
            // Start of ORDER BY clause
            if lower_line.contains(" order by ") {
                in_order_by = true;
                in_group_by = false;
                clause_lines.clear();
                // Extract content after ORDER BY on same line
                if let Some(pos) = lower_line.find(" order by ") {
                    let content_after = &line[pos + 10..];
                    if !content_after.trim().is_empty() {
                        clause_lines.push((line_idx, content_after));
                    }
                }
                continue;
            }
            
            // End of clause
            if (in_group_by || in_order_by) && self.is_clause_end(&lower_line) {
                if !clause_lines.is_empty() {
                    let clause_type = if in_group_by { "GROUP BY" } else { "ORDER BY" };
                    self.check_clause_content(&clause_lines, clause_type, node, violations);
                }
                in_group_by = false;
                in_order_by = false;
                clause_lines.clear();
            }
            
            // Continue collecting clause lines
            if (in_group_by || in_order_by) && !trimmed.is_empty() {
                clause_lines.push((line_idx, *line));
            }
        }
        
        // Check final clause if query ends
        if (in_group_by || in_order_by) && !clause_lines.is_empty() {
            let clause_type = if in_group_by { "GROUP BY" } else { "ORDER BY" };
            self.check_clause_content(&clause_lines, clause_type, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_positional_references(child, source, violations);
            }
        }
    }
    
    fn is_clause_end(&self, lower_line: &str) -> bool {
        lower_line.contains(" having ") ||
        lower_line.contains(" order ") ||
        lower_line.contains(" limit ") ||
        lower_line.contains(" offset ") ||
        lower_line.contains(" union ") ||
        lower_line.contains(" except ") ||
        lower_line.contains(" intersect ") ||
        lower_line.contains(";")
    }
    
    fn check_clause_content(
        &self,
        clause_lines: &[(usize, &str)],
        clause_type: &str,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        for (line_idx, line) in clause_lines {
            let tokens: Vec<&str> = line.split(',')
                .flat_map(|part| part.split_whitespace())
                .collect();
            
            for token in tokens {
                let trimmed = token.trim();
                
                // Check if token is a positive integer (positional reference)
                if trimmed.chars().all(|c| c.is_ascii_digit()) && !trimmed.is_empty() {
                    if let Ok(num) = trimmed.parse::<u32>() {
                        if num > 0 && num < 100 { // Reasonable range for column positions
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "{} contains positional reference '{}'. Use column name instead",
                                    clause_type, num
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