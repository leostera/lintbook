use crate::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct ErasingOp;

impl Rule for ErasingOp {
    fn id(&self) -> &'static str {
        "RS014"
    }

    fn name(&self) -> &'static str {
        "erasing-op"
    }

    fn description(&self) -> &'static str {
        "Detects operations that always return a constant value regardless of operands"
    }

    fn explanation(&self) -> &'static str {
        "Operations like `x * 0`, `x & 0`, `x | !0`, etc. always produce the same result \
         regardless of the value of `x`. This might indicate a logic error or unnecessary computation."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl ErasingOp {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "binary_expression" {
            if let Some(operator) = get_erasing_operator(node, source) {
                if let (Some(left), Some(right)) = (
                    node.child_by_field_name("left"),
                    node.child_by_field_name("right"),
                ) {
                    let left_text = &source[left.byte_range()];
                    let right_text = &source[right.byte_range()];

                    if let Some((erasing_result, explanation)) =
                        check_erasing_operation(&operator, left_text, right_text)
                    {
                        let position = node.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "This operation always evaluates to {}: {}",
                                erasing_result, explanation
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

fn get_erasing_operator(node: Node, source: &str) -> Option<String> {
    if let Some(operator_node) = node.child_by_field_name("operator") {
        let op_text = &source[operator_node.byte_range()];
        match op_text {
            "*" | "&" | "|" | "^" | "<<" | ">>" => Some(op_text.to_string()),
            _ => None,
        }
    } else {
        None
    }
}

fn check_erasing_operation(operator: &str, left: &str, right: &str) -> Option<(String, String)> {
    match operator {
        "*" => {
            if is_zero(left) || is_zero(right) {
                Some((
                    "0".to_string(),
                    format!("`{} {} {}` - multiplication by zero", left, operator, right),
                ))
            } else {
                None
            }
        }
        "&" => {
            if is_zero(left) || is_zero(right) {
                Some((
                    "0".to_string(),
                    format!("`{} {} {}` - bitwise AND with zero", left, operator, right),
                ))
            } else if is_all_ones(left) {
                Some((
                    right.to_string(),
                    format!(
                        "`{} {} {}` - bitwise AND with all 1s",
                        left, operator, right
                    ),
                ))
            } else if is_all_ones(right) {
                Some((
                    left.to_string(),
                    format!(
                        "`{} {} {}` - bitwise AND with all 1s",
                        left, operator, right
                    ),
                ))
            } else {
                None
            }
        }
        "|" => {
            if is_all_ones(left) || is_all_ones(right) {
                Some((
                    "all 1s".to_string(),
                    format!("`{} {} {}` - bitwise OR with all 1s", left, operator, right),
                ))
            } else if is_zero(left) {
                Some((
                    right.to_string(),
                    format!("`{} {} {}` - bitwise OR with zero", left, operator, right),
                ))
            } else if is_zero(right) {
                Some((
                    left.to_string(),
                    format!("`{} {} {}` - bitwise OR with zero", left, operator, right),
                ))
            } else {
                None
            }
        }
        "^" => {
            if is_zero(left) {
                Some((
                    right.to_string(),
                    format!("`{} {} {}` - XOR with zero", left, operator, right),
                ))
            } else if is_zero(right) {
                Some((
                    left.to_string(),
                    format!("`{} {} {}` - XOR with zero", left, operator, right),
                ))
            } else {
                None
            }
        }
        "<<" | ">>" => {
            // Shifting by more than the bit width is problematic
            if let Ok(shift_amount) = right.parse::<u32>() {
                if shift_amount >= 64 {
                    // Assume 64-bit for simplicity
                    Some((
                        "undefined/0".to_string(),
                        format!(
                            "`{} {} {}` - shift by {} bits",
                            left, operator, right, shift_amount
                        ),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_zero(text: &str) -> bool {
    matches!(
        text,
        "0" | "0u8"
            | "0u16"
            | "0u32"
            | "0u64"
            | "0u128"
            | "0usize"
            | "0i8"
            | "0i16"
            | "0i32"
            | "0i64"
            | "0i128"
            | "0isize"
            | "0.0"
            | "0.0f32"
            | "0.0f64"
            | "0x00"
            | "0x0000"
            | "0x00000000"
            | "0x0000000000000000"
    ) || text.parse::<i64>().map_or(false, |n| n == 0)
}

fn is_all_ones(text: &str) -> bool {
    // Common patterns for all 1s
    matches!(
        text,
        "!0" | "u8::MAX"
            | "u16::MAX"
            | "u32::MAX"
            | "u64::MAX"
            | "u128::MAX"
            | "usize::MAX"
            | "0xFF"
            | "0xFFFF"
            | "0xFFFFFFFF"
            | "0xFFFFFFFFFFFFFFFF"
            | "0b11111111"
            | "255"
            | "65535"
            | "4294967295"
            | "0xffffff"
            | "0xffffffff"
    ) || text.starts_with("0xFF")
        || text.starts_with("0b1111")
}
