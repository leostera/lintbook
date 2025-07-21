use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct MultiAssignments;

impl Rule for MultiAssignments {
    fn id(&self) -> &'static str {
        "RS109"
    }

    fn name(&self) -> &'static str {
        "multi-assignments"
    }

    fn description(&self) -> &'static str {
        "Checks for multiple assignments in one statement"
    }

    fn explanation(&self) -> &'static str {
        "Multiple assignments in one statement can be confusing. \
         Consider using separate statements for clarity."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl MultiAssignments {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "assignment_expression" {
            if let Some(right) = node.child_by_field_name("right") {
                if right.kind() == "assignment_expression" {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "Multiple assignments in one statement - consider separating for clarity".to_string(),
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