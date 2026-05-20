use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct UnusedVariableNames;

impl Rule for UnusedVariableNames {
    fn id(&self) -> &'static str {
        "EX1008"
    }

    fn name(&self) -> &'static str {
        "unused_variable_names"
    }

    fn description(&self) -> &'static str {
        "Consistent naming of unused variables (anonymous _ vs meaningful _name)"
    }

    fn explanation(&self) -> &'static str {
        "Use consistent naming for unused variables. Either use a single underscore `_` \
        for all unused variables (anonymous style), or use meaningful names starting \
        with underscore like `_user_id`, `_config` (meaningful style). Mixing styles \
        reduces code clarity and consistency."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        let unused_vars = self.collect_unused_variables(tree.root_node(), source);
        let style_analysis = self.analyze_naming_style(&unused_vars);

        match style_analysis {
            UnusedVariableStyle::Inconsistent {
                anonymous_vars,
                meaningful_vars,
            } => {
                // Report violations for the minority style
                let (minority_vars, minority_style, majority_style) =
                    if anonymous_vars.len() <= meaningful_vars.len() {
                        (
                            anonymous_vars,
                            "anonymous underscore",
                            "meaningful underscore names",
                        )
                    } else {
                        (
                            meaningful_vars,
                            "meaningful underscore names",
                            "anonymous underscore",
                        )
                    };

                for var in minority_vars {
                    violations.push(LintViolation {
                        line: var.line,
                        column: var.column,
                        message: format!(
                            "Inconsistent unused variable naming: '{}' uses {} but file primarily uses {}",
                            var.name, minority_style, majority_style
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
            UnusedVariableStyle::Consistent | UnusedVariableStyle::NoUnusedVariables => {
                // No violations for consistent naming
            }
        }

        violations
    }
}

#[derive(Debug, PartialEq, Clone)]
struct UnusedVariable {
    name: String,
    line: usize,
    column: usize,
    is_anonymous: bool,
}

#[derive(Debug, PartialEq)]
enum UnusedVariableStyle {
    Consistent,
    NoUnusedVariables,
    Inconsistent {
        anonymous_vars: Vec<UnusedVariable>,
        meaningful_vars: Vec<UnusedVariable>,
    },
}

impl UnusedVariableNames {
    fn collect_unused_variables(&self, node: Node, source: &str) -> Vec<UnusedVariable> {
        let mut unused_vars = Vec::new();
        self.traverse_for_unused_variables(node, source, &mut unused_vars);
        unused_vars
    }

    fn traverse_for_unused_variables(
        &self,
        node: Node,
        source: &str,
        unused_vars: &mut Vec<UnusedVariable>,
    ) {
        // Check if this is an identifier that represents an unused variable
        if node.kind() == "identifier" {
            let var_name = &source[node.start_byte()..node.end_byte()];

            if self.is_unused_variable(var_name) {
                let position = node.start_position();
                unused_vars.push(UnusedVariable {
                    name: var_name.to_string(),
                    line: position.row + 1,
                    column: position.column + 1,
                    is_anonymous: var_name == "_",
                });
            }
        }

        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_for_unused_variables(child, source, unused_vars);
            }
        }
    }

    fn is_unused_variable(&self, name: &str) -> bool {
        // Variables starting with underscore are considered unused/ignored
        // This includes both "_" (anonymous) and "_name" (meaningful)
        name.starts_with('_') && self.is_variable_context(name)
    }

    fn is_variable_context(&self, name: &str) -> bool {
        // Simple heuristic: variables are typically lowercase or snake_case
        // This excludes module names, atoms, etc.
        if name == "_" {
            return true;
        }

        // Check if it looks like a variable name (starts with lowercase or underscore)
        name.chars()
            .next()
            .map_or(false, |c| c.is_ascii_lowercase() || c == '_')
    }

    fn analyze_naming_style(&self, unused_vars: &[UnusedVariable]) -> UnusedVariableStyle {
        if unused_vars.is_empty() {
            return UnusedVariableStyle::NoUnusedVariables;
        }

        let anonymous_vars: Vec<_> = unused_vars
            .iter()
            .filter(|var| var.is_anonymous)
            .cloned()
            .collect();

        let meaningful_vars: Vec<_> = unused_vars
            .iter()
            .filter(|var| !var.is_anonymous)
            .cloned()
            .collect();

        // If we only have one style, it's consistent
        if anonymous_vars.is_empty() || meaningful_vars.is_empty() {
            UnusedVariableStyle::Consistent
        } else {
            // We have both styles - inconsistent
            UnusedVariableStyle::Inconsistent {
                anonymous_vars,
                meaningful_vars,
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
    fn test_detects_inconsistent_unused_variable_naming() {
        let code = r#"
defmodule Example do
  def test(_user, data) do
    _ = some_value
    _user_id = fetch_id()
    {_, result} = process()
    _config = load_config()
    result
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnusedVariableNames;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Inconsistent unused variable naming")));
    }

    #[test]
    fn test_allows_consistent_anonymous_style() {
        let code = r#"
defmodule Example do
  def test(_, data) do
    _ = some_value
    {_, result} = process()
    _ = load_config()
    result
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnusedVariableNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_consistent_meaningful_style() {
        let code = r#"
defmodule Example do
  def test(_user, data) do
    _temp_value = some_value
    _user_id = fetch_id()
    {_ignored, result} = process()
    _config = load_config()
    result
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnusedVariableNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_handles_no_unused_variables() {
        let code = r#"
defmodule Example do
  def test(user, data) do
    temp_value = some_value
    user_id = fetch_id()
    {status, result} = process()
    config = load_config()
    {user, data, temp_value, user_id, status, result, config}
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnusedVariableNames;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_reports_minority_style_as_violations() {
        let code = r#"
defmodule Example do
  def test(_user, data) do
    _temp_value = some_value
    _ = fetch_id()
    {_ignored, result} = process()
    _config = load_config()
    result
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnusedVariableNames;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        // Should report the single anonymous underscore since meaningful names are more common
        assert!(violations
            .iter()
            .any(|v| v.message.contains("anonymous underscore")
                && v.message.contains("meaningful underscore names")));
        assert_eq!(violations.len(), 1); // Only the single "_" should be reported
    }

    #[test]
    fn test_handles_mixed_case_with_anonymous_majority() {
        let code = r#"
defmodule Example do
  def test(_, data) do
    _ = some_value
    _ = fetch_id()
    {_, result} = process()
    _single_meaningful = load_config()
    result
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnusedVariableNames;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        // Should report the single meaningful name since anonymous underscores are more common
        assert!(violations
            .iter()
            .any(|v| v.message.contains("_single_meaningful")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("meaningful underscore names")
                && v.message.contains("anonymous underscore")));
    }

    #[test]
    fn test_ignores_non_variable_underscores() {
        let code = r#"
defmodule Example do
  def test(user, data) do
    result = some_function()
    result
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnusedVariableNames;
        let violations = lint.check(&tree, code);

        // Should not flag module names, function names, etc.
        assert_eq!(violations.len(), 0);
    }
}
