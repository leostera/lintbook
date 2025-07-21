use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct ComplexExpressionsNeedAliases;

impl Rule for ComplexExpressionsNeedAliases {
    fn id(&self) -> &'static str {
        "SQL014"
    }

    fn name(&self) -> &'static str {
        "complex-expressions-need-aliases"
    }

    fn description(&self) -> &'static str {
        "Complex expressions in SELECT should have aliases"
    }

    fn explanation(&self) -> &'static str {
        "Complex expressions (calculations, function calls, CASE statements) in SELECT clauses 
        should have meaningful aliases to improve readability and make column references clearer."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_select_expressions(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ComplexExpressionsNeedAliases {
    fn check_select_expressions(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            if lower_line.contains("select ") && !lower_line.contains("*") {
                self.check_select_clause(line, line_idx, violations, node);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_select_expressions(child, source, violations);
            }
        }
    }

    fn check_select_clause(
        &self,
        line: &str,
        line_idx: usize,
        violations: &mut Vec<LintViolation>,
        node: tree_sitter::Node,
    ) {
        if let Some(select_pos) = line.to_lowercase().find("select ") {
            let after_select = &line[select_pos + 7..];
            let from_pos = after_select.to_lowercase().find(" from ");
            let columns_part = if let Some(pos) = from_pos {
                &after_select[..pos]
            } else {
                after_select
            };

            let columns: Vec<&str> = columns_part.split(',').collect();

            for column in columns {
                let trimmed = column.trim();

                // Skip if already has alias
                if trimmed.to_lowercase().contains(" as ") {
                    continue;
                }

                // Check for complex expressions that need aliases
                if self.is_complex_expression(trimmed) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Complex expression '{}' should have an alias for clarity",
                            if trimmed.len() > 50 {
                                format!("{}...", &trimmed[..47])
                            } else {
                                trimmed.to_string()
                            }
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn is_complex_expression(&self, expression: &str) -> bool {
        let trimmed = expression.trim();

        // Function calls
        if trimmed.contains('(') && trimmed.contains(')') {
            // Skip simple single-argument functions like COUNT(*)
            if !(trimmed.starts_with("count(*)") || trimmed.starts_with("COUNT(*)")) {
                return true;
            }
        }

        // Mathematical operations
        if trimmed.contains('+')
            || trimmed.contains('-')
            || trimmed.contains('*')
            || trimmed.contains('/')
        {
            // Skip simple column references with table prefixes (table.column)
            if trimmed.matches('.').count() <= 1 && !trimmed.contains(' ') {
                return false;
            }
            return true;
        }

        // CASE statements
        if trimmed.to_lowercase().contains("case") {
            return true;
        }

        // String concatenation
        if trimmed.contains("||") || trimmed.to_lowercase().contains("concat") {
            return true;
        }

        // Subqueries
        if trimmed.to_lowercase().contains("select") {
            return true;
        }

        // COALESCE and similar functions
        if trimmed.to_lowercase().contains("coalesce")
            || trimmed.to_lowercase().contains("ifnull")
            || trimmed.to_lowercase().contains("nvl")
        {
            return true;
        }

        false
    }
}
