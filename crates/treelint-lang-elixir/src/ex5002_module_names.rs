use tree_sitter::{Node, Tree, TreeCursor};
use treelint_core::{LintViolation, Rule};

pub struct ModuleNames;

impl Rule for ModuleNames {
    fn id(&self) -> &'static str {
        "EX5002"
    }

    fn name(&self) -> &'static str {
        "module-names"
    }

    fn description(&self) -> &'static str {
        "Enforce PascalCase for module names"
    }

    fn explanation(&self) -> &'static str {
        "Module names should use PascalCase naming convention. This is the standard \
        Elixir naming convention that promotes consistency and readability across \
        codebases. Module names should start with an uppercase letter and use \
        PascalCase for multi-word names."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, source, &mut violations);
        violations
    }
}

impl ModuleNames {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for defmodule calls
        if node.kind() == "call" {
            if let Some(function_node) = node.child(0) {
                let function_text = &source[function_node.start_byte()..function_node.end_byte()];
                if function_text == "defmodule" {
                    if let Some(module_name_node) = node.child(1) {
                        self.check_module_name(module_name_node, source, violations);
                    }
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

    fn check_module_name(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if let Some(module_name) = self.extract_module_name(node, source) {
            let components = module_name.split('.').collect::<Vec<&str>>();

            for component in components {
                if !component.is_empty() && !self.is_pascal_case(component) {
                    let start_position = node.start_position();
                    violations.push(LintViolation {
                        line: start_position.row + 1,
                        column: start_position.column + 1,
                        message: format!(
                            "Module name component '{}' should use PascalCase. \
                            Consider renaming to '{}'",
                            component,
                            self.to_pascal_case(component)
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }

    fn extract_module_name(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "alias" => {
                // Handle aliased module names like MyApp.User
                Some(source[node.start_byte()..node.end_byte()].to_string())
            }
            "identifier" => {
                // Handle simple module names
                Some(source[node.start_byte()..node.end_byte()].to_string())
            }
            "dot" => {
                // Handle dotted module names
                Some(source[node.start_byte()..node.end_byte()].to_string())
            }
            _ => None,
        }
    }

    fn is_pascal_case(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must start with uppercase letter
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_uppercase() {
            return false;
        }

        // Check each character - only letters and digits allowed
        for ch in name.chars() {
            if !ch.is_ascii_alphanumeric() {
                return false;
            }
        }

        true
    }

    fn to_pascal_case(&self, name: &str) -> String {
        if name.is_empty() {
            return name.to_string();
        }

        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in name.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch.to_ascii_lowercase());
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
    fn test_detects_snake_case_modules() {
        let code = r#"
defmodule my_module do
  def function do
    :ok
  end
end

defmodule another_bad_module do
  def function do
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ModuleNames;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX5002");
        assert!(violations[0].message.contains("my_module"));
        assert!(violations[0].message.contains("MyModule"));
    }

    #[test]
    fn test_allows_pascal_case_modules() {
        let code = r#"
defmodule MyModule do
  def function do
    :ok
  end
end

defmodule MyApp.UserController do
  def index do
    :ok
  end
end

defmodule HTTPClient do
  def get(url) do
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ModuleNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_lowercase_start() {
        let code = r#"
defmodule myModule do
  def function do
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ModuleNames;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX5002");
        assert!(violations[0].message.contains("myModule"));
        assert!(violations[0].message.contains("MyModule"));
    }

    #[test]
    fn test_detects_nested_module_violations() {
        let code = r#"
defmodule MyApp.bad_controller do
  def index do
    :ok
  end
end

defmodule my_app.UserController do
  def show do
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ModuleNames;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 1);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("bad_controller")));
    }

    #[test]
    fn test_handles_acronyms() {
        let code = r#"
defmodule HTTPSClient do
  def get(url) do
    :ok
  end
end

defmodule XMLParser do
  def parse(xml) do
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ModuleNames;
        let violations = lint.check(&tree, code);

        // Acronyms in PascalCase are allowed
        assert_eq!(violations.len(), 0);
    }
}
