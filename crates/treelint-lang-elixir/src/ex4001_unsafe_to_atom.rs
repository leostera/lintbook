use tree_sitter::{Node, Tree, TreeCursor};
use treelint_core::{LintViolation, Rule};

pub struct UnsafeToAtom;

impl Rule for UnsafeToAtom {
    fn id(&self) -> &'static str {
        "EX4001"
    }

    fn name(&self) -> &'static str {
        "unsafe-to-atom"
    }

    fn description(&self) -> &'static str {
        "Prevent creating atoms dynamically from external sources"
    }

    fn explanation(&self) -> &'static str {
        "Creating atoms dynamically from untrusted external sources can lead to atom table \
        exhaustion attacks. The Erlang VM has a limited atom table and once it's full, \
        the system crashes. Use String.to_existing_atom/1 instead of String.to_atom/1 \
        when converting strings from external sources, or use strings directly instead \
        of atoms."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, source, &mut violations);
        violations
    }
}

impl UnsafeToAtom {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for function call patterns that create atoms unsafely
        if node.kind() == "call" {
            if self.is_unsafe_to_atom_call(node, source) {
                let start_position = node.start_position();
                violations.push(LintViolation {
                    line: start_position.row + 1,
                    column: start_position.column + 1,
                    message: "Use String.to_existing_atom/1 instead of String.to_atom/1 \
                             to prevent atom table exhaustion attacks"
                        .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            } else if self.is_list_to_atom_call(node, source) {
                let start_position = node.start_position();
                violations.push(LintViolation {
                    line: start_position.row + 1,
                    column: start_position.column + 1,
                    message: "Use List.to_existing_atom/1 instead of List.to_atom/1 \
                             to prevent atom table exhaustion attacks"
                        .to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Recursively check children
        if cursor.goto_first_child() {
            loop {
                self.traverse_node(cursor.node(), cursor, source, violations);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    fn is_unsafe_to_atom_call(&self, node: Node, source: &str) -> bool {
        if node.kind() != "call" {
            return false;
        }

        // Get the function being called
        if let Some(function_node) = node.child(0) {
            let function_text = &source[function_node.start_byte()..function_node.end_byte()];

            // Check for String.to_atom calls
            if function_text == "String.to_atom" {
                return true;
            }

            // Check for qualified calls
            if function_node.kind() == "dot" {
                return self.is_string_to_atom_qualified(function_node, source);
            }

            // Check for imported function calls
            if function_text == "to_atom" {
                // This could be dangerous if String is imported
                return true;
            }
        }

        false
    }

    fn is_list_to_atom_call(&self, node: Node, source: &str) -> bool {
        if node.kind() != "call" {
            return false;
        }

        if let Some(function_node) = node.child(0) {
            let function_text = &source[function_node.start_byte()..function_node.end_byte()];

            // Check for List.to_atom calls
            if function_text == "List.to_atom" {
                return true;
            }

            // Check for qualified calls
            if function_node.kind() == "dot" {
                return self.is_list_to_atom_qualified(function_node, source);
            }
        }

        false
    }

    fn is_string_to_atom_qualified(&self, dot_node: Node, source: &str) -> bool {
        if dot_node.kind() != "dot" {
            return false;
        }

        let dot_text = &source[dot_node.start_byte()..dot_node.end_byte()];
        dot_text == "String.to_atom"
    }

    fn is_list_to_atom_qualified(&self, dot_node: Node, source: &str) -> bool {
        if dot_node.kind() != "dot" {
            return false;
        }

        let dot_text = &source[dot_node.start_byte()..dot_node.end_byte()];
        dot_text == "List.to_atom"
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
    fn test_detects_string_to_atom() {
        let code = r#"
defmodule Unsafe do
  def convert_user_input(input) do
    # This is dangerous - user input can create unlimited atoms
    atom = String.to_atom(input)
    process_atom(atom)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeToAtom;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX4001");
        assert!(violations[0].message.contains("String.to_existing_atom"));
    }

    #[test]
    fn test_detects_list_to_atom() {
        let code = r#"
defmodule Unsafe do
  def convert_char_list(char_list) do
    # This is also dangerous
    atom = List.to_atom(char_list)
    process_atom(atom)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeToAtom;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX4001");
        assert!(violations[0].message.contains("List.to_existing_atom"));
    }

    #[test]
    fn test_allows_safe_alternatives() {
        let code = r#"
defmodule Safe do
  def convert_safely(input) do
    # These are safe alternatives
    case String.to_existing_atom(input) do
      {:ok, atom} -> process_atom(atom)
      {:error, _} -> :invalid_atom
    end
  end
  
  def use_string_directly(input) do
    # Using strings directly is also safe
    process_string(input)
  end
  
  def use_predefined_atom do
    # Hardcoded atoms are fine
    :my_predefined_atom
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeToAtom;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_imported_to_atom() {
        let code = r#"
defmodule WithImport do
  import String, only: [to_atom: 1]
  
  def dangerous_function(input) do
    # Even when imported, this is dangerous
    atom = to_atom(input)
    process_atom(atom)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeToAtom;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX4001");
    }
}
