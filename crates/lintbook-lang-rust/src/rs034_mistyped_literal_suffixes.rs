use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct MistypedLiteralSuffixes;

impl Rule for MistypedLiteralSuffixes {
    fn id(&self) -> &'static str {
        "RS034"
    }

    fn name(&self) -> &'static str {
        "mistyped-literal-suffixes"
    }

    fn description(&self) -> &'static str {
        "Warns for mistyped suffix in literals"
    }

    fn explanation(&self) -> &'static str {
        "This is most probably a typo. Common mistakes include using `2_32` instead of `2_i32` or `250_8` instead of `250_u8`."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl MistypedLiteralSuffixes {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for integer literals
        if node.kind() == "integer_literal" {
            let literal_text = &source[node.byte_range()];

            if let Some(suggestion) = self.detect_mistyped_suffix(literal_text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: format!(
                        "Mistyped literal suffix: `{}` should probably be `{}`",
                        literal_text, suggestion
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn detect_mistyped_suffix(&self, literal: &str) -> Option<String> {
        // Remove any underscores for analysis but preserve them in suggestions
        let cleaned = literal.replace('_', "");

        // Check for common mistyped suffixes
        // Pattern: number followed by underscore and suspicious suffix
        if let Some(underscore_pos) = literal.rfind('_') {
            let (number_part, suffix_part) = literal.split_at(underscore_pos + 1);

            match suffix_part {
                // Check for floating point mistyped suffixes first
                "32" if literal.contains('.') => Some(format!("{}f32", number_part)),
                "64" if literal.contains('.') => Some(format!("{}f64", number_part)),

                // Common integer type suffixes that are mistyped - default to i32/i64 for common sizes
                "32" => Some(format!("{}i32", number_part)),
                "64" => Some(format!("{}i64", number_part)),

                // Smaller sizes default to unsigned
                "8" => Some(format!("{}u8", number_part)),
                "16" => Some(format!("{}u16", number_part)),
                "128" => Some(format!("{}u128", number_part)),

                // Other common mistakes
                "size" => Some(format!("{}usize", number_part)),

                _ => None,
            }
        } else {
            // Check for patterns without underscore but with suspicious numeric endings
            if cleaned.len() > 2 {
                if cleaned.ends_with("32")
                    && !cleaned.ends_with("f32")
                    && !cleaned.ends_with("i32")
                    && !cleaned.ends_with("u32")
                {
                    let base = &literal[..literal.len() - 2];
                    if base.chars().all(|c| c.is_ascii_digit()) {
                        return Some(format!("{}_i32", base));
                    }
                } else if cleaned.ends_with("64")
                    && !cleaned.ends_with("f64")
                    && !cleaned.ends_with("i64")
                    && !cleaned.ends_with("u64")
                {
                    let base = &literal[..literal.len() - 2];
                    if base.chars().all(|c| c.is_ascii_digit()) {
                        return Some(format!("{}_i64", base));
                    }
                } else if cleaned.ends_with("16")
                    && !cleaned.ends_with("i16")
                    && !cleaned.ends_with("u16")
                {
                    let base = &literal[..literal.len() - 2];
                    if base.chars().all(|c| c.is_ascii_digit()) {
                        return Some(format!("{}_u16", base));
                    }
                } else if cleaned.ends_with("8")
                    && !cleaned.ends_with("i8")
                    && !cleaned.ends_with("u8")
                {
                    let base = &literal[..literal.len() - 1];
                    if base.chars().all(|c| c.is_ascii_digit())
                        && base.parse::<u32>().map_or(false, |n| n <= 255)
                    {
                        return Some(format!("{}_u8", base));
                    }
                }
            }
            None
        }
    }
}
