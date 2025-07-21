use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct CteVsSubquery;

impl Rule for CteVsSubquery {
    fn id(&self) -> &'static str {
        "SQL043"
    }

    fn name(&self) -> &'static str {
        "cte-vs-subquery"
    }

    fn description(&self) -> &'static str {
        "Prefer CTEs over complex subqueries for readability"
    }

    fn explanation(&self) -> &'static str {
        "Use Common Table Expressions (CTEs) instead of complex subqueries when the 
        subquery is used multiple times or is complex. CTEs improve readability, 
        maintainability, and can often be reused within the same query."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_subquery_complexity(tree.root_node(), source, &mut violations);

        violations
    }
}

impl CteVsSubquery {
    fn check_subquery_complexity(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        
        // Find subqueries and analyze their complexity
        let mut subqueries = Vec::new();
        self.find_subqueries(node_text, &mut subqueries);
        
        for (start_line, subquery) in subqueries {
            let complexity_score = self.calculate_subquery_complexity(&subquery);
            
            if complexity_score >= 3 {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + start_line + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Complex subquery (complexity: {}) should be converted to a CTE for better readability",
                        complexity_score
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        // Check for repeated subqueries
        self.check_repeated_subqueries(node_text, node, violations);

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_subquery_complexity(child, source, violations);
            }
        }
    }
    
    fn find_subqueries(&self, text: &str, subqueries: &mut Vec<(usize, String)>) {
        let lines: Vec<&str> = text.split('\n').collect();
        let mut paren_depth = 0;
        let mut in_subquery = false;
        let mut subquery_start_line = 0;
        let mut subquery_content = String::new();
        
        for (line_idx, line) in lines.iter().enumerate() {
            let mut chars = line.chars().peekable();
            
            while let Some(ch) = chars.next() {
                match ch {
                    '(' => {
                        paren_depth += 1;
                        
                        // Check if this starts a subquery
                        let remaining: String = chars.clone().collect();
                        if remaining.trim_start().to_lowercase().starts_with("select ") {
                            if !in_subquery {
                                in_subquery = true;
                                subquery_start_line = line_idx;
                                subquery_content.clear();
                            }
                        }
                        
                        if in_subquery {
                            subquery_content.push(ch);
                        }
                    }
                    ')' => {
                        if in_subquery {
                            subquery_content.push(ch);
                        }
                        
                        paren_depth -= 1;
                        
                        if in_subquery && paren_depth == 0 {
                            subqueries.push((subquery_start_line, subquery_content.clone()));
                            in_subquery = false;
                            subquery_content.clear();
                        }
                    }
                    _ => {
                        if in_subquery {
                            subquery_content.push(ch);
                        }
                    }
                }
            }
            
            if in_subquery && line_idx < lines.len() - 1 {
                subquery_content.push('\n');
            }
        }
    }
    
    fn calculate_subquery_complexity(&self, subquery: &str) -> u32 {
        let lower_subquery = subquery.to_lowercase();
        let mut complexity = 0;
        
        // Base complexity for being a subquery
        complexity += 1;
        
        // Count SQL keywords that add complexity
        let complex_keywords = [
            "join", "inner join", "left join", "right join", "full join",
            "where", "group by", "having", "order by", "union", "except", "intersect"
        ];
        
        for keyword in complex_keywords.iter() {
            complexity += lower_subquery.matches(keyword).count() as u32;
        }
        
        // Count nested subqueries
        complexity += self.count_nested_selects(&lower_subquery);
        
        // Count aggregation functions
        let agg_functions = ["count(", "sum(", "avg(", "max(", "min(", "group_concat("];
        for func in agg_functions.iter() {
            complexity += lower_subquery.matches(func).count() as u32;
        }
        
        // Count CASE statements
        complexity += lower_subquery.matches(" case ").count() as u32;
        
        // Line count penalty for very long subqueries
        let line_count = subquery.lines().count();
        if line_count > 5 {
            complexity += (line_count - 5) as u32;
        }
        
        complexity
    }
    
    fn count_nested_selects(&self, text: &str) -> u32 {
        // Count SELECT keywords that indicate nested queries
        let select_count = text.matches("select ").count();
        if select_count > 1 {
            (select_count - 1) as u32 // -1 because the main SELECT doesn't count as nesting
        } else {
            0
        }
    }
    
    fn check_repeated_subqueries(
        &self,
        text: &str,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut subqueries = Vec::new();
        self.find_subqueries(text, &mut subqueries);
        
        // Group similar subqueries
        for i in 0..subqueries.len() {
            for j in i + 1..subqueries.len() {
                let (_line1, query1) = &subqueries[i];
                let (line2, query2) = &subqueries[j];
                
                if self.are_similar_subqueries(query1, query2) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line2 + 1,
                        column: start_pos.column + 1,
                        message: "Similar subquery found elsewhere in query. Consider using a CTE to avoid repetition".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
    
    fn are_similar_subqueries(&self, query1: &str, query2: &str) -> bool {
        // Simple similarity check - normalize and compare
        let normalized1 = self.normalize_query(query1);
        let normalized2 = self.normalize_query(query2);
        
        // If they're identical after normalization, they're similar
        normalized1 == normalized2
    }
    
    fn normalize_query(&self, query: &str) -> String {
        query
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}