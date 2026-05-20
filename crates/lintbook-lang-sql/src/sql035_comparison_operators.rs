use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct ComparisonOperators;

impl Rule for ComparisonOperators {
    fn id(&self) -> &'static str {
        "SQL035"
    }

    fn name(&self) -> &'static str {
        "comparison-operators"
    }

    fn description(&self) -> &'static str {
        "Use standard comparison operators"
    }

    fn explanation(&self) -> &'static str {
        "Use standard SQL comparison operators for better portability. Avoid non-standard
        operators like '!<', '!>', or other database-specific operators. Use standard
        operators: =, <>, !=, <, >, <=, >=, IS NULL, IS NOT NULL, LIKE, IN, BETWEEN."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_comparison_operators(tree.root_node(), source, &mut violations);

        violations
    }
}

impl ComparisonOperators {
    fn check_comparison_operators(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Look for non-standard operators
            let non_standard_ops = [
                ("!<", "Use '>=' instead of '!<'"),
                ("!>", "Use '<=' instead of '!>'"),
                ("~=", "Use '<>' or '!=' instead of '~='"),
                ("~<", "Use standard comparison operators"),
                ("~>", "Use standard comparison operators"),
                ("!~", "Use NOT LIKE instead of '!~'"),
                ("@@", "Avoid database-specific operators like '@@'"),
                ("<=>", "Use IS NOT DISTINCT FROM or standard comparison"),
                ("!!<", "Use standard comparison operators"),
                ("!!>", "Use standard comparison operators"),
            ];

            for (op, message) in non_standard_ops.iter() {
                if line.contains(op) {
                    // Check it's not in a string literal or comment
                    if let Some(pos) = self.find_operator_position(line, op) {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + pos + 1,
                            message: message.to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }

            // Check for ISNULL/NOTNULL functions (should use IS NULL/IS NOT NULL)
            let lower_line = line.to_lowercase();
            if lower_line.contains("isnull(") && !lower_line.contains("ifnull(") {
                if let Some(pos) = lower_line.find("isnull(") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + pos + 1,
                        message: "Use 'IS NULL' instead of 'ISNULL()' function".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_comparison_operators(child, source, violations);
            }
        }
    }

    fn find_operator_position(&self, line: &str, operator: &str) -> Option<usize> {
        let mut in_string = false;
        let mut string_char = ' ';
        let chars: Vec<char> = line.chars().collect();

        for i in 0..chars.len() {
            // Handle string literals
            if !in_string && (chars[i] == '\'' || chars[i] == '"') {
                in_string = true;
                string_char = chars[i];
            } else if in_string && chars[i] == string_char {
                // Check if it's escaped
                if i == 0 || chars[i - 1] != '\\' {
                    in_string = false;
                }
            }

            // Skip if we're in a string
            if in_string {
                continue;
            }

            // Check for comment
            if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' {
                break; // Rest of line is comment
            }

            // Check if operator starts at this position
            if i + operator.len() <= chars.len() {
                let slice: String = chars[i..i + operator.len()].iter().collect();
                if slice == operator {
                    return Some(i);
                }
            }
        }

        None
    }
}
