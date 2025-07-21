use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct CommasPlacement;

impl Rule for CommasPlacement {
    fn id(&self) -> &'static str {
        "SQL041"
    }

    fn name(&self) -> &'static str {
        "commas-placement"
    }

    fn description(&self) -> &'static str {
        "Use consistent comma placement style"
    }

    fn explanation(&self) -> &'static str {
        "Be consistent with comma placement. Use either trailing commas (at the end of lines) 
        or leading commas (at the beginning of lines) throughout your SQL. Leading commas 
        make it easier to comment out lines and spot missing commas."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_comma_placement(tree.root_node(), source, &mut violations);

        violations
    }
}

impl CommasPlacement {
    fn check_comma_placement(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();
        
        let mut in_list = false;
        let mut list_start_line = 0;
        let mut comma_styles = Vec::new(); // (line_idx, is_trailing)

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            let trimmed = line.trim();
            
            // Start of potential comma-separated list
            if lower_line.contains("select ") || 
               lower_line.contains("values (") ||
               lower_line.contains("insert into") ||
               lower_line.contains("order by") ||
               lower_line.contains("group by") {
                in_list = true;
                list_start_line = line_idx;
                comma_styles.clear();
            }
            
            // End of comma-separated list
            if in_list && (lower_line.contains(" from ") || 
                          lower_line.contains(" where ") ||
                          lower_line.contains(");") ||
                          lower_line.contains(" having ") ||
                          lower_line.contains(" limit ")) {
                
                self.analyze_comma_consistency(&comma_styles, list_start_line, node, violations);
                in_list = false;
                comma_styles.clear();
            }
            
            // Track comma placement within lists
            if in_list && trimmed.contains(',') {
                let has_trailing_comma = trimmed.ends_with(',');
                let has_leading_comma = trimmed.starts_with(',');
                
                if has_trailing_comma {
                    comma_styles.push((line_idx, true)); // trailing
                } else if has_leading_comma {
                    comma_styles.push((line_idx, false)); // leading
                }
            }
        }

        // Check the final list if we ended in one
        if in_list && !comma_styles.is_empty() {
            self.analyze_comma_consistency(&comma_styles, list_start_line, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_comma_placement(child, source, violations);
            }
        }
    }
    
    fn analyze_comma_consistency(
        &self,
        comma_styles: &[(usize, bool)],
        _start_line: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        if comma_styles.len() < 2 {
            return; // Need at least 2 commas to check consistency
        }
        
        let trailing_count = comma_styles.iter().filter(|(_, is_trailing)| *is_trailing).count();
        let leading_count = comma_styles.iter().filter(|(_, is_trailing)| !*is_trailing).count();
        
        // If we have both styles, report inconsistency
        if trailing_count > 0 && leading_count > 0 {
            // Report on the minority style
            let report_trailing = trailing_count < leading_count;
            
            for (line_idx, is_trailing) in comma_styles {
                if *is_trailing == report_trailing {
                    let start_pos = node.start_position();
                    let expected_style = if report_trailing { "leading" } else { "trailing" };
                    let current_style = if *is_trailing { "trailing" } else { "leading" };
                    
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: format!(
                            "Inconsistent comma placement: found {} comma but {} commas are used elsewhere in this list",
                            current_style, expected_style
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}