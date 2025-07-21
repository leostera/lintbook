use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct SpaceAroundOperators;

impl Rule for SpaceAroundOperators {
    fn id(&self) -> &'static str {
        "EX1005"
    }

    fn name(&self) -> &'static str {
        "space_around_operators"
    }

    fn description(&self) -> &'static str {
        "Consistent spacing around operators (+, -, *, /, etc.)"
    }

    fn explanation(&self) -> &'static str {
        "Use consistent spacing around binary operators to improve readability. \
        Operators like +, -, *, /, ==, !=, <, >, etc. should have spaces on both sides. \
        This makes expressions easier to read and maintains consistency with Elixir \
        style conventions."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        self.traverse_operators(tree.root_node(), source, &mut violations);
        
        violations
    }
}

impl SpaceAroundOperators {
    fn traverse_operators(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Check if this is a binary operator that needs spacing
        if node.kind() == "binary_operator" {
            self.check_operator_spacing(node, source, violations);
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_operators(child, source, violations);
            }
        }
    }
    
    fn check_operator_spacing(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut operator_node = None;
        let mut left_node = None;
        let mut right_node = None;
        
        // Find the operator and its operands
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match i {
                    0 => left_node = Some(child),
                    1 => operator_node = Some(child),
                    2 => right_node = Some(child),
                    _ => {}
                }
            }
        }
        
        if let (Some(left), Some(op), Some(right)) = (left_node, operator_node, right_node) {
            let operator_text = &source[op.start_byte()..op.end_byte()];
            
            // Only check spacing for specific operators that should have spaces
            if self.should_have_spaces(operator_text) {
                // Check space before operator
                let space_before = self.has_space_before(left, op, source);
                let space_after = self.has_space_after(op, right, source);
                
                if !space_before || !space_after {
                    let position = op.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: format!(
                            "Add spaces around '{}' operator{}",
                            operator_text,
                            if !space_before && !space_after {
                                " (missing spaces before and after)"
                            } else if !space_before {
                                " (missing space before)"
                            } else {
                                " (missing space after)"
                            }
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
    
    fn should_have_spaces(&self, operator: &str) -> bool {
        // List of operators that should have spaces around them
        matches!(operator, 
            "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">" | "<=" | ">=" |
            "and" | "or" | "&&" | "||" | "=" | "++" | "--" | "**" | 
            "=~" | "in" | "when" | "|>" | "<>" | "===" | "!==" | 
            "<<<" | ">>>" | "~" | "&&&" | "|||" | "^^^"
        )
    }
    
    fn has_space_before(&self, left_node: Node, operator_node: Node, source: &str) -> bool {
        let left_end = left_node.end_byte();
        let op_start = operator_node.start_byte();
        
        if op_start > left_end {
            let between = &source[left_end..op_start];
            between.chars().any(|c| c.is_whitespace())
        } else {
            false
        }
    }
    
    fn has_space_after(&self, operator_node: Node, right_node: Node, source: &str) -> bool {
        let op_end = operator_node.end_byte();
        let right_start = right_node.start_byte();
        
        if right_start > op_end {
            let between = &source[op_end..right_start];
            between.chars().any(|c| c.is_whitespace())
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_elixir_code(code: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_detects_missing_spaces_around_operators() {
        let code = r#"
defmodule Example do
  def test do
    x = a+b
    y = c*d
    z = e==f
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceAroundOperators;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 3);
        assert_eq!(violations[0].lint_id, "EX1005");
        assert!(violations[0].message.contains("Add spaces around"));
    }

    #[test]
    fn test_allows_proper_spacing() {
        let code = r#"
defmodule Example do
  def test do
    x = a + b
    y = c * d
    z = e == f
    result = (a + b) * (c - d)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceAroundOperators;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_partial_spacing_issues() {
        let code = r#"
defmodule Example do
  def test do
    x = a+ b
    y = c *d
    z = e== f
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceAroundOperators;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 3);
        assert!(violations.iter().any(|v| v.message.contains("missing space before") || v.message.contains("missing space after")));
    }

    #[test]
    fn test_handles_various_operators() {
        let code = r#"
defmodule Example do
  def test do
    a=b
    c!=d
    e<=f
    g>=h
    i&&j
    k||l
    m**n
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceAroundOperators;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 7);
        assert!(violations.iter().all(|v| v.lint_id == "EX1005"));
    }

    #[test]
    fn test_handles_pipe_operators() {
        let code = r#"
defmodule Example do
  def test do
    result = data|>transform()|>process()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceAroundOperators;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.message.contains("|>")));
    }

    #[test]
    fn test_allows_function_calls_without_spaces() {
        let code = r#"
defmodule Example do
  def test do
    result = func(a, b)
    list = [1, 2, 3]
    map = %{key: value}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceAroundOperators;
        let violations = lint.check(&tree, code);

        // Should not flag function calls, lists, or maps
        assert_eq!(violations.len(), 0);
    }
}