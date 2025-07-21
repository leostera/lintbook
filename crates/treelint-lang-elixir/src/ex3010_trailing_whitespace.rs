use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct TrailingWhitespace;

impl Rule for TrailingWhitespace {
    fn id(&self) -> &'static str {
        "EX3010"
    }

    fn name(&self) -> &'static str {
        "trailing-whitespace"
    }

    fn description(&self) -> &'static str {
        "No trailing whitespace at end of lines"
    }

    fn explanation(&self) -> &'static str {
        "Trailing whitespace at the end of lines serves no purpose and can cause issues with \
        version control systems by creating unnecessary diffs. It can also lead to inconsistent \
        behavior in some editors and tools. Remove all trailing spaces and tabs at the end of lines."
    }

    fn check(&self, _tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            if self.has_trailing_whitespace(line) {
                let line_len = line.len();
                let trimmed_len = line.trim_end().len();
                let whitespace_start = trimmed_len;
                
                violations.push(LintViolation {
                    line: line_num + 1,
                    column: whitespace_start + 1,
                    message: format!(
                        "Line has {} trailing whitespace character(s)",
                        line_len - trimmed_len
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        violations
    }
}

impl TrailingWhitespace {
    fn has_trailing_whitespace(&self, line: &str) -> bool {
        // Check if line ends with whitespace
        line.len() > 0 && line.chars().last().map_or(false, |c| c.is_whitespace())
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
    fn test_detects_trailing_spaces() {
        // Note: The spaces after 'do' and 'end' are intentional
        let code = "defmodule Example do  \n  def hello do   \n    :world\n  end \nend\n";

        let tree = parse_elixir_code(code);
        let lint = TrailingWhitespace;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].lint_id, "EX3010");
        assert_eq!(violations[0].line, 1); // Line with "do  "
        assert_eq!(violations[1].line, 2); // Line with "do   "
        assert_eq!(violations[2].line, 4); // Line with "end "
    }

    #[test]
    fn test_detects_trailing_tabs() {
        let code = "defmodule Example do\t\n  def hello do\n    :world\t\t\n  end\nend\n";

        let tree = parse_elixir_code(code);
        let lint = TrailingWhitespace;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1); // Tab after "do"
        assert_eq!(violations[1].line, 3); // Tabs after ":world"
    }

    #[test]
    fn test_allows_clean_lines() {
        let code = r#"defmodule Example do
  def hello do
    :world
  end

  def goodbye do
    :farewell
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = TrailingWhitespace;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mixed_trailing_whitespace() {
        // Mix of spaces and tabs at end of lines
        let code = "def test do \t\n  value = 42\t \n  value\nend   \t  \n";

        let tree = parse_elixir_code(code);
        let lint = TrailingWhitespace;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("2 trailing whitespace"));
        assert!(violations[1].message.contains("2 trailing whitespace"));
        assert!(violations[2].message.contains("6 trailing whitespace"));
    }

    #[test]
    fn test_empty_lines_with_whitespace() {
        let code = "defmodule Test do\n  \n    def test, do: :ok\n\t\t\nend\n";

        let tree = parse_elixir_code(code);
        let lint = TrailingWhitespace;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2); // Line with only spaces
        assert_eq!(violations[1].line, 4); // Line with only tabs
    }
}