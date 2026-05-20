use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct UnitCmp;

impl Rule for UnitCmp {
    fn id(&self) -> &'static str {
        "RS059"
    }

    fn name(&self) -> &'static str {
        "unit-cmp"
    }

    fn description(&self) -> &'static str {
        "Checks for comparisons with unit type ()"
    }

    fn explanation(&self) -> &'static str {
        "Comparing unit types `()` is usually meaningless since all unit values are equal. \
         This comparison will always evaluate to true and may indicate a logic error."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl UnitCmp {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "binary_expression" {
            if let Some(operator) = get_comparison_operator(node, source) {
                if let (Some(left), Some(right)) = (
                    node.child_by_field_name("left"),
                    node.child_by_field_name("right"),
                ) {
                    let left_text = &source[left.byte_range()].trim();
                    let right_text = &source[right.byte_range()].trim();

                    // Check if either side is a unit type
                    let is_unit_comparison = is_unit_value(left_text) || is_unit_value(right_text);

                    if is_unit_comparison {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Comparison with unit type: `{} {} {}` will always be true",
                                left_text, operator, right_text
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn get_comparison_operator(node: Node, source: &str) -> Option<String> {
    if let Some(operator_node) = node.child_by_field_name("operator") {
        let op_text = &source[operator_node.byte_range()];
        match op_text {
            "==" | "!=" | "<" | "<=" | ">" | ">=" => Some(op_text.to_string()),
            _ => None,
        }
    } else {
        None
    }
}

fn is_unit_value(text: &str) -> bool {
    // Check for unit literal "()"
    text == "()" ||
    // Check for function calls that return unit (common patterns)
    text.ends_with("()") && (
        text.ends_with("println!()") ||
        text.ends_with("print!()") ||
        text.ends_with("panic!()") ||
        text.ends_with("unreachable!()") ||
        text.ends_with("todo!()")
    )
}
