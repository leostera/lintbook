use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct TrailingWhitespace;

impl Rule for TrailingWhitespace {
    fn id(&self) -> &'static str {
        "SQL032"
    }

    fn name(&self) -> &'static str {
        "trailing-whitespace"
    }

    fn description(&self) -> &'static str {
        "Lines should not have trailing whitespace"
    }

    fn explanation(&self) -> &'static str {
        "Trailing whitespace at the end of lines is unnecessary and can cause issues 
        with version control systems. Remove all spaces and tabs at the end of lines."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_trailing_whitespace(tree.root_node(), source, &mut violations);

        violations
    }
}

impl TrailingWhitespace {
    fn check_trailing_whitespace(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let lines: Vec<&str> = source.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Skip completely empty lines
            if line.is_empty() {
                continue;
            }
            
            // Check if line ends with whitespace
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed = line.trim_end();
                let trailing_count = line.len() - trimmed.len();
                
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: trimmed.len() + 1,
                    message: format!(
                        "Line has {} trailing whitespace character{}",
                        trailing_count,
                        if trailing_count == 1 { "" } else { "s" }
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
            
            // Also check for other whitespace characters
            let last_char = line.chars().last();
            if let Some(ch) = last_char {
                if ch.is_whitespace() && ch != ' ' && ch != '\t' {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: line.len(),
                        message: "Line ends with non-standard whitespace character".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        // Note: We're not recursing into child nodes as we're checking the entire source
    }
}