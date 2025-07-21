use treelint_core::*;
use tree_sitter::{Node, Tree};

pub struct PossibleMissingComma;

impl Rule for PossibleMissingComma {
    fn id(&self) -> &'static str {
        "RS046"
    }

    fn name(&self) -> &'static str {
        "possible-missing-comma"
    }

    fn description(&self) -> &'static str {
        "Checks for possible missing comma in an array"
    }

    fn explanation(&self) -> &'static str {
        "This could lead to unexpected results. When an array element is a binary operator expression \
        that spans multiple lines, it might indicate a missing comma between array elements."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl PossibleMissingComma {
    fn check_node(
        &self,
        node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for array expressions
        if node.kind() == "array_expression" {
            self.check_array_elements(node, source, violations);
        }
        
        // Also check vec! macro calls
        if node.kind() == "macro_invocation" {
            if let Some(macro_name) = node.child_by_field_name("macro") {
                let name_text = &source[macro_name.byte_range()];
                if name_text == "vec" {
                    self.check_macro_elements(node, source, violations);
                }
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn check_array_elements(
        &self,
        array_node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut cursor = array_node.walk();
        let mut prev_element: Option<Node> = None;
        
        for child in array_node.children(&mut cursor) {
            // Look for consecutive expressions that might be missing commas
            if self.is_array_element(child) {
                if let Some(prev) = prev_element {
                    self.check_potential_missing_comma(prev, child, source, violations);
                }
                prev_element = Some(child);
            }
        }
    }

    fn check_macro_elements(
        &self,
        macro_node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for token_tree containing the macro arguments
        let mut cursor = macro_node.walk();
        for child in macro_node.children(&mut cursor) {
            if child.kind() == "token_tree" {
                self.check_token_tree_elements(child, source, violations);
            }
        }
    }

    fn check_token_tree_elements(
        &self,
        token_tree: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut cursor = token_tree.walk();
        let mut prev_element: Option<Node> = None;
        
        for child in token_tree.children(&mut cursor) {
            // Skip punctuation like commas, brackets
            if !self.is_punctuation(child, source) {
                if let Some(prev) = prev_element {
                    self.check_potential_missing_comma(prev, child, source, violations);
                }
                prev_element = Some(child);
            }
        }
    }

    fn check_potential_missing_comma(
        &self,
        prev_element: Node,
        current_element: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check if elements are on different lines
        let prev_end_line = prev_element.end_position().row;
        let current_start_line = current_element.start_position().row;
        
        if current_start_line > prev_end_line {
            // Elements are on different lines - check if this looks like a missing comma
            if self.looks_like_missing_comma(prev_element, current_element, source) {
                let position = current_element.start_position();
                
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: "Possible missing comma in array. This could be a binary operation or separate array elements".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn looks_like_missing_comma(
        &self,
        prev_element: Node,
        current_element: Node,
        source: &str,
    ) -> bool {
        // Check for patterns that suggest missing comma
        
        // Pattern 1: Previous element ends with a literal, current starts with operator
        // Example: "-3" followed by "-4" (could be "-3, -4" or "-3 - 4")
        if self.ends_with_literal_or_identifier(prev_element, source) &&
           self.starts_with_unary_operator(current_element, source) {
            return true;
        }
        
        // Pattern 2: Both elements are literals/identifiers without operators between
        // Example: "a" followed by "b" (likely missing comma)
        if self.is_simple_literal_or_identifier(prev_element, source) &&
           self.is_simple_literal_or_identifier(current_element, source) {
            return true;
        }
        
        // Pattern 3: Previous element is complex expression, current is simple literal
        // Example: "x + y" followed by "z" (could be missing comma)
        if self.is_complex_expression(prev_element) &&
           self.is_simple_literal_or_identifier(current_element, source) {
            return true;
        }
        
        false
    }

    fn ends_with_literal_or_identifier(&self, node: Node, source: &str) -> bool {
        match node.kind() {
            "integer_literal" | "float_literal" | "string_literal" | "identifier" => true,
            "unary_expression" => {
                // For unary expressions like -3, check the argument
                if let Some(argument) = node.child_by_field_name("argument") {
                    self.ends_with_literal_or_identifier(argument, source)
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    fn starts_with_unary_operator(&self, node: Node, source: &str) -> bool {
        if node.kind() == "unary_expression" {
            if let Some(operator) = node.child_by_field_name("operator") {
                let op_text = &source[operator.byte_range()];
                op_text == "-" || op_text == "+" || op_text == "!"
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_simple_literal_or_identifier(&self, node: Node, _source: &str) -> bool {
        matches!(node.kind(), 
            "integer_literal" | "float_literal" | "string_literal" | "identifier" | "boolean_literal"
        )
    }

    fn is_complex_expression(&self, node: Node) -> bool {
        matches!(node.kind(),
            "binary_expression" | "call_expression" | "field_expression" | 
            "method_call_expression" | "macro_invocation"
        )
    }

    fn is_array_element(&self, node: Node) -> bool {
        // Skip punctuation like commas and brackets
        !matches!(node.kind(), "," | "[" | "]")
    }

    fn is_punctuation(&self, node: Node, source: &str) -> bool {
        let text = &source[node.byte_range()];
        matches!(text, "," | "[" | "]" | "(" | ")")
    }
}