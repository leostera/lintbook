use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct Semicolons;

impl Rule for Semicolons {
    fn id(&self) -> &'static str {
        "EX3011"
    }

    fn name(&self) -> &'static str {
        "semicolons"
    }

    fn description(&self) -> &'static str {
        "Don't use semicolons to separate statements"
    }

    fn explanation(&self) -> &'static str {
        "Semicolons should not be used to separate statements in Elixir. Each statement should \
        be on its own line. While Elixir technically allows semicolons, they go against the \
        language's style conventions and make code harder to read. The only acceptable use of \
        semicolons is in IEx (interactive Elixir) for quick one-liners."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        // Collect all string and comment ranges
        let string_ranges = self.collect_string_ranges(tree.root_node());
        
        // Find semicolons not in strings/comments
        for (line_num, line) in source.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ';' {
                    let byte_offset = source.lines().take(line_num).map(|l| l.len() + 1).sum::<usize>() + col;
                    
                    // Check if this semicolon is inside a string/comment range
                    let in_string = string_ranges.iter().any(|(start, end)| {
                        byte_offset >= *start && byte_offset < *end
                    });
                    
                    if !in_string {
                        violations.push(LintViolation {
                            line: line_num + 1,
                            column: col + 1,
                            message: "Remove semicolon and place statements on separate lines".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
        
        violations
    }
}

impl Semicolons {
    fn collect_string_ranges(&self, node: Node) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        self.collect_string_ranges_recursive(node, &mut ranges);
        ranges
    }
    
    fn collect_string_ranges_recursive(&self, node: Node, ranges: &mut Vec<(usize, usize)>) {
        // If this node is a string/comment content, add its range
        if matches!(
            node.kind(),
            "string" | "charlist" | "comment" | "quoted_content" | "sigil"
        ) {
            ranges.push((node.start_byte(), node.end_byte()));
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_string_ranges_recursive(child, ranges);
            }
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
    fn test_detects_semicolons_between_statements() {
        let code = r#"
defmodule Example do
  def bad_style do
    x = 1; y = 2; z = 3
    result = x + y; IO.puts(result)
    z
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Semicolons;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 2);
        assert_eq!(violations[0].lint_id, "EX3011");
        assert!(violations[0].message.contains("Remove semicolon"));
    }

    #[test]
    fn test_detects_semicolon_in_one_liner() {
        let code = r#"
defmodule Example do
  def one_liner, do: x = 1; x + 1
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Semicolons;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
    }

    #[test]
    fn test_allows_clean_code() {
        let code = r#"
defmodule Example do
  def good_style do
    x = 1
    y = 2
    z = 3
    
    result = x + y
    IO.puts(result)
    z
  end
  
  def another_good do
    Enum.map([1, 2, 3], fn x -> 
      x * 2
    end)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Semicolons;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_semicolons_in_strings() {
        let code = r#"
defmodule Example do
  def string_semicolons do
    message = "This has a semicolon; but it's in a string"
    regex = ~r/pattern;/
    charlist = 'also has; semicolon'
    :ok
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Semicolons;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_semicolons_on_line() {
        let code = r#"
defmodule Example do
  def many_semicolons do
    a = 1; b = 2; c = 3; d = 4
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Semicolons;
        let violations = lint.check(&tree, code);

        // Should detect the semicolons
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_semicolon_with_blocks() {
        let code = r#"
defmodule Example do
  def block_semicolon do
    if true do
      x = 1; y = 2
    end
    
    case value do
      :ok -> a = 1; b = 2
      _ -> :error
    end
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = Semicolons;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 2);
    }
}