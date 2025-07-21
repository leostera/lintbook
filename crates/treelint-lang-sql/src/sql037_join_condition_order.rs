use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct JoinConditionOrder;

impl Rule for JoinConditionOrder {
    fn id(&self) -> &'static str {
        "SQL037"
    }

    fn name(&self) -> &'static str {
        "join-condition-order"
    }

    fn description(&self) -> &'static str {
        "Join conditions should reference tables in consistent order"
    }

    fn explanation(&self) -> &'static str {
        "In JOIN conditions, reference the earlier table first for consistency and readability. 
        For example, in 'FROM a JOIN b ON ...', use 'a.id = b.a_id' rather than 'b.a_id = a.id'. 
        This makes the data flow clearer."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_join_conditions(tree.root_node(), source, &mut violations);

        violations
    }
}

impl JoinConditionOrder {
    fn check_join_conditions(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        let mut table_order = Vec::new();
        let mut current_join_line = None;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            
            // Track table order from FROM clause
            if lower_line.contains(" from ") {
                if let Some(table) = self.extract_table_after_keyword(line, "from") {
                    table_order.clear();
                    table_order.push(table);
                }
            }
            
            // Track tables from JOIN clauses
            if lower_line.contains(" join ") {
                if let Some(table) = self.extract_table_after_keyword(line, "join") {
                    table_order.push(table);
                    current_join_line = Some(line_idx);
                }
            }
            
            // Check ON conditions
            if lower_line.contains(" on ") && !table_order.is_empty() {
                // Extract the condition after ON
                if let Some(on_pos) = lower_line.find(" on ") {
                    let condition = &line[on_pos + 4..];
                    
                    // Check if this is a continuation of the previous JOIN
                    if current_join_line.is_some() || lower_line.contains(" join ") {
                        self.check_condition_order(
                            condition,
                            &table_order,
                            line_idx,
                            node,
                            violations
                        );
                    }
                }
            }
            
            // Continue checking conditions on subsequent lines if they're part of the ON clause
            if current_join_line.is_some() && 
               !lower_line.contains(" from ") && 
               !lower_line.contains(" where ") &&
               !lower_line.contains(" group ") &&
               !lower_line.contains(" order ") &&
               !lower_line.contains(" join ") {
                // This might be a continuation of the ON condition
                let trimmed = line.trim();
                if trimmed.contains('=') {
                    self.check_condition_order(
                        trimmed,
                        &table_order,
                        line_idx,
                        node,
                        violations
                    );
                }
            } else if lower_line.contains(" where ") || 
                      lower_line.contains(" group ") || 
                      lower_line.contains(" order ") {
                current_join_line = None;
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_join_conditions(child, source, violations);
            }
        }
    }
    
    fn extract_table_after_keyword(&self, line: &str, keyword: &str) -> Option<String> {
        let lower_line = line.to_lowercase();
        if let Some(pos) = lower_line.find(&format!(" {} ", keyword)) {
            let after_keyword = &line[pos + keyword.len() + 2..];
            let tokens: Vec<&str> = after_keyword.split_whitespace().collect();
            
            if !tokens.is_empty() {
                let table_part = tokens[0];
                // Extract just the table name/alias (before any keywords)
                if let Some(space_pos) = table_part.find(' ') {
                    return Some(table_part[..space_pos].to_string());
                } else {
                    // Check if next token is AS or an alias
                    if tokens.len() > 1 && tokens[1].to_lowercase() == "as" && tokens.len() > 2 {
                        return Some(tokens[2].to_string());
                    } else if tokens.len() > 1 && !self.is_sql_keyword(&tokens[1]) {
                        return Some(tokens[1].to_string());
                    }
                    return Some(table_part.to_string());
                }
            }
        }
        None
    }
    
    fn check_condition_order(
        &self,
        condition: &str,
        table_order: &[String],
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for equality conditions like "a.id = b.id"
        if let Some(eq_pos) = condition.find('=') {
            let left = condition[..eq_pos].trim();
            let right = condition[eq_pos + 1..].trim();
            
            // Extract table references
            let left_table = self.extract_table_from_column_ref(left);
            let right_table = self.extract_table_from_column_ref(right);
            
            if let (Some(lt), Some(rt)) = (left_table, right_table) {
                // Find positions in table order
                let left_pos = table_order.iter().position(|t| t == &lt);
                let right_pos = table_order.iter().position(|t| t == &rt);
                
                if let (Some(lp), Some(rp)) = (left_pos, right_pos) {
                    if lp > rp {
                        // Tables are in wrong order
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "Join condition should reference '{}' before '{}' to match table order",
                                rt, lt
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    fn extract_table_from_column_ref(&self, column_ref: &str) -> Option<String> {
        // Handle "table.column" pattern
        if let Some(dot_pos) = column_ref.find('.') {
            let table = column_ref[..dot_pos].trim();
            Some(table.to_string())
        } else {
            None
        }
    }
    
    fn is_sql_keyword(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        matches!(lower.as_str(),
            "where" | "join" | "inner" | "left" | "right" | "full" | "outer" |
            "on" | "and" | "or" | "group" | "by" | "having" | "order" | "limit" |
            "as" | "asc" | "desc" | "union" | "all" | "distinct"
        )
    }
}