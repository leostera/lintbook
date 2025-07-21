use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct AvoidAliasesInCtes;

impl Rule for AvoidAliasesInCtes {
    fn id(&self) -> &'static str {
        "SQL027"
    }

    fn name(&self) -> &'static str {
        "avoid-aliases-in-ctes"
    }

    fn description(&self) -> &'static str {
        "Avoid unnecessary table aliases in CTE definitions"
    }

    fn explanation(&self) -> &'static str {
        "Common Table Expressions (CTEs) already have names, so aliasing them adds 
        unnecessary complexity. Use the CTE name directly instead of creating an alias."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_cte_aliases(tree.root_node(), source, &mut violations);

        violations
    }
}

impl AvoidAliasesInCtes {
    fn check_cte_aliases(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        let mut cte_names = Vec::new();
        let mut in_with_clause = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            let trimmed = line.trim();
            
            // Start of WITH clause
            if lower_line.trim_start().starts_with("with ") {
                in_with_clause = true;
                // Extract CTE name from same line if present
                if let Some(cte_name) = self.extract_cte_name_from_line(trimmed) {
                    cte_names.push(cte_name);
                }
                continue;
            }
            
            // Inside WITH clause, look for CTE definitions
            if in_with_clause && trimmed.contains(" AS (") {
                if let Some(cte_name) = self.extract_cte_name_from_line(trimmed) {
                    cte_names.push(cte_name);
                }
            }
            
            // End of WITH clause (main query starts)
            if in_with_clause && (lower_line.contains("select ") && !lower_line.contains(" as (")) {
                in_with_clause = false;
            }
            
            // Check for aliased CTEs in FROM/JOIN clauses
            if !in_with_clause && (lower_line.contains(" from ") || lower_line.contains(" join ")) {
                self.check_line_for_cte_aliases(line, &cte_names, line_idx, node, violations);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_cte_aliases(child, source, violations);
            }
        }
    }
    
    fn extract_cte_name_from_line(&self, line: &str) -> Option<String> {
        // Pattern: cte_name AS (
        let trimmed = line.trim();
        if let Some(as_pos) = trimmed.find(" AS (") {
            let name_part = &trimmed[..as_pos];
            let tokens: Vec<&str> = name_part.split_whitespace().collect();
            if !tokens.is_empty() {
                let last_token = tokens[tokens.len() - 1];
                // Skip if it's WITH keyword
                if last_token.to_lowercase() != "with" {
                    return Some(last_token.to_string());
                }
            }
        }
        None
    }
    
    fn check_line_for_cte_aliases(
        &self,
        line: &str,
        cte_names: &[String],
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        for i in 0..tokens.len() {
            let token = tokens[i];
            
            // Check if this token is a known CTE name
            if cte_names.iter().any(|cte| cte.eq_ignore_ascii_case(token)) {
                // Check if it has an alias
                if i + 1 < tokens.len() {
                    let next_token = tokens[i + 1];
                    let next_lower = next_token.to_lowercase();
                    
                    // Skip if next token is a SQL keyword (not an alias)
                    if matches!(next_lower.as_str(), 
                        "where" | "join" | "inner" | "left" | "right" | 
                        "full" | "outer" | "on" | "and" | "or" | "," | ")" | "group" | "order"
                    ) {
                        continue;
                    }
                    
                    // Check for explicit AS
                    if next_lower == "as" && i + 2 < tokens.len() {
                        let alias = tokens[i + 2];
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "CTE '{}' should not be aliased as '{}'. Use the CTE name directly",
                                token, alias
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    } else if !matches!(next_lower.as_str(), "as") {
                        // Implicit alias
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + 1,
                            message: format!(
                                "CTE '{}' should not be aliased as '{}'. Use the CTE name directly",
                                token, next_token
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