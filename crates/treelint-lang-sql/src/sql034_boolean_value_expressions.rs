use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct BooleanValueExpressions;

impl Rule for BooleanValueExpressions {
    fn id(&self) -> &'static str {
        "SQL034"
    }

    fn name(&self) -> &'static str {
        "boolean-value-expressions"
    }

    fn description(&self) -> &'static str {
        "Avoid redundant boolean comparisons"
    }

    fn explanation(&self) -> &'static str {
        "Comparing boolean expressions to TRUE/FALSE is redundant. Instead of 
        'WHERE is_active = TRUE', use 'WHERE is_active'. Instead of 
        'WHERE is_deleted = FALSE', use 'WHERE NOT is_deleted'."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_boolean_comparisons(tree.root_node(), source, &mut violations);

        violations
    }
}

impl BooleanValueExpressions {
    fn check_boolean_comparisons(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            
            // Look for patterns like "= true", "= false", "!= true", "!= false", "<> true", "<> false"
            let patterns = [
                ("= true", "Remove '= TRUE', use the boolean expression directly"),
                ("=true", "Remove '=TRUE', use the boolean expression directly"),
                ("= false", "Replace '= FALSE' with 'NOT'"),
                ("=false", "Replace '=FALSE' with 'NOT'"),
                ("!= true", "Replace '!= TRUE' with 'NOT'"),
                ("!=true", "Replace '!=TRUE' with 'NOT'"),
                ("!= false", "Remove '!= FALSE', use the boolean expression directly"),
                ("!=false", "Remove '!=FALSE', use the boolean expression directly"),
                ("<> true", "Replace '<> TRUE' with 'NOT'"),
                ("<>true", "Replace '<>TRUE' with 'NOT'"),
                ("<> false", "Remove '<> FALSE', use the boolean expression directly"),
                ("<>false", "Remove '<>FALSE', use the boolean expression directly"),
                ("is true", "Remove 'IS TRUE', use the boolean expression directly"),
                ("is false", "Replace 'IS FALSE' with 'NOT' or check for NULL values if needed"),
                ("is not true", "Replace 'IS NOT TRUE' with 'NOT' or check for NULL values if needed"),
                ("is not false", "Remove 'IS NOT FALSE', use the boolean expression directly or check for NULL"),
            ];
            
            for (pattern, message) in patterns.iter() {
                if lower_line.contains(pattern) {
                    // Try to extract the column/expression name before the comparison
                    if let Some(pos) = lower_line.find(pattern) {
                        let before = &line[..pos];
                        let words: Vec<&str> = before.split_whitespace().collect();
                        
                        if !words.is_empty() {
                            let last_word = words[words.len() - 1];
                            
                            // Skip if it looks like a string literal or comment
                            if !last_word.contains("'") && !last_word.contains("\"") && !before.trim_end().ends_with("--") {
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
                }
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_boolean_comparisons(child, source, violations);
            }
        }
    }
}