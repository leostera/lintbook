use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct LineBreaksInClauses;

impl Rule for LineBreaksInClauses {
    fn id(&self) -> &'static str {
        "SQL042"
    }

    fn name(&self) -> &'static str {
        "line-breaks-in-clauses"
    }

    fn description(&self) -> &'static str {
        "Use consistent line breaks between major SQL clauses"
    }

    fn explanation(&self) -> &'static str {
        "Major SQL clauses (SELECT, FROM, WHERE, GROUP BY, ORDER BY, etc.) should be 
        on separate lines for better readability. This makes the query structure clearer 
        and easier to understand at a glance."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_clause_line_breaks(tree.root_node(), source, &mut violations);

        violations
    }
}

impl LineBreaksInClauses {
    fn check_clause_line_breaks(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            
            // Check for multiple major clauses on the same line
            let major_clauses = [
                ("select ", "SELECT"),
                (" from ", "FROM"),
                (" where ", "WHERE"),
                (" group by ", "GROUP BY"),
                (" having ", "HAVING"),
                (" order by ", "ORDER BY"),
                (" limit ", "LIMIT"),
                (" union ", "UNION"),
                (" except ", "EXCEPT"),
                (" intersect ", "INTERSECT"),
            ];
            
            let mut found_clauses = Vec::new();
            
            for (pattern, clause_name) in major_clauses.iter() {
                let mut search_start = 0;
                while let Some(pos) = lower_line[search_start..].find(pattern) {
                    let absolute_pos = search_start + pos;
                    found_clauses.push((absolute_pos, clause_name));
                    search_start = absolute_pos + pattern.len();
                }
            }
            
            // Sort by position
            found_clauses.sort_by_key(|(pos, _)| *pos);
            
            // If we found more than one major clause on the same line, report violation
            if found_clauses.len() > 1 {
                let start_pos = node.start_position();
                let clause_names: Vec<&str> = found_clauses.iter().map(|(_, name)| **name).collect();
                
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Multiple major clauses on same line: {}. Each major clause should be on its own line",
                        clause_names.join(", ")
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
            
            // Check for clauses that should be on new lines but aren't
            // Look for patterns like "SELECT ... FROM" where FROM should be on next line
            self.check_specific_clause_patterns(line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_clause_line_breaks(child, source, violations);
            }
        }
    }
    
    fn check_specific_clause_patterns(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();
        
        // Pattern: SELECT columns FROM table (FROM should be on new line for complex queries)
        if lower_line.contains("select ") && lower_line.contains(" from ") {
            if let Some(select_pos) = lower_line.find("select ") {
                if let Some(from_pos) = lower_line.find(" from ") {
                    // If there's substantial content between SELECT and FROM, suggest line break
                    let content_between = &line[select_pos + 7..from_pos];
                    if content_between.len() > 30 || content_between.contains(',') {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + from_pos + 1,
                            message: "FROM clause should be on a new line for better readability".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
        
        // Pattern: WHERE condition AND/OR (complex conditions should break)
        if lower_line.contains(" where ") {
            let and_or_count = lower_line.matches(" and ").count() + lower_line.matches(" or ").count();
            if and_or_count >= 2 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Complex WHERE conditions should be split across multiple lines".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
}