use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct SpaceInParentheses;

impl Rule for SpaceInParentheses {
    fn id(&self) -> &'static str {
        "EX1006"
    }

    fn name(&self) -> &'static str {
        "space_in_parentheses"
    }

    fn description(&self) -> &'static str {
        "Consistent spacing inside parentheses, brackets, and braces"
    }

    fn explanation(&self) -> &'static str {
        "Be consistent with spacing inside parentheses, brackets, and braces. \
        Either always use spaces: `func( a, b )`, `[ 1, 2, 3 ]`, `%{ key: value }` \
        or never use spaces: `func(a, b)`, `[1, 2, 3]`, `%{key: value}`. \
        Mixing styles reduces readability."
    }

    fn check(&self, _tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        // Scan through the source character by character to find parentheses, brackets, and braces
        self.scan_spacing_issues(source, &mut violations);

        violations
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BracketType {
    Parentheses,
    SquareBrackets,
    CurlyBraces,
}

#[derive(Debug, Clone)]
struct BracketPair {
    bracket_type: BracketType,
    open_pos: usize,
    line: usize,
    has_space_after_open: bool,
    has_space_before_close: bool,
}

impl SpaceInParentheses {
    fn scan_spacing_issues(&self, source: &str, violations: &mut Vec<LintViolation>) {
        let bracket_pairs = self.find_bracket_pairs(source);

        if bracket_pairs.is_empty() {
            return;
        }

        // Analyze consistent spacing within each bracket type
        self.check_consistency(&bracket_pairs, violations);
    }

    fn find_bracket_pairs(&self, source: &str) -> Vec<BracketPair> {
        let mut pairs = Vec::new();
        let mut stack = Vec::new();
        let mut in_string = false;
        let mut in_charlist = false;
        let mut escape_next = false;

        let chars: Vec<char> = source.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => escape_next = true,
                '"' if !in_charlist => in_string = !in_string,
                '\'' if !in_string => in_charlist = !in_charlist,
                '(' | '[' | '{' if !in_string && !in_charlist => {
                    let bracket_type = match ch {
                        '(' => BracketType::Parentheses,
                        '[' => BracketType::SquareBrackets,
                        '{' => BracketType::CurlyBraces,
                        _ => unreachable!(),
                    };
                    stack.push((bracket_type, i));
                }
                ')' | ']' | '}' if !in_string && !in_charlist => {
                    let expected_type = match ch {
                        ')' => BracketType::Parentheses,
                        ']' => BracketType::SquareBrackets,
                        '}' => BracketType::CurlyBraces,
                        _ => unreachable!(),
                    };

                    if let Some((bracket_type, open_pos)) = stack.pop() {
                        if bracket_type == expected_type {
                            let line = source[..open_pos].chars().filter(|&c| c == '\n').count();

                            // Check spacing after opening bracket
                            let has_space_after_open = i > open_pos + 1
                                && chars.get(open_pos + 1).map_or(false, |c| c.is_whitespace());

                            // Check spacing before closing bracket
                            let has_space_before_close =
                                i > 0 && chars.get(i - 1).map_or(false, |c| c.is_whitespace());

                            // Only consider non-empty brackets
                            if i > open_pos + 1 {
                                pairs.push(BracketPair {
                                    bracket_type,
                                    open_pos,
                                    line: line + 1,
                                    has_space_after_open,
                                    has_space_before_close,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        pairs
    }

    fn check_consistency(&self, pairs: &[BracketPair], violations: &mut Vec<LintViolation>) {
        // Group by bracket type
        for bracket_type in [
            BracketType::Parentheses,
            BracketType::SquareBrackets,
            BracketType::CurlyBraces,
        ] {
            let type_pairs: Vec<_> = pairs
                .iter()
                .filter(|p| p.bracket_type == bracket_type)
                .collect();

            if type_pairs.len() < 2 {
                continue; // Need at least 2 to check consistency
            }

            // Determine the predominant style
            let with_spaces = type_pairs
                .iter()
                .filter(|p| p.has_space_after_open && p.has_space_before_close)
                .count();
            let without_spaces = type_pairs
                .iter()
                .filter(|p| !p.has_space_after_open && !p.has_space_before_close)
                .count();

            if with_spaces == 0 && without_spaces == 0 {
                continue; // No clear pattern
            }

            let prefer_spaces = with_spaces > without_spaces;
            let bracket_name = match bracket_type {
                BracketType::Parentheses => "parentheses",
                BracketType::SquareBrackets => "square brackets",
                BracketType::CurlyBraces => "curly braces",
            };

            // Report violations for inconsistent spacing
            for pair in &type_pairs {
                let current_has_spaces = pair.has_space_after_open && pair.has_space_before_close;
                let current_mixed = pair.has_space_after_open != pair.has_space_before_close;

                if current_mixed || (current_has_spaces != prefer_spaces) {
                    let expected_style = if prefer_spaces { "with" } else { "without" };
                    violations.push(LintViolation {
                        line: pair.line,
                        column: pair.open_pos + 1,
                        message: format!(
                            "Inconsistent spacing in {}. Use {} spaces consistently{}",
                            bracket_name,
                            expected_style,
                            if current_mixed {
                                " (currently mixed)"
                            } else {
                                ""
                            }
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
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
    fn test_detects_inconsistent_parentheses_spacing() {
        let code = r#"
defmodule Example do
  def test do
    func( a, b )
    other(c, d)
    third( e, f)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceInParentheses;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Inconsistent spacing in parentheses")));
    }

    #[test]
    fn test_allows_consistent_spacing_with_spaces() {
        let code = r#"
defmodule Example do
  def test do
    func( a, b )
    other( c, d )
    third( e, f )
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceInParentheses;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_consistent_spacing_without_spaces() {
        let code = r#"
defmodule Example do
  def test do
    func(a, b)
    other(c, d)
    third(e, f)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceInParentheses;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_handles_square_brackets() {
        let code = r#"
defmodule Example do
  def test do
    list = [ 1, 2, 3 ]
    other = [4, 5, 6]
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceInParentheses;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("square brackets")));
    }

    #[test]
    fn test_handles_curly_braces() {
        let code = r#"
defmodule Example do
  def test do
    map1 = %{ key: value }
    map2 = %{other: data}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceInParentheses;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("curly braces")));
    }

    #[test]
    fn test_ignores_strings_and_charlists() {
        let code = r#"
defmodule Example do
  def test do
    string = "func( a, b )"
    charlist = 'list[ 1, 2, 3 ]'
    func(a, b)
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceInParentheses;
        let violations = lint.check(&tree, code);

        // Should not flag brackets inside strings
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_handles_empty_brackets() {
        let code = r#"
defmodule Example do
  def test do
    func()
    list = []
    map = %{}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = SpaceInParentheses;
        let violations = lint.check(&tree, code);

        // Empty brackets should not be considered for spacing
        assert_eq!(violations.len(), 0);
    }
}
