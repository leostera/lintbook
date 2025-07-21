use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct TableAliasLength;

impl Rule for TableAliasLength {
    fn id(&self) -> &'static str {
        "SQL026"
    }

    fn name(&self) -> &'static str {
        "table-alias-length"
    }

    fn description(&self) -> &'static str {
        "Table aliases should be meaningful and not too short"
    }

    fn explanation(&self) -> &'static str {
        "Single-letter table aliases (except for simple queries) reduce code readability. 
        Use meaningful abbreviations that indicate the table name, e.g., 'usr' for 'users', 
        'ord' for 'orders'."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_alias_length(tree.root_node(), source, &mut violations);

        violations
    }
}

impl TableAliasLength {
    fn check_alias_length(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        // Count total number of table references to determine complexity
        let table_count = self.count_table_references(&lines);
        
        // Only enforce for queries with multiple tables
        if table_count < 2 {
            return;
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            
            // Check for table aliases in FROM clause
            if lower_line.contains(" from ") || lower_line.contains(" join ") {
                self.check_line_for_aliases(line, line_idx, node, violations);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_alias_length(child, source, violations);
            }
        }
    }
    
    fn count_table_references(&self, lines: &[&str]) -> usize {
        let mut count = 0;
        for line in lines {
            let lower_line = line.to_lowercase();
            if lower_line.contains(" from ") {
                count += 1;
            }
            count += lower_line.matches(" join ").count();
        }
        count
    }
    
    fn check_line_for_aliases(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Pattern: table_name AS alias or table_name alias
        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        for i in 0..tokens.len() {
            let token = tokens[i];
            let lower_token = token.to_lowercase();
            
            // Skip SQL keywords
            if matches!(lower_token.as_str(), 
                "select" | "from" | "where" | "join" | "inner" | "left" | 
                "right" | "full" | "outer" | "on" | "as" | "and" | "or"
            ) {
                continue;
            }
            
            // Check if next token could be an alias
            if i + 1 < tokens.len() {
                let next_token = tokens[i + 1];
                let next_lower = next_token.to_lowercase();
                
                // Skip if next token is a keyword (not an alias)
                if matches!(next_lower.as_str(), 
                    "where" | "join" | "inner" | "left" | "right" | 
                    "full" | "outer" | "on" | "and" | "or" | ","
                ) {
                    continue;
                }
                
                // Check for explicit AS
                if i + 2 < tokens.len() && tokens[i + 1].to_lowercase() == "as" {
                    let alias = tokens[i + 2];
                    if alias.len() == 1 && alias.chars().all(|c| c.is_alphabetic()) {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "Single-letter alias '{}' is too short for complex queries. Use a meaningful abbreviation",
                                alias
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                } else if next_token.len() == 1 && next_token.chars().all(|c| c.is_alphabetic()) {
                    // Implicit alias
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Single-letter alias '{}' is too short for complex queries. Use a meaningful abbreviation",
                            next_token
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}