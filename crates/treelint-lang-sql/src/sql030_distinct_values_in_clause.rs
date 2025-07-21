use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};
use std::collections::HashSet;

pub struct DistinctValuesInClause;

impl Rule for DistinctValuesInClause {
    fn id(&self) -> &'static str {
        "SQL030"
    }

    fn name(&self) -> &'static str {
        "distinct-values-in-clause"
    }

    fn description(&self) -> &'static str {
        "IN clauses should not contain duplicate values"
    }

    fn explanation(&self) -> &'static str {
        "Duplicate values in IN clauses are redundant and can impact query performance. 
        Each value should appear only once. For example, use IN (1, 2, 3) instead of IN (1, 2, 1, 3)."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_in_clauses(tree.root_node(), source, &mut violations);

        violations
    }
}

impl DistinctValuesInClause {
    fn check_in_clauses(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        let mut current_in_clause = String::new();
        let mut in_clause = false;
        let mut clause_start_line = 0;
        let mut paren_depth = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            
            // Look for IN clause start
            if !in_clause && (lower_line.contains(" in (") || lower_line.contains(" in(")) {
                in_clause = true;
                clause_start_line = line_idx;
                current_in_clause.clear();
                
                // Extract the part after IN
                if let Some(pos) = lower_line.find(" in") {
                    let after_in = &line[pos + 3..];
                    current_in_clause.push_str(after_in);
                    paren_depth = after_in.chars().filter(|&c| c == '(').count() as i32
                                - after_in.chars().filter(|&c| c == ')').count() as i32;
                }
                continue;
            }
            
            // Continue collecting IN clause content
            if in_clause {
                current_in_clause.push(' ');
                current_in_clause.push_str(line);
                paren_depth += line.chars().filter(|&c| c == '(').count() as i32
                            - line.chars().filter(|&c| c == ')').count() as i32;
                
                // Check if IN clause is complete
                if paren_depth <= 0 {
                    self.check_in_clause_values(
                        &current_in_clause,
                        clause_start_line,
                        node,
                        violations
                    );
                    in_clause = false;
                    current_in_clause.clear();
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_in_clauses(child, source, violations);
            }
        }
    }
    
    fn check_in_clause_values(
        &self,
        clause: &str,
        start_line: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Extract values between parentheses
        if let Some(start) = clause.find('(') {
            if let Some(end) = clause.rfind(')') {
                let values_str = &clause[start + 1..end];
                let values = self.parse_in_clause_values(values_str);
                
                // Check for duplicates
                let mut seen = HashSet::new();
                let mut duplicates = Vec::new();
                
                for value in &values {
                    let normalized = value.trim().to_lowercase();
                    if !seen.insert(normalized.clone()) {
                        duplicates.push(value.trim());
                    }
                }
                
                if !duplicates.is_empty() {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + start_line + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "IN clause contains duplicate values: {}. Each value should appear only once",
                            duplicates.join(", ")
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
    
    fn parse_in_clause_values(&self, values_str: &str) -> Vec<String> {
        let mut values = Vec::new();
        let mut current_value = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';
        
        for ch in values_str.chars() {
            match ch {
                '\'' | '"' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                    current_value.push(ch);
                }
                '\'' | '"' if in_quotes && ch == quote_char => {
                    in_quotes = false;
                    current_value.push(ch);
                }
                ',' if !in_quotes => {
                    if !current_value.trim().is_empty() {
                        values.push(current_value.trim().to_string());
                    }
                    current_value.clear();
                }
                _ => {
                    current_value.push(ch);
                }
            }
        }
        
        // Don't forget the last value
        if !current_value.trim().is_empty() {
            values.push(current_value.trim().to_string());
        }
        
        values
    }
}