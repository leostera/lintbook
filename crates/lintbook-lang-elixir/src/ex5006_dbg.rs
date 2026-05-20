use tree_sitter::{Node, Tree, TreeCursor};
use lintbook_core::{LintViolation, Rule};

pub struct Dbg;

impl Rule for Dbg {
    fn id(&self) -> &'static str {
        "EX5006"
    }

    fn name(&self) -> &'static str {
        "dbg"
    }

    fn description(&self) -> &'static str {
        "Detect leftover dbg/0,1,2 calls (Elixir 1.14+)"
    }

    fn explanation(&self) -> &'static str {
        "The dbg() function was introduced in Elixir 1.14 as a debugging tool that prints \
        the value of an expression along with its code representation. Like IEx.pry() and \
        IO.inspect(), dbg() calls should not be left in production code as they can expose \
        sensitive data and affect performance. Remove all dbg() calls before committing."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, source, &mut violations);
        violations
    }
}

impl Dbg {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for function call patterns that might be dbg
        if node.kind() == "call" {
            if self.is_dbg_call(node, source) {
                let start_position = node.start_position();
                violations.push(LintViolation {
                    line: start_position.row + 1,
                    column: start_position.column + 1,
                    message: "Remove leftover dbg() call before committing".to_string(),
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

    fn is_dbg_call(&self, node: Node, source: &str) -> bool {
        // Check if this is a call to dbg()
        if node.kind() != "call" {
            return false;
        }

        // Get the function being called
        if let Some(function_node) = node.child(0) {
            match function_node.kind() {
                "identifier" => {
                    let function_text =
                        &source[function_node.start_byte()..function_node.end_byte()];
                    // Check for bare dbg calls
                    return function_text == "dbg";
                }
                "dot" => {
                    // Check for qualified calls like Kernel.dbg
                    let dot_text = &source[function_node.start_byte()..function_node.end_byte()];
                    return dot_text.ends_with(".dbg") || dot_text == "Kernel.dbg";
                }
                _ => {}
            }
        }

        false
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
    fn test_detects_bare_dbg_calls() {
        let code = r#"
defmodule Example do
  def debug_function(value) do
    result = compute(value)
    dbg(result)
    result
  end

  def another_debug do
    dbg()
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Dbg;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].lint_id, "EX5006");
        assert!(violations[0].message.contains("Remove leftover dbg()"));
    }

    #[test]
    fn test_detects_dbg_with_arguments() {
        let code = r#"
defmodule Example do
  def process(data) do
    transformed = transform(data)
    dbg(transformed, limit: 10)

    result = dbg(compute(transformed))
    result
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Dbg;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_detects_kernel_dbg() {
        let code = r#"
defmodule Example do
  def explicit_kernel do
    value = 42
    Kernel.dbg(value)
    value
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Dbg;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "EX5006");
    }

    #[test]
    fn test_ignores_non_dbg_functions() {
        let code = r#"
defmodule Example do
  def debug_info do
    # These are not dbg() calls
    debug_print("info")
    dbg_helper()
    my_dbg = fn x -> IO.inspect(x) end
    my_dbg.(42)
    :ok
  end

  def dbg_helper do
    # This is a custom function, not the built-in dbg
    :custom_function
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Dbg;
        let violations = lint.check(&tree, code);

        // Should not detect dbg_helper or other non-dbg functions
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_dbg_in_pipe() {
        let code = r#"
defmodule Example do
  def pipeline_debug(list) do
    list
    |> Enum.map(&(&1 * 2))
    |> dbg()
    |> Enum.sum()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Dbg;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "EX5006");
    }
}
