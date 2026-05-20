use tree_sitter::{Node, Tree, TreeCursor};
use lintbook_core::{LintViolation, Rule};

pub struct VariableNames;

impl Rule for VariableNames {
    fn id(&self) -> &'static str {
        "EX3003"
    }

    fn name(&self) -> &'static str {
        "variable-names"
    }

    fn description(&self) -> &'static str {
        "Enforce snake_case for variable names"
    }

    fn explanation(&self) -> &'static str {
        "Variable names in Elixir should follow the snake_case convention. This means using \
        lowercase letters with underscores separating words. Variables like 'userName' should \
        be 'user_name', and 'HTTPResponse' should be 'http_response'. This convention makes \
        code more readable and consistent with Elixir community standards."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, source, &mut violations);
        violations
    }
}

impl VariableNames {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for variable patterns
        if self.is_variable(node) {
            if let Some(var_name) = self.extract_variable_name(node, source) {
                // Skip special variables like _ and _var
                if !var_name.starts_with('_') && !self.is_snake_case(&var_name) {
                    let start_position = node.start_position();
                    violations.push(LintViolation {
                        line: start_position.row + 1,
                        column: start_position.column + 1,
                        message: format!(
                            "Variable name '{}' should use snake_case. \
                            Consider renaming to '{}'",
                            var_name,
                            self.to_snake_case(&var_name)
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

    fn is_variable(&self, node: Node) -> bool {
        // Check for various variable patterns
        matches!(node.kind(), "identifier" | "variable") && !self.is_function_or_module_name(node)
    }

    fn is_function_or_module_name(&self, node: Node) -> bool {
        // Check if this identifier is actually a function or module name
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "call" => {
                    // Check if it's the function being called
                    if let Some(first_child) = parent.child(0) {
                        return first_child == node;
                    }
                }
                "module" | "alias" => return true,
                _ => {}
            }
        }
        false
    }

    fn extract_variable_name(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "identifier" | "variable" => {
                let name = &source[node.start_byte()..node.end_byte()];
                Some(name.to_string())
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

        // Check the rest of the characters
        let mut prev_underscore = false;
        for ch in name.chars() {
            match ch {
                'a'..='z' | '0'..='9' => prev_underscore = false,
                '_' => {
                    if prev_underscore {
                        return false; // No double underscores
                    }
                    prev_underscore = true;
                }
                '?' | '!' => {
                    // These are allowed at the end for predicates and dangerous functions
                    if name.chars().last() != Some(ch) {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        // No trailing underscores (unless it's just "_")
        !prev_underscore || name == "_"
    }

    fn to_snake_case(&self, name: &str) -> String {
        let mut result = String::new();
        let mut prev_lowercase = false;

        for (i, ch) in name.chars().enumerate() {
            if ch.is_ascii_uppercase() {
                if i > 0 && prev_lowercase {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
                prev_lowercase = false;
            } else {
                result.push(ch);
                prev_lowercase = ch.is_ascii_lowercase();
            }
        }

        // Handle sequences like "HTTPResponse" -> "http_response"
        result = result.replace("__", "_");

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
    fn test_detects_camel_case_variables() {
        let code = r#"
defmodule Example do
  def process_data(userId, userName) do
    responseData = fetch_data(userId)
    HTTPResponse = format_response(responseData)
    {userName, HTTPResponse}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = VariableNames;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 3);
        assert_eq!(violations[0].lint_id, "EX3003");

        // Check that it detects userId, userName, responseData, HTTPResponse
        let messages: Vec<_> = violations.iter().map(|v| &v.message).collect();
        assert!(messages.iter().any(|m| m.contains("userId")));
        assert!(messages.iter().any(|m| m.contains("responseData")));
    }

    #[test]
    fn test_allows_snake_case_variables() {
        let code = r#"
defmodule Example do
  def process_data(user_id, user_name) do
    response_data = fetch_data(user_id)
    http_response = format_response(response_data)
    {user_name, http_response}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = VariableNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_underscore_prefixed_variables() {
        let code = r#"
defmodule Example do
  def unused_params(_userId, _userName) do
    _unusedVar = compute_something()
    _ = side_effect()
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = VariableNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_snake_case_conversion() {
        let lint = VariableNames;

        assert_eq!(lint.to_snake_case("userId"), "user_id");
        assert_eq!(lint.to_snake_case("userName"), "user_name");
        assert_eq!(lint.to_snake_case("HTTPResponse"), "httpresponse");
        assert_eq!(lint.to_snake_case("responseHTTPData"), "response_httpdata");
        assert_eq!(lint.to_snake_case("camelCase"), "camel_case");
        assert_eq!(lint.to_snake_case("PascalCase"), "pascal_case");
    }
}
