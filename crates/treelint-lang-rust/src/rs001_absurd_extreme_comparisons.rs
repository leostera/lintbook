use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct AbsurdExtremeComparisons;

impl Rule for AbsurdExtremeComparisons {
    fn id(&self) -> &'static str {
        "RS001"
    }

    fn name(&self) -> &'static str {
        "absurd-extreme-comparisons"
    }

    fn description(&self) -> &'static str {
        "Checks for comparisons with extreme values that are always true or false"
    }

    fn explanation(&self) -> &'static str {
        "Comparisons with extreme values like `x >= u32::MAX` or `x < i32::MIN` are typically mistakes. \
         These comparisons will always evaluate to the same value and may indicate a logic error."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl AbsurdExtremeComparisons {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "binary_expression" {
            if let Some(operator) = get_comparison_operator(node, source) {
                if let (Some(left), Some(right)) = (
                    node.child_by_field_name("left"),
                    node.child_by_field_name("right"),
                ) {
                    let left_text = &source[left.byte_range()];
                    let right_text = &source[right.byte_range()];

                    if is_extreme_value(left_text) || is_extreme_value(right_text) {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Comparison with extreme value: `{} {} {}` will likely always be true or false",
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

fn is_extreme_value(text: &str) -> bool {
    // Common extreme values in Rust
    matches!(
        text,
        "u8::MAX"
            | "u8::MIN"
            | "u16::MAX"
            | "u16::MIN"
            | "u32::MAX"
            | "u32::MIN"
            | "u64::MAX"
            | "u64::MIN"
            | "u128::MAX"
            | "u128::MIN"
            | "usize::MAX"
            | "usize::MIN"
            | "i8::MAX"
            | "i8::MIN"
            | "i16::MAX"
            | "i16::MIN"
            | "i32::MAX"
            | "i32::MIN"
            | "i64::MAX"
            | "i64::MIN"
            | "i128::MAX"
            | "i128::MIN"
            | "isize::MAX"
            | "isize::MIN"
            | "f32::INFINITY"
            | "f32::NEG_INFINITY"
            | "f64::INFINITY"
            | "f64::NEG_INFINITY"
            | "255"
            | "65535"
            | "4294967295" // Common max values as literals
    )
}
