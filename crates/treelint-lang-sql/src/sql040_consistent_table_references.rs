use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};
use std::collections::HashMap;

pub struct ConsistentTableReferences;

impl Rule for ConsistentTableReferences {
    fn id(&self) -> &'static str {
        "SQL040"
    }

    fn name(&self) -> &'static str {
        "consistent-table-references"
    }

    fn description(&self) -> &'static str {
        "Reference tables consistently throughout the query"
    }

    fn explanation(&self) -> &'static str {
        "Use the same identifier (table name or alias) consistently throughout a query. 
        Don't mix table names and aliases when referencing the same table. If you define 
        an alias, use it everywhere in that query."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_table_reference_consistency(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ConsistentTableReferences {
    fn check_table_reference_consistency(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        // Map of table names to their aliases
        let mut table_aliases: HashMap<String, String> = HashMap::new();
        let mut query_sections = Vec::new();
        let mut current_section = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            
            // New query starts (rough detection)
            if lower_line.trim().starts_with("select ") || 
               lower_line.trim().starts_with("insert ") ||
               lower_line.trim().starts_with("update ") ||
               lower_line.trim().starts_with("delete ") {
                if !current_section.is_empty() {
                    query_sections.push(current_section.clone());
                    current_section.clear();
                }
            }
            
            current_section.push((line_idx, *line));
            
            // Extract table aliases from FROM and JOIN clauses
            if lower_line.contains(" from ") || lower_line.contains(" join ") {
                self.extract_table_aliases(line, &mut table_aliases);
            }
        }
        
        // Don't forget the last section
        if !current_section.is_empty() {
            query_sections.push(current_section);
        }
        
        // Check each query section
        for section in query_sections {
            if !table_aliases.is_empty() {
                self.check_section_references(&section, &table_aliases, node, violations);
            }
            table_aliases.clear();
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_table_reference_consistency(child, source, violations);
            }
        }
    }
    
    fn extract_table_aliases(&self, line: &str, aliases: &mut HashMap<String, String>) {
        let lower_line = line.to_lowercase();
        
        // Extract from FROM clause
        if let Some(from_pos) = lower_line.find(" from ") {
            let after_from = &line[from_pos + 6..];
            self.extract_alias_from_clause(after_from, aliases);
        }
        
        // Extract from JOIN clauses
        for join_type in &[" join ", " inner join ", " left join ", " right join ", " full join "] {
            if let Some(join_pos) = lower_line.find(join_type) {
                let after_join = &line[join_pos + join_type.len()..];
                self.extract_alias_from_clause(after_join, aliases);
            }
        }
    }
    
    fn extract_alias_from_clause(&self, clause: &str, aliases: &mut HashMap<String, String>) {
        let tokens: Vec<&str> = clause.split_whitespace().collect();
        
        if tokens.len() >= 2 {
            let table_name = tokens[0].trim_end_matches(',');
            
            // Check for AS alias
            if tokens.len() >= 3 && tokens[1].to_lowercase() == "as" {
                let alias = tokens[2].trim_end_matches(',');
                aliases.insert(table_name.to_lowercase(), alias.to_string());
            }
            // Check for implicit alias
            else if tokens.len() >= 2 && !self.is_sql_keyword(&tokens[1]) {
                let alias = tokens[1].trim_end_matches(',');
                aliases.insert(table_name.to_lowercase(), alias.to_string());
            }
        }
    }
    
    fn check_section_references(
        &self,
        section: &[(usize, &str)],
        aliases: &HashMap<String, String>,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        for (line_idx, line) in section {
            // Look for table references (table.column pattern)
            let tokens: Vec<&str> = line.split(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')')
                .filter(|s| !s.is_empty())
                .collect();
            
            for token in tokens {
                if token.contains('.') {
                    let parts: Vec<&str> = token.split('.').collect();
                    if parts.len() >= 2 {
                        let table_ref = parts[0].to_lowercase();
                        
                        // Check if this table has an alias
                        if let Some(alias) = aliases.get(&table_ref) {
                            // Table is referenced by name but has an alias
                            let start_pos = node.start_position();
                            violations.push(LintViolation {
                                line: start_pos.row + line_idx + 1,
                                column: start_pos.column + 1,
                                message: format!(
                                    "Table '{}' is referenced by name but has alias '{}'. Use the alias consistently",
                                    table_ref, alias
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
    
    fn is_sql_keyword(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        matches!(lower.as_str(),
            "where" | "join" | "inner" | "left" | "right" | "full" | "outer" |
            "on" | "and" | "or" | "group" | "by" | "having" | "order" | "limit" |
            "as" | "asc" | "desc" | "union" | "all" | "distinct" | "from" | "select"
        )
    }
}