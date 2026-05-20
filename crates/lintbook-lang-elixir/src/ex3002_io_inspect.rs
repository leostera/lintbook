use tree_sitter::{Node, Tree, TreeCursor};
use lintbook_core::{LintViolation, Rule};

pub struct IoInspect;

impl Rule for IoInspect {
    fn id(&self) -> &'static str {
        "EX3002"
    }

    fn name(&self) -> &'static str {
        "io-inspect"
    }

    fn description(&self) -> &'static str {
        "Detect leftover IO.inspect/1 calls"
    }

    fn explanation(&self) -> &'static str {
        "IO.inspect/1 calls are debugging tools that should not be left in production code. \
        While they don't stop execution like IEx.pry, they can clutter logs and expose \
        sensitive information in production environments. Use proper logging with Logger \
        instead of IO.inspect for production code."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, source, &mut violations);
        violations
    }
}

impl IoInspect {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for function call patterns that might be IO.inspect
        if node.kind() == "call" {
            if self.is_io_inspect_call(node, source) {
                let start_position = node.start_position();
                violations.push(LintViolation {
                    line: start_position.row + 1,
                    column: start_position.column + 1,
                    message: "Remove leftover IO.inspect/1 call before committing. \
                             Use Logger for production logging."
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

    fn is_io_inspect_call(&self, node: Node, source: &str) -> bool {
        // Check if this is a call to IO.inspect()
        if node.kind() != "call" {
            return false;
        }

        // Get the function being called
        if let Some(function_node) = node.child(0) {
            let function_text = &source[function_node.start_byte()..function_node.end_byte()];

            // Check for direct IO.inspect calls
            if function_text == "IO.inspect" {
                return true;
            }

            // Also check for qualified calls that might be aliased
            if function_node.kind() == "dot" {
                return self.is_qualified_io_inspect(function_node, source);
            }

            // Check for simple inspect calls (if IO is imported)
            if function_text == "inspect" {
                return self.has_io_import(source);
            }
        }

        false
    }

    fn has_io_import(&self, source: &str) -> bool {
        source.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == "import IO" || trimmed.starts_with("import IO,")
        })
    }

    fn is_qualified_io_inspect(&self, dot_node: Node, source: &str) -> bool {
        // Check if this is a qualified call like IO.inspect
        if dot_node.kind() != "dot" {
            return false;
        }

        let dot_text = &source[dot_node.start_byte()..dot_node.end_byte()];
        dot_text == "IO.inspect"
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
    fn test_detects_io_inspect_calls() {
        let code = r#"
defmodule Debug do
  def problematic_function do
    result = calculate_something()
    IO.inspect(result, label: "Debug result")
    process_result(result)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = IoInspect;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX3002");
        assert!(violations[0].message.contains("IO.inspect"));
    }

    #[test]
    fn test_detects_bare_inspect_calls() {
        let code = r#"
defmodule Debug do
  import IO

  def problematic_function do
    result = calculate_something()
    inspect(result)
    process_result(result)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = IoInspect;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX3002");
    }

    #[test]
    fn test_ignores_clean_code() {
        let code = r#"
defmodule Clean do
  require Logger

  def good_function do
    result = calculate_something()
    Logger.info("Processing result: #{inspect(result)}")
    process_result(result)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = IoInspect;
        let violations = lint.check(&tree, code);

        // Logger.info with inspect is fine
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_kernel_inspect() {
        let code = r#"
defmodule Clean do
  def function_with_kernel_inspect do
    # Kernel.inspect is used for string interpolation, not debugging
    "Value: #{inspect(some_value)}"
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = IoInspect;
        let violations = lint.check(&tree, code);

        // Kernel.inspect in string interpolation is fine
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_io_inspect_with_options() {
        let code = r#"
defmodule Debug do
  def debug_with_options do
    data = %{a: 1, b: 2}
    IO.inspect(data, label: "Debug", pretty: true, limit: :infinity)
    process_data(data)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = IoInspect;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX3002");
        assert!(violations[0].message.contains("Logger"));
    }
}
