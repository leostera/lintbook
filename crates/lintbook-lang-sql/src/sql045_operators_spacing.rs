use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct OperatorsSpacing;

impl Rule for OperatorsSpacing {
    fn id(&self) -> &'static str {
        "SQL045"
    }

    fn name(&self) -> &'static str {
        "operators-spacing"
    }

    fn description(&self) -> &'static str {
        "Use consistent spacing around operators"
    }

    fn explanation(&self) -> &'static str {
        "Operators should have consistent spacing for better readability. Use spaces around
        binary operators (=, !=, <>, <, >, <=, >=, +, -, *, /, AND, OR). Avoid extra spaces
        that make expressions harder to read."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_operator_spacing(tree.root_node(), source, &mut violations);

        violations
    }
}

impl OperatorsSpacing {
    fn check_operator_spacing(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }

            self.check_line_operators(line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_operator_spacing(child, source, violations);
            }
        }
    }

    fn check_line_operators(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let operators = [
            ("=", "equals"),
            ("!=", "not equals"),
            ("<>", "not equals"),
            ("<=", "less than or equal"),
            (">=", "greater than or equal"),
            ("<", "less than"),
            (">", "greater than"),
            ("+", "plus"),
            ("-", "minus"),
            ("*", "multiply"),
            ("/", "divide"),
        ];

        for (op, op_name) in operators.iter() {
            self.check_operator_in_line(line, op, op_name, line_idx, node, violations);
        }

        // Check AND/OR operators
        self.check_logical_operators(line, line_idx, node, violations);
    }

    fn check_operator_in_line(
        &self,
        line: &str,
        operator: &str,
        op_name: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut search_start = 0;

        while let Some(pos) = line[search_start..].find(operator) {
            let absolute_pos = search_start + pos;

            // Skip if it's inside a string literal
            if self.is_inside_string(line, absolute_pos) {
                search_start = absolute_pos + operator.len();
                continue;
            }

            // Skip special cases
            if self.should_skip_operator(line, absolute_pos, operator) {
                search_start = absolute_pos + operator.len();
                continue;
            }

            // Check spacing around the operator
            let before_char = if absolute_pos > 0 {
                line.chars().nth(absolute_pos - 1)
            } else {
                None
            };

            let after_pos = absolute_pos + operator.len();
            let after_char = line.chars().nth(after_pos);

            // Check for missing spaces
            let needs_space_before = before_char.map_or(false, |c| !c.is_whitespace() && c != '(');
            let needs_space_after = after_char.map_or(false, |c| !c.is_whitespace() && c != ')');

            if needs_space_before || needs_space_after {
                let start_pos = node.start_position();
                let message = if needs_space_before && needs_space_after {
                    format!("Missing spaces around {} operator", op_name)
                } else if needs_space_before {
                    format!("Missing space before {} operator", op_name)
                } else {
                    format!("Missing space after {} operator", op_name)
                };

                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + absolute_pos + 1,
                    message,
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            search_start = absolute_pos + operator.len();
        }
    }

    fn check_logical_operators(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();
        let logical_ops = [(" and ", "AND"), (" or ", "OR")];

        for (pattern, op_name) in logical_ops.iter() {
            let mut search_start = 0;

            while let Some(pos) = lower_line[search_start..].find(pattern) {
                let absolute_pos = search_start + pos;

                // Check if there are extra spaces
                let before_spaces = self.count_trailing_spaces(&line[..absolute_pos]);
                let after_start = absolute_pos + pattern.len();
                let after_spaces = self.count_leading_spaces(&line[after_start..]);

                if before_spaces > 1 || after_spaces > 1 {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + absolute_pos + 1,
                        message: format!(
                            "Excessive spacing around {} operator. Use single spaces",
                            op_name
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }

                search_start = absolute_pos + pattern.len();
            }
        }
    }

    fn is_inside_string(&self, line: &str, pos: usize) -> bool {
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        for (i, ch) in line.chars().enumerate() {
            if i >= pos {
                break;
            }

            match ch {
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                _ => {}
            }
        }

        in_single_quote || in_double_quote
    }

    fn should_skip_operator(&self, line: &str, pos: usize, operator: &str) -> bool {
        // Skip minus in negative numbers
        if operator == "-" {
            if pos == 0
                || line
                    .chars()
                    .nth(pos - 1)
                    .map_or(false, |c| c.is_whitespace() || c == '(')
            {
                // Check if it's followed by a digit
                if line
                    .chars()
                    .nth(pos + 1)
                    .map_or(false, |c| c.is_ascii_digit())
                {
                    return true;
                }
            }
        }

        // Skip operators in function calls or complex expressions where spacing rules differ
        let _context_before = if pos >= 3 {
            &line[pos.saturating_sub(3)..pos]
        } else {
            &line[..pos]
        };
        let context_after = &line[pos + operator.len()..]
            .chars()
            .take(3)
            .collect::<String>();

        // Skip if it looks like part of a larger operator or special context
        if operator == ">" && context_after.starts_with('=') {
            return true; // This is ">=" handled elsewhere
        }

        if operator == "<" && context_after.starts_with('=') {
            return true; // This is "<=" handled elsewhere
        }

        false
    }

    fn count_trailing_spaces(&self, text: &str) -> usize {
        text.chars().rev().take_while(|&c| c == ' ').count()
    }

    fn count_leading_spaces(&self, text: &str) -> usize {
        text.chars().take_while(|&c| c == ' ').count()
    }
}
