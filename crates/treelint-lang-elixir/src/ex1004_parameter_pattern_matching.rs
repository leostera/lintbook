use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct ParameterPatternMatching;

impl Rule for ParameterPatternMatching {
    fn id(&self) -> &'static str {
        "EX1004"
    }

    fn name(&self) -> &'static str {
        "parameter_pattern_matching"
    }

    fn description(&self) -> &'static str {
        "Consistent variable placement in pattern matching (before/after)"
    }

    fn explanation(&self) -> &'static str {
        "Be consistent with variable naming in pattern matching. In map/struct patterns, \
        prefer either key-matching style `%{name: name}` where the variable matches the key, \
        or explicit naming style `%{name: user_name}` where variables have descriptive names. \
        Mixing both styles in the same function or module reduces readability."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        // Find all function definitions and analyze their parameter patterns
        self.traverse_functions(tree.root_node(), source, &mut violations);
        
        violations
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PatternStyle {
    KeyMatching,    // %{name: name}
    ExplicitNaming, // %{name: user_name}
}

#[derive(Debug, Clone)]
struct PatternInfo {
    style: PatternStyle,
    line: usize,
    column: usize,
    key: String,
}

impl ParameterPatternMatching {
    fn traverse_functions(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Check if this is a function definition
        if self.is_function_definition(node, source) {
            if let Some(args_node) = self.find_function_arguments(node) {
                self.check_function_patterns(args_node, source, violations);
            }
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_functions(child, source, violations);
            }
        }
    }
    
    fn is_function_definition(&self, node: Node, source: &str) -> bool {
        node.kind() == "call" && 
        node.child(0).map_or(false, |child| 
            child.kind() == "identifier" && {
                let text = &source[child.start_byte()..child.end_byte()];
                matches!(text, "def" | "defp" | "defmacro" | "defmacrop")
            }
        )
    }
    
    fn find_function_arguments<'a>(&self, func_node: Node<'a>) -> Option<Node<'a>> {
        // Look for arguments in function definition
        // Structure: call -> arguments -> call -> arguments (function parameters)
        if let Some(args) = func_node.child(1) {
            if args.kind() == "arguments" {
                if let Some(func_call) = args.child(0) {
                    if func_call.kind() == "call" {
                        return func_call.child(1); // The function's arguments
                    }
                }
            }
        }
        None
    }
    
    fn check_function_patterns(&self, args_node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let patterns = self.collect_patterns_from_arguments(args_node, source);
        
        if patterns.len() < 2 {
            return; // Need at least 2 patterns to check consistency
        }
        
        // Group patterns by style
        let key_matching: Vec<_> = patterns.iter().filter(|p| p.style == PatternStyle::KeyMatching).collect();
        let explicit_naming: Vec<_> = patterns.iter().filter(|p| p.style == PatternStyle::ExplicitNaming).collect();
        
        // If we have both styles in the same function, report violations
        if !key_matching.is_empty() && !explicit_naming.is_empty() {
            // Report violations for the minority style
            let violations_to_report = if key_matching.len() <= explicit_naming.len() {
                &key_matching
            } else {
                &explicit_naming
            };
            
            let preferred_style = if key_matching.len() > explicit_naming.len() {
                "key-matching"
            } else {
                "explicit naming"
            };
            
            for pattern in violations_to_report {
                violations.push(LintViolation {
                    line: pattern.line,
                    column: pattern.column,
                    message: format!(
                        "Inconsistent pattern style. Use {} style like other patterns in this function ({})",
                        preferred_style,
                        if preferred_style == "key-matching" {
                            format!("%{{{}: {}}}", pattern.key, pattern.key)
                        } else {
                            format!("%{{{}: {}_value}}", pattern.key, pattern.key)
                        }
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
    
    fn collect_patterns_from_arguments(&self, args_node: Node, source: &str) -> Vec<PatternInfo> {
        let mut patterns = Vec::new();
        
        // Traverse all argument nodes looking for map patterns
        self.collect_map_patterns(args_node, source, &mut patterns);
        
        patterns
    }
    
    fn collect_map_patterns(&self, node: Node, source: &str, patterns: &mut Vec<PatternInfo>) {
        if node.kind() == "map" {
            // This is a map pattern, analyze it
            if let Some(pattern_info) = self.analyze_map_pattern(node, source) {
                patterns.push(pattern_info);
            }
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_map_patterns(child, source, patterns);
            }
        }
    }
    
    fn analyze_map_pattern(&self, map_node: Node, source: &str) -> Option<PatternInfo> {
        // Look for map_content -> keywords -> pair pattern
        for i in 0..map_node.child_count() {
            if let Some(map_content) = map_node.child(i) {
                if map_content.kind() == "map_content" {
                    for j in 0..map_content.child_count() {
                        if let Some(keywords) = map_content.child(j) {
                            if keywords.kind() == "keywords" {
                                // Look at the first pair to determine style
                                for k in 0..keywords.child_count() {
                                    if let Some(pair) = keywords.child(k) {
                                        if pair.kind() == "pair" {
                                            return self.analyze_pair_pattern(pair, source);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    fn analyze_pair_pattern(&self, pair_node: Node, source: &str) -> Option<PatternInfo> {
        let mut key_text = None;
        let mut value_text = None;
        
        // Extract key and value from the pair
        for i in 0..pair_node.child_count() {
            if let Some(child) = pair_node.child(i) {
                if i == 0 && child.kind() == "keyword" {
                    let text = source[child.start_byte()..child.end_byte()].to_string();
                    // Remove the trailing ": " from keyword
                    key_text = Some(text.trim_end_matches(": ").to_string());
                } else if i == 1 && child.kind() == "identifier" {
                    value_text = Some(source[child.start_byte()..child.end_byte()].to_string());
                }
            }
        }
        
        if let (Some(key), Some(value)) = (key_text, value_text) {
            let style = if key == value {
                PatternStyle::KeyMatching
            } else {
                PatternStyle::ExplicitNaming
            };
            
            let position = pair_node.start_position();
            return Some(PatternInfo {
                style,
                line: position.row + 1,
                column: position.column + 1,
                key,
            });
        }
        
        None
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
    fn test_detects_inconsistent_pattern_styles() {
        let code = r#"
defmodule Example do
  def mixed_styles(%{name: name}, %{id: user_id}) do
    {name, user_id}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ParameterPatternMatching;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX1004");
        assert!(violations[0].message.contains("Inconsistent pattern style"));
    }

    #[test]
    fn test_allows_consistent_key_matching_style() {
        let code = r#"
defmodule Example do
  def key_matching(%{name: name}, %{id: id}, %{title: title}) do
    {name, id, title}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ParameterPatternMatching;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_consistent_explicit_naming_style() {
        let code = r#"
defmodule Example do
  def explicit_naming(%{name: user_name}, %{id: user_id}, %{title: post_title}) do
    {user_name, user_id, post_title}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ParameterPatternMatching;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_single_pattern() {
        let code = r#"
defmodule Example do
  def single_pattern(%{name: name}) do
    name
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ParameterPatternMatching;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_handles_nested_patterns() {
        let code = r#"
defmodule Example do
  def nested_patterns(%{user: %{name: name}}, %{post: %{id: post_id}}) do
    {name, post_id}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ParameterPatternMatching;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("Inconsistent pattern style"));
    }

    #[test]
    fn test_handles_multiple_functions() {
        let code = r#"
defmodule Example do
  def func1(%{name: name}, %{id: id}) do
    {name, id}
  end
  
  def func2(%{title: post_title}, %{content: post_content}) do
    {post_title, post_content}
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = ParameterPatternMatching;
        let violations = lint.check(&tree, code);

        // Each function should be consistent within itself
        assert_eq!(violations.len(), 0);
    }
}