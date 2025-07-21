use crate::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct EqOp;

impl Rule for EqOp {
    fn id(&self) -> &'static str {
        "RS013"
    }

    fn name(&self) -> &'static str {
        "eq-op"
    }

    fn description(&self) -> &'static str {
        "Detects equal operands in binary operations"
    }

    fn explanation(&self) -> &'static str {
        "Binary operations with equal operands are usually mistakes. For example, \
         `x == x` is always true, `x - x` is always 0, and `x / x` is always 1 (except for 0 or NaN). \
         This may indicate a copy-paste error or logical mistake."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl EqOp {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "binary_expression" {
            if let Some(operator) = get_binary_operator(node, source) {
                if let (Some(left), Some(right)) = (
                    node.child_by_field_name("left"),
                    node.child_by_field_name("right"),
                ) {
                    let left_text = &source[left.byte_range()];
                    let right_text = &source[right.byte_range()];

                    // Skip if operands are different
                    if left_text != right_text {
                        // Continue with recursion
                    } else if !is_potentially_intentional(&operator, left_text) {
                        let position = node.start_position();
                        let result_hint = get_operation_result_hint(&operator);

                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Equal operands in binary operation: `{} {} {}` {}",
                                left_text, operator, right_text, result_hint
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

fn get_binary_operator(node: Node, source: &str) -> Option<String> {
    if let Some(operator_node) = node.child_by_field_name("operator") {
        let op_text = &source[operator_node.byte_range()];
        match op_text {
            "==" | "!=" | "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "/" | "%" | "&&" | "||"
            | "&" | "|" | "^" | "<<" | ">>" => Some(op_text.to_string()),
            _ => None,
        }
    } else {
        None
    }
}

fn is_potentially_intentional(operator: &str, operand: &str) -> bool {
    // Some cases where equal operands might be intentional:

    // Bitwise operations on constants might be intentional
    if matches!(operator, "&" | "|" | "^") && is_constant_like(operand) {
        return true;
    }

    // Self-comparison for NaN checking (x != x)
    if operator == "!=" && looks_like_float_check(operand) {
        return true;
    }

    // Addition and multiplication might be intentional (doubling, squaring)
    if matches!(operator, "+" | "*") {
        return true;
    }

    false
}

fn is_constant_like(text: &str) -> bool {
    // Check if the operand looks like a constant
    text.chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
        || text.starts_with("0x")
        || text.starts_with("0b")
        || text.parse::<f64>().is_ok()
}

fn looks_like_float_check(text: &str) -> bool {
    // Common patterns for float variables
    text.contains("float") || text.contains("f32") || text.contains("f64") || text.ends_with("_f")
}

fn get_operation_result_hint(operator: &str) -> &'static str {
    match operator {
        "==" => "(always true)",
        "!=" => "(always false)",
        "<" | ">" => "(always false)",
        "<=" | ">=" => "(always true)",
        "-" => "(always 0)",
        "/" => "(always 1, unless 0)",
        "%" => "(always 0)",
        "^" => "(always 0)",
        "&&" | "||" => "(redundant)",
        _ => "",
    }
}
