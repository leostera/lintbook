use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct ModuloOne;

impl Rule for ModuloOne {
    fn id(&self) -> &'static str {
        "RS035"
    }

    fn name(&self) -> &'static str {
        "modulo-one"
    }

    fn description(&self) -> &'static str {
        "Checks for getting the remainder of integer division by one or minus one"
    }

    fn explanation(&self) -> &'static str {
        "The result for a divisor of one can only ever be zero; for minus one it can cause panic/overflow \
        (if the left operand is the minimal value of the respective integer type) or results in zero. \
        No one will write such code deliberately, unless trying to win an Underhanded Rust Contest."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl ModuloOne {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for binary expressions with modulo operator
        if node.kind() == "binary_expression" {
            if let Some(operator) = node.child_by_field_name("operator") {
                let operator_text = &source[operator.byte_range()];

                if operator_text == "%" {
                    if let Some(right) = node.child_by_field_name("right") {
                        if self.is_modulo_one_operand(right, source) {
                            let position = node.start_position();
                            let right_text = &source[right.byte_range()];

                            let message = if right_text == "1" {
                                "Modulo by one will always result in zero"
                            } else if right_text == "-1" {
                                "Modulo by minus one can cause panic/overflow or always results in zero"
                            } else {
                                "Modulo by one or minus one is problematic"
                            };

                            violations.push(LintViolation {
                                line: position.row + 1,
                                column: position.column + 1,
                                message: message.to_string(),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
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

    fn is_modulo_one_operand(&self, node: Node, source: &str) -> bool {
        let text = source[node.byte_range()].trim();

        match node.kind() {
            "integer_literal" => {
                // Direct literal values
                text == "1" || text == "-1"
            }
            "unary_expression" => {
                // Handle negative numbers like -1
                if let Some(operator) = node.child_by_field_name("operator") {
                    let op_text = &source[operator.byte_range()];
                    if op_text == "-" {
                        if let Some(argument) = node.child_by_field_name("argument") {
                            let arg_text = &source[argument.byte_range()];
                            return arg_text == "1";
                        }
                    }
                }
                false
            }
            "parenthesized_expression" => {
                // Handle parenthesized expressions like (1) or (-1)
                if let Some(inner) = node.named_child(0) {
                    self.is_modulo_one_operand(inner, source)
                } else {
                    false
                }
            }
            _ => {
                // For other node types, check if the text is exactly "1" or "-1"
                // This handles cases like const values that evaluate to 1 or -1
                text == "1" || text == "-1"
            }
        }
    }
}
