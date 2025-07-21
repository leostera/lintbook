use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct MutRangeBound;

impl Rule for MutRangeBound {
    fn id(&self) -> &'static str {
        "RS111"
    }

    fn name(&self) -> &'static str {
        "mut-range-bound"
    }

    fn description(&self) -> &'static str {
        "Checks for mutable variables used in range bounds"
    }

    fn explanation(&self) -> &'static str {
        "Using mutable variables as range bounds can be confusing since \
         modifying them won't affect the range. Consider using immutable variables."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl MutRangeBound {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if matches!(node.kind(), "range_expression" | "range_inclusive_expression") {
            // Check left and right bounds
            if let Some(left) = node.child_by_field_name("left") {
                if is_mutable_variable(left, source) {
                    let position = left.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "Mutable variable used as range bound - consider using immutable variable".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
            
            if let Some(right) = node.child_by_field_name("right") {
                if is_mutable_variable(right, source) {
                    let position = right.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "Mutable variable used as range bound - consider using immutable variable".to_string(),
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

fn is_mutable_variable(node: Node, source: &str) -> bool {
    if node.kind() == "identifier" {
        let var_name = source[node.byte_range()].trim();
        // This is a simplified check - in practice you'd need to track variable declarations
        // For now, we'll check for common mutable naming patterns
        var_name.starts_with("mut_") || var_name.ends_with("_mut")
    } else {
        false
    }
}