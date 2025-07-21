use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct TypeCastingStyle;

impl Rule for TypeCastingStyle {
    fn id(&self) -> &'static str {
        "SQL023"
    }

    fn name(&self) -> &'static str {
        "type-casting-style"
    }

    fn description(&self) -> &'static str {
        "Use consistent type casting style"
    }

    fn explanation(&self) -> &'static str {
        "SQL supports multiple type casting syntaxes: CAST(value AS type), value::type (PostgreSQL), 
        and CONVERT(type, value) (SQL Server). Choose one style for consistency."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cast_styles = Vec::new();

        // First pass: collect all casting styles
        self.collect_cast_styles(tree.root_node(), source, &mut cast_styles);

        // Determine preferred style (most common)
        if cast_styles.is_empty() {
            return violations;
        }

        let mut style_counts = std::collections::HashMap::new();
        for style in &cast_styles {
            *style_counts.entry(style.style.clone()).or_insert(0) += 1;
        }

        let preferred_style = style_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(style, _)| style.clone())
            .unwrap_or_else(|| "CAST".to_string());

        // Second pass: report inconsistencies
        for cast_info in cast_styles {
            if cast_info.style != preferred_style {
                violations.push(LintViolation {
                    line: cast_info.line,
                    column: cast_info.column,
                    message: format!(
                        "Inconsistent type casting style '{}'. Use '{}' style for consistency",
                        cast_info.style, preferred_style
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        violations
    }
}

#[derive(Debug)]
struct CastStyle {
    style: String,
    line: usize,
    column: usize,
}

impl TypeCastingStyle {
    fn collect_cast_styles(
        &self,
        node: tree_sitter::Node,
        source: &str,
        styles: &mut Vec<CastStyle>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Find CAST function calls
            let mut pos = 0;
            while let Some(cast_pos) = lower_line[pos..].find("cast(") {
                let actual_pos = pos + cast_pos;
                let start_pos = node.start_position();
                styles.push(CastStyle {
                    style: "CAST".to_string(),
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + actual_pos + 1,
                });
                pos = actual_pos + 5;
            }

            // Find CONVERT function calls (SQL Server style)
            let mut pos = 0;
            while let Some(convert_pos) = lower_line[pos..].find("convert(") {
                let actual_pos = pos + convert_pos;
                let start_pos = node.start_position();
                styles.push(CastStyle {
                    style: "CONVERT".to_string(),
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + actual_pos + 1,
                });
                pos = actual_pos + 8;
            }

            // Find PostgreSQL-style casting (::)
            let mut pos = 0;
            while let Some(double_colon_pos) = line[pos..].find("::") {
                let actual_pos = pos + double_colon_pos;

                // Make sure this is not in a comment
                let before_colon = &line[..actual_pos];
                if !before_colon.contains("--") {
                    let start_pos = node.start_position();
                    styles.push(CastStyle {
                        style: "::".to_string(),
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + actual_pos + 1,
                    });
                }
                pos = actual_pos + 2;
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_cast_styles(child, source, styles);
            }
        }
    }
}
