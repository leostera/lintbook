use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct QualificationConsistency;

impl Rule for QualificationConsistency {
    fn id(&self) -> &'static str {
        "SQL039"
    }

    fn name(&self) -> &'static str {
        "qualification-consistency"
    }

    fn description(&self) -> &'static str {
        "Use consistent column qualification within queries"
    }

    fn explanation(&self) -> &'static str {
        "Be consistent with column qualification. If you qualify some columns with table
        names/aliases, qualify all columns. This improves clarity, especially in queries
        with multiple tables."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_qualification_consistency(tree.root_node(), source, &mut violations);

        violations
    }
}

impl QualificationConsistency {
    fn check_qualification_consistency(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        let mut in_select = false;
        let mut select_items = Vec::new();
        let mut has_joins = false;
        let mut select_start_line = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            let trimmed = line.trim();

            // Check for SELECT clause
            if lower_line.contains("select ") && !in_select {
                in_select = true;
                select_start_line = line_idx;
                select_items.clear();

                // Extract items from same line
                if let Some(pos) = lower_line.find("select ") {
                    let after_select = &line[pos + 7..];
                    self.extract_select_items(after_select, &mut select_items);
                }
            }

            // Continue collecting SELECT items
            if in_select && !lower_line.contains(" from ") {
                if line_idx > select_start_line {
                    self.extract_select_items(trimmed, &mut select_items);
                }
            }

            // Check for JOIN (indicates multiple tables)
            if lower_line.contains(" join ") {
                has_joins = true;
            }

            // End of SELECT clause
            if in_select && lower_line.contains(" from ") {
                in_select = false;

                // Only check if we have joins (multiple tables)
                if has_joins && select_items.len() > 1 {
                    self.analyze_qualification_consistency(
                        &select_items,
                        select_start_line,
                        node,
                        violations,
                    );
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_qualification_consistency(child, source, violations);
            }
        }
    }

    fn extract_select_items(&self, text: &str, items: &mut Vec<(String, bool)>) {
        // Split by commas but handle nested functions
        let mut current_item = String::new();
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';

        for ch in text.chars() {
            match ch {
                '\'' | '"' if !in_string => {
                    in_string = true;
                    string_char = ch;
                    current_item.push(ch);
                }
                '\'' | '"' if in_string && ch == string_char => {
                    in_string = false;
                    current_item.push(ch);
                }
                '(' if !in_string => {
                    paren_depth += 1;
                    current_item.push(ch);
                }
                ')' if !in_string => {
                    paren_depth -= 1;
                    current_item.push(ch);
                }
                ',' if !in_string && paren_depth == 0 => {
                    if !current_item.trim().is_empty() {
                        let has_qualifier = self.has_table_qualifier(&current_item);
                        items.push((current_item.trim().to_string(), has_qualifier));
                    }
                    current_item.clear();
                }
                _ => {
                    current_item.push(ch);
                }
            }
        }

        // Don't forget the last item
        if !current_item.trim().is_empty() {
            let has_qualifier = self.has_table_qualifier(&current_item);
            items.push((current_item.trim().to_string(), has_qualifier));
        }
    }

    fn has_table_qualifier(&self, item: &str) -> bool {
        let trimmed = item.trim();

        // Skip if it's SELECT * or aggregate without column
        if trimmed == "*" || trimmed.starts_with("COUNT(*)") || trimmed.starts_with("count(*)") {
            return true; // Don't count these as unqualified
        }

        // Check for table.column pattern
        if trimmed.contains('.') {
            // Make sure it's not a decimal number
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() >= 2 && !parts[0].chars().all(|c| c.is_numeric()) {
                return true;
            }
        }

        false
    }

    fn analyze_qualification_consistency(
        &self,
        items: &[(String, bool)],
        start_line: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Count qualified vs unqualified columns
        let qualified_count = items.iter().filter(|(_, q)| *q).count();
        let unqualified_count = items.iter().filter(|(_, q)| !*q).count();

        // If we have both qualified and unqualified columns, report inconsistency
        if qualified_count > 0 && unqualified_count > 0 {
            // Report on the unqualified columns
            for (item, qualified) in items {
                if !qualified && !self.is_literal_or_function_only(item) {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + start_line + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Column '{}' is not qualified while other columns are. Use consistent qualification",
                            item
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn is_literal_or_function_only(&self, item: &str) -> bool {
        let trimmed = item.trim();

        // Check if it's a literal value
        if trimmed.starts_with('\'') || trimmed.starts_with('"') || trimmed.parse::<f64>().is_ok() {
            return true;
        }

        // Check if it's a function without column reference
        let function_keywords = [
            "CURRENT_DATE",
            "CURRENT_TIME",
            "CURRENT_TIMESTAMP",
            "NOW()",
            "GETDATE()",
            "SYSDATE",
            "NULL",
        ];
        for keyword in function_keywords.iter() {
            if trimmed.to_uppercase() == *keyword {
                return true;
            }
        }

        false
    }
}
