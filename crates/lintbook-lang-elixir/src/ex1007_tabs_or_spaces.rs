use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct TabsOrSpaces;

impl Rule for TabsOrSpaces {
    fn id(&self) -> &'static str {
        "EX1007"
    }

    fn name(&self) -> &'static str {
        "tabs_or_spaces"
    }

    fn description(&self) -> &'static str {
        "Consistent use of tabs or spaces for indentation"
    }

    fn explanation(&self) -> &'static str {
        "Use either tabs or spaces for indentation consistently throughout your code. \
        Mixing tabs and spaces for indentation reduces readability and can cause \
        issues with different editors and environments. Choose one style and \
        stick with it across the entire file."
    }

    fn check(&self, _tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        let indentation_analysis = self.analyze_indentation(source);

        match indentation_analysis {
            IndentationStyle::Mixed { mixed_lines } => {
                for &line_num in &mixed_lines {
                    violations.push(LintViolation {
                        line: line_num,
                        column: 1,
                        message:
                            "Mixed indentation: line uses both tabs and spaces for indentation"
                                .to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
            IndentationStyle::Inconsistent {
                tab_lines,
                space_lines,
            } => {
                // Report violations based on the less common style
                let (minority_lines, minority_style) = if tab_lines.len() <= space_lines.len() {
                    (tab_lines, "tabs")
                } else {
                    (space_lines, "spaces")
                };

                for &line_num in &minority_lines {
                    violations.push(LintViolation {
                        line: line_num,
                        column: 1,
                        message: format!(
                            "Inconsistent indentation: file primarily uses {} but this line uses {}",
                            if minority_style == "tabs" { "spaces" } else { "tabs" },
                            minority_style
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
            IndentationStyle::Consistent | IndentationStyle::NoIndentation => {
                // No violations for consistent indentation
            }
        }

        violations
    }
}

#[derive(Debug, PartialEq)]
enum IndentationStyle {
    Consistent,
    NoIndentation,
    Mixed {
        mixed_lines: Vec<usize>,
    },
    Inconsistent {
        tab_lines: Vec<usize>,
        space_lines: Vec<usize>,
    },
}

#[derive(Debug, PartialEq)]
enum LineIndentationType {
    None,
    Tabs,
    Spaces,
    Mixed,
}

impl TabsOrSpaces {
    fn analyze_indentation(&self, source: &str) -> IndentationStyle {
        let mut tab_lines = Vec::new();
        let mut space_lines = Vec::new();
        let mut mixed_lines = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            let line_number = line_num + 1;

            match self.analyze_line_indentation(line) {
                LineIndentationType::Tabs => tab_lines.push(line_number),
                LineIndentationType::Spaces => space_lines.push(line_number),
                LineIndentationType::Mixed => mixed_lines.push(line_number),
                LineIndentationType::None => {
                    // Empty lines or lines with no indentation don't affect the style
                }
            }
        }

        // Check for mixed indentation first (highest priority)
        if !mixed_lines.is_empty() {
            return IndentationStyle::Mixed { mixed_lines };
        }

        // Check if we have both tabs and spaces
        if !tab_lines.is_empty() && !space_lines.is_empty() {
            return IndentationStyle::Inconsistent {
                tab_lines,
                space_lines,
            };
        }

        // If we only have one type or no indentation at all
        if tab_lines.is_empty() && space_lines.is_empty() {
            IndentationStyle::NoIndentation
        } else {
            IndentationStyle::Consistent
        }
    }

    fn analyze_line_indentation(&self, line: &str) -> LineIndentationType {
        if line.trim().is_empty() {
            return LineIndentationType::None;
        }

        let mut has_tabs = false;
        let mut has_spaces = false;

        // Analyze the leading whitespace
        for ch in line.chars() {
            match ch {
                '\t' => has_tabs = true,
                ' ' => has_spaces = true,
                _ => break, // Stop at first non-whitespace character
            }
        }

        // Determine the type based on what we found
        match (has_tabs, has_spaces) {
            (true, true) => LineIndentationType::Mixed,
            (true, false) => LineIndentationType::Tabs,
            (false, true) => LineIndentationType::Spaces,
            (false, false) => LineIndentationType::None,
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
    fn test_detects_mixed_indentation_within_line() {
        let code = "defmodule Example do\n\t  def test do\n    x = 1\n  end\nend";

        let tree = parse_elixir_code(code);
        let lint = TabsOrSpaces;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Mixed indentation")));
        assert_eq!(violations[0].line, 2); // The line with mixed tabs and spaces
    }

    #[test]
    fn test_detects_inconsistent_indentation_across_lines() {
        let code = "defmodule Example do\n  def with_spaces do\n    x = 1\n  end\n\n\tdef with_tabs do\n\t\tx = 1\n\tend\nend";

        let tree = parse_elixir_code(code);
        let lint = TabsOrSpaces;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Inconsistent indentation")));
    }

    #[test]
    fn test_allows_consistent_space_indentation() {
        let code = r#"
defmodule Example do
  def test do
    x = 1
    y = 2
    if x > 0 do
      IO.puts("positive")
    end
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = TabsOrSpaces;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_consistent_tab_indentation() {
        let code = "defmodule Example do\n\tdef test do\n\t\tx = 1\n\t\ty = 2\n\t\tif x > 0 do\n\t\t\tIO.puts(\"positive\")\n\t\tend\n\tend\nend";

        let tree = parse_elixir_code(code);
        let lint = TabsOrSpaces;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_empty_lines() {
        let code = r#"
defmodule Example do
  def test do

    x = 1

    y = 2
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = TabsOrSpaces;
        let violations = lint.check(&tree, code);

        // Empty lines should not affect consistency check
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_handles_no_indentation() {
        let code = "defmodule Example do\ndef test do\nx = 1\ny = 2\nend\nend";

        let tree = parse_elixir_code(code);
        let lint = TabsOrSpaces;
        let violations = lint.check(&tree, code);

        // No indentation is consistent (though not good style)
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_reports_minority_style_as_violations() {
        let code = "defmodule Example do\n  def test1 do\n    x = 1\n  end\n\n\tdef test2 do\n\t\ty = 2\n\tend\n\n  def test3 do\n    z = 3\n  end\nend";

        let tree = parse_elixir_code(code);
        let lint = TabsOrSpaces;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        // Should report the lines with tabs since spaces are more common
        assert!(violations
            .iter()
            .any(|v| v.line == 6 && v.message.contains("this line uses tabs")));
        assert!(violations
            .iter()
            .any(|v| v.line == 7 && v.message.contains("this line uses tabs")));
        assert!(violations
            .iter()
            .any(|v| v.line == 8 && v.message.contains("this line uses tabs")));
    }
}
