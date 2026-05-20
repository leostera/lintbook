use tree_sitter::{Node, Tree, TreeCursor};
use lintbook_core::{LintViolation, Rule};

pub struct FunctionNames;

impl Rule for FunctionNames {
    fn id(&self) -> &'static str {
        "EX5001"
    }

    fn name(&self) -> &'static str {
        "function-names"
    }

    fn description(&self) -> &'static str {
        "Enforce snake_case for function names"
    }

    fn explanation(&self) -> &'static str {
        "Function names should use snake_case naming convention. This is the standard \
        Elixir naming convention that promotes consistency and readability across \
        codebases. Function names should be lowercase with words separated by underscores."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, source, &mut violations);
        violations
    }
}

impl FunctionNames {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for function definitions
        if self.is_function_definition(node) {
            if let Some(function_name) = self.extract_function_name(node, source) {
                if !self.is_snake_case(&function_name) && !self.is_allowed_exception(&function_name)
                {
                    let start_position = node.start_position();
                    violations.push(LintViolation {
                        line: start_position.row + 1,
                        column: start_position.column + 1,
                        message: format!(
                            "Function name '{}' should use snake_case. \
                            Consider renaming to '{}'",
                            function_name,
                            self.to_snake_case(&function_name)
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
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

    fn is_function_definition(&self, node: Node) -> bool {
        // Check for various function definition patterns
        matches!(node.kind(), "call" | "function_definition")
    }

    fn extract_function_name(&self, node: Node, source: &str) -> Option<String> {
        // Handle different function definition patterns
        if node.kind() == "call" {
            // Look for def/defp calls
            if let Some(function_node) = node.child(0) {
                let function_text = &source[function_node.start_byte()..function_node.end_byte()];
                if matches!(function_text, "def" | "defp" | "defmacro" | "defmacrop") {
                    // Get the function name (second child)
                    if let Some(name_node) = node.child(1) {
                        return self.get_function_name_from_node(name_node, source);
                    }
                }
            }
        }

        None
    }

    fn get_function_name_from_node(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "identifier" => {
                let name = &source[node.start_byte()..node.end_byte()];
                Some(name.to_string())
            }
            "call" => {
                // Function with parameters, get the function name
                if let Some(name_node) = node.child(0) {
                    self.get_function_name_from_node(name_node, source)
                } else {
                    None
                }
            }
            "arguments" => {
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        if child.is_named() {
                            return self.get_function_name_from_node(child, source);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn is_snake_case(&self, name: &str) -> bool {
        // Check if the name follows snake_case convention
        if name.is_empty() {
            return false;
        }

        // Must start with lowercase letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() && first_char != '_' {
            return false;
        }

        // Check each character
        for ch in name.chars() {
            if !ch.is_ascii_lowercase()
                && !ch.is_ascii_digit()
                && ch != '_'
                && ch != '?'
                && ch != '!'
            {
                return false;
            }
        }

        // Cannot have consecutive underscores
        !name.contains("__")
    }

    fn is_allowed_exception(&self, name: &str) -> bool {
        // Some exceptions are allowed (operators, special functions)
        matches!(
            name,
            "+" | "-"
                | "*"
                | "/"
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                | "and"
                | "or"
                | "not"
                | "in"
                | "when"
                | "||"
                | "&&"
                | "|>"
                | "..."
                | ".."
                | "^"
                | "&"
                | "|"
                | "~"
                | "%"
                | ":"
                | "::"
                | "unquote"
                | "unquote_splicing"
                | "quote"
                | "var!"
                | "alias!"
                | "require"
                | "import"
                | "use"
                | "defstruct"
                | "defexception"
                | "defprotocol"
                | "defimpl"
                | "defmodule"
                | "defguard"
                | "defguardp"
        )
    }

    fn to_snake_case(&self, name: &str) -> String {
        let mut result = String::new();
        let mut prev_char_was_lowercase = false;

        for (i, ch) in name.chars().enumerate() {
            if ch.is_ascii_uppercase() {
                if i > 0 && prev_char_was_lowercase {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
                prev_char_was_lowercase = false;
            } else {
                result.push(ch);
                prev_char_was_lowercase = ch.is_ascii_lowercase();
            }
        }

        result
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
    fn test_detects_camel_case_functions() {
        let code = r#"
defmodule Example do
  def getUserData(id) do
    # This should be get_user_data
    fetch_user(id)
  end

  def processHTTPRequest(request) do
    # This should be process_http_request
    handle(request)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = FunctionNames;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX5001");
        assert!(violations[0].message.contains("getUserData"));
        assert!(violations[0].message.contains("get_user_data"));
    }

    #[test]
    fn test_allows_snake_case_functions() {
        let code = r#"
defmodule Example do
  def get_user_data(id) do
    fetch_user(id)
  end

  def process_http_request(request) do
    handle(request)
  end

  def valid_function? do
    true
  end

  def dangerous_function! do
    :ok
  end

  defp private_helper do
    :helper
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = FunctionNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_operator_functions() {
        let code = r#"
defmodule Example do
  def +(left, right) do
    left + right
  end

  def ==(left, right) do
    left == right
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = FunctionNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_mixed_naming() {
        let code = r#"
defmodule Example do
  def good_function do
    :ok
  end

  def badFunction do
    :not_ok
  end

  def another_Good_Function do
    :also_not_ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = FunctionNames;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 1);
        assert!(violations.iter().any(|v| v.message.contains("badFunction")));
    }

    #[test]
    fn test_handles_macros() {
        let code = r#"
defmodule Example do
  defmacro badMacroName do
    quote do: :ok
  end

  defmacrop good_private_macro do
    quote do: :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = FunctionNames;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("badMacroName")));
    }
}
