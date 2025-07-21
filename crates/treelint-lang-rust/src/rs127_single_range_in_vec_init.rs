use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct SingleRangeInVecInit;

impl Rule for SingleRangeInVecInit {
    fn id(&self) -> &'static str {
        "RS127"
    }

    fn name(&self) -> &'static str {
        "single-range-in-vec-init"
    }

    fn description(&self) -> &'static str {
        "Checks for single range in Vec initialization"
    }

    fn explanation(&self) -> &'static str {
        "Using vec![0..n] creates a Vec containing a single Range object, not numbers 0 to n. \
         Use (0..n).collect::<Vec<_>>() if you want the individual numbers."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl SingleRangeInVecInit {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "macro_invocation" {
            let macro_text = source[node.byte_range()].trim();
            
            if macro_text.starts_with("vec![") && 
               macro_text.contains("..") &&
               !macro_text.contains(",") &&
               !macro_text.contains(";") {
                // Check if it's a single range like vec![0..10]
                if is_single_range_in_vec(macro_text) {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "vec![range] creates a Vec with single Range object - use (range).collect() for individual values".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_single_range_in_vec(macro_text: &str) -> bool {
    // Extract content between vec![ and ]
    if let Some(start) = macro_text.find("vec![") {
        if let Some(end) = macro_text.rfind(']') {
            let content = &macro_text[start + 5..end].trim();
            // Check if it's just a range expression
            return content.contains("..") && !content.contains(',');
        }
    }
    false
}