use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct LineLength;

impl Rule for LineLength {
    fn id(&self) -> &'static str {
        "SQL036"
    }

    fn name(&self) -> &'static str {
        "line-length"
    }

    fn description(&self) -> &'static str {
        "Lines should not exceed maximum length"
    }

    fn explanation(&self) -> &'static str {
        "Long lines are difficult to read and review. Keep lines under 120 characters.
        For better readability, consider breaking long lines at logical points like
        commas, operators, or keywords."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let max_length = 120; // Configurable in the future

        self.check_line_length(tree.root_node(), source, max_length, &mut violations);

        violations
    }
}

impl LineLength {
    fn check_line_length(
        &self,
        node: tree_sitter::Node,
        source: &str,
        max_length: usize,
        violations: &mut Vec<LintViolation>,
    ) {
        let lines: Vec<&str> = source.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Count visual length (handling tabs as 4 spaces)
            let visual_length = self.calculate_visual_length(line);

            if visual_length > max_length {
                let start_pos = node.start_position();

                // Suggest where to break the line
                let break_suggestion = self.suggest_line_break(line, max_length);

                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: max_length + 1,
                    message: format!(
                        "Line is {} characters long (max: {}). {}",
                        visual_length, max_length, break_suggestion
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Note: We're not recursing into child nodes as we're checking the entire source
    }

    fn calculate_visual_length(&self, line: &str) -> usize {
        let mut length = 0;
        for ch in line.chars() {
            if ch == '\t' {
                // Tab counts as 4 spaces
                length += 4;
            } else {
                length += 1;
            }
        }
        length
    }

    fn suggest_line_break(&self, line: &str, max_length: usize) -> String {
        let lower_line = line.to_lowercase();

        // Find potential break points before max_length
        let mut break_points = Vec::new();

        // Check for commas
        for (i, ch) in line.chars().enumerate() {
            if i < max_length && ch == ',' {
                break_points.push((i, "after comma"));
            }
        }

        // Check for SQL keywords
        let keywords = [
            "and", "or", "where", "from", "join", "on", "group", "order", "having",
        ];
        for keyword in keywords.iter() {
            if let Some(pos) = lower_line.find(&format!(" {} ", keyword)) {
                if pos < max_length {
                    break_points.push((pos, "before keyword"));
                }
            }
        }

        // Check for operators
        let operators = [
            " = ", " != ", " <> ", " < ", " > ", " <= ", " >= ", " + ", " - ",
        ];
        for op in operators.iter() {
            if let Some(pos) = line.find(op) {
                if pos < max_length {
                    break_points.push((pos, "around operator"));
                }
            }
        }

        if !break_points.is_empty() {
            // Sort by position descending to get the latest possible break point
            break_points.sort_by(|a, b| b.0.cmp(&a.0));
            let (_, location) = break_points[0];
            format!("Consider breaking line {}", location)
        } else {
            "Consider breaking this long line into multiple lines".to_string()
        }
    }
}
