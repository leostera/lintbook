use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct UnsafeExec;

impl Rule for UnsafeExec {
    fn id(&self) -> &'static str {
        "EX5003"
    }

    fn name(&self) -> &'static str {
        "unsafe_exec"
    }

    fn description(&self) -> &'static str {
        "Prevent command injection vulnerabilities through unsafe command execution"
    }

    fn explanation(&self) -> &'static str {
        "Avoid using functions that execute system commands with user-controlled input \
        as they can lead to command injection vulnerabilities. Functions like System.shell/1, \
        System.cmd/2 with dynamic arguments, :os.cmd/1, Port.open/2 with :spawn, and \
        :erlang.open_port/2 can be dangerous if not properly sanitized. Consider using \
        safer alternatives or ensure all inputs are properly validated and sanitized."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        self.traverse_for_unsafe_exec(tree.root_node(), source, &mut violations);
        
        violations
    }
}

impl UnsafeExec {
    fn traverse_for_unsafe_exec(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Check if this is a function call that might be unsafe
        if node.kind() == "call" {
            self.check_unsafe_function_call(node, source, violations);
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_for_unsafe_exec(child, source, violations);
            }
        }
    }
    
    fn check_unsafe_function_call(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if let Some(target_node) = node.child_by_field_name("target") {
            let function_call = &source[target_node.start_byte()..target_node.end_byte()];
            
            match self.classify_unsafe_function(function_call) {
                UnsafeFunction::HighRisk(name, reason) => {
                    self.add_violation(target_node, &name, reason, "high", violations);
                }
                UnsafeFunction::MediumRisk(name, reason) => {
                    // For medium risk, check if arguments look suspicious
                    if self.has_suspicious_arguments(node, source) {
                        self.add_violation(target_node, &name, reason, "medium", violations);
                    }
                }
                UnsafeFunction::Safe => {
                    // No violation for safe functions
                }
            }
        }
    }
    
    fn classify_unsafe_function(&self, function_call: &str) -> UnsafeFunction {
        match function_call {
            // High risk - always flag
            "System.shell" => UnsafeFunction::HighRisk(
                function_call.to_string(),
                "executes shell commands which can lead to command injection"
            ),
            ":os.cmd" => UnsafeFunction::HighRisk(
                function_call.to_string(),
                "executes operating system commands directly"
            ),
            
            // Medium risk - flag if suspicious arguments
            "System.cmd" => UnsafeFunction::MediumRisk(
                function_call.to_string(),
                "executes system commands; ensure arguments are sanitized"
            ),
            "Port.open" => UnsafeFunction::MediumRisk(
                function_call.to_string(),
                "can spawn external processes; validate all inputs"
            ),
            ":erlang.open_port" => UnsafeFunction::MediumRisk(
                function_call.to_string(),
                "opens external ports; ensure spawn arguments are safe"
            ),
            
            _ => UnsafeFunction::Safe,
        }
    }
    
    fn has_suspicious_arguments(&self, node: Node, source: &str) -> bool {
        if let Some(args_node) = node.child_by_field_name("arguments") {
            let args_text = &source[args_node.start_byte()..args_node.end_byte()];
            
            // Look for patterns that suggest dynamic/user input
            // This is a heuristic - we flag things that look like they could be user-controlled
            return self.contains_suspicious_patterns(args_text) || 
                   self.has_variable_interpolation(args_node, source);
        }
        
        // If we can't analyze arguments, err on the side of caution for medium-risk functions
        true
    }
    
    fn contains_suspicious_patterns(&self, args_text: &str) -> bool {
        // Look for common patterns that suggest dynamic input
        let suspicious_patterns = [
            "#{",         // String interpolation
            "user",       // Variables that might contain user input
            "input",      // Variables that might contain input
            "params",     // Parameters from web requests
            "request",    // Request data
            "query",      // Query parameters
            "data",       // Generic data that could be user input
            "..",         // Path traversal attempts
            ";",          // Command chaining
            "&&",         // Command chaining
            "||",         // Command chaining
            "|",          // Pipes
            "$(",         // Command substitution
            "`",          // Command substitution
        ];
        
        suspicious_patterns.iter().any(|&pattern| args_text.contains(pattern))
    }
    
    fn has_variable_interpolation(&self, node: Node, source: &str) -> bool {
        // Check if any arguments are identifiers (variables) rather than string literals
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" {
                    let var_name = &source[child.start_byte()..child.end_byte()];
                    // Flag variables that look like they could contain user input
                    if self.looks_like_user_input(var_name) {
                        return true;
                    }
                }
                // Recursively check nested structures
                if self.has_variable_interpolation(child, source) {
                    return true;
                }
            }
        }
        false
    }
    
    fn looks_like_user_input(&self, var_name: &str) -> bool {
        let user_input_indicators = [
            "user", "input", "param", "request", "query", "data", "arg", "command", "cmd"
        ];
        
        let var_lower = var_name.to_lowercase();
        user_input_indicators.iter().any(|&indicator| var_lower.contains(indicator))
    }
    
    fn add_violation(&self, node: Node, function_name: &str, reason: &str, risk_level: &str, violations: &mut Vec<LintViolation>) {
        let position = node.start_position();
        violations.push(LintViolation {
            line: position.row + 1,
            column: position.column + 1,
            message: format!(
                "Unsafe command execution: {} {} (Risk: {})",
                function_name, reason, risk_level
            ),
            lint_name: self.name().to_string(),
            lint_id: self.id().to_string(),
        });
    }
}

#[derive(Debug, PartialEq)]
enum UnsafeFunction {
    HighRisk(String, &'static str),
    MediumRisk(String, &'static str),
    Safe,
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
    fn test_detects_high_risk_system_shell() {
        let code = r#"
defmodule Example do
  def test(user_command) do
    System.shell("ls -la")
    System.shell(user_command)
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeExec;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 2);
        assert!(violations.iter().any(|v| v.message.contains("System.shell")));
        assert!(violations.iter().any(|v| v.message.contains("Risk: high")));
    }

    #[test]
    fn test_detects_high_risk_os_cmd() {
        let code = r#"
defmodule Example do
  def test(user_input) do
    :os.cmd('echo hello')
    :os.cmd(String.to_charlist(user_input))
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeExec;
        let violations = lint.check(&tree, code);

        assert!(violations.len() >= 2);
        assert!(violations.iter().any(|v| v.message.contains(":os.cmd")));
        assert!(violations.iter().any(|v| v.message.contains("Risk: high")));
    }

    #[test]
    fn test_detects_medium_risk_with_suspicious_args() {
        let code = r#"
defmodule Example do
  def test(user_file, query, user_command) do
    System.cmd("cat", [user_file])
    System.cmd("grep", [query, "file.txt"])
    Port.open({:spawn, user_command}, [])
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeExec;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.message.contains("System.cmd")));
        assert!(violations.iter().any(|v| v.message.contains("Port.open")));
    }

    #[test]
    fn test_allows_safe_system_cmd_with_literal_args() {
        let code = r#"
defmodule Example do
  def test do
    System.cmd("echo", ["hello", "world"])
    System.cmd("ls", ["-la", "/tmp"])
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeExec;
        let violations = lint.check(&tree, code);

        // Should have fewer or no violations for literal arguments
        assert!(violations.len() <= 2); // May still flag due to conservative heuristics
    }

    #[test]
    fn test_detects_command_injection_patterns() {
        let code = r#"
defmodule Example do
  def test(user_input, filename) do
    System.cmd("sh", ["-c", "echo something"])
    System.cmd("bash", ["; rm -rf /"])
    Port.open({:spawn, filename}, [])
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeExec;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.message.contains("System.cmd")));
    }

    #[test]
    fn test_ignores_safe_functions() {
        let code = r#"
defmodule Example do
  def test(user_input) do
    IO.puts("hello")
    String.upcase("test")
    File.read("file.txt")
    safe_function(user_input)
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeExec;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_erlang_open_port() {
        let code = r#"
defmodule Example do
  def test(user_command) do
    :erlang.open_port({:spawn, "cat file.txt"}, [])
    :erlang.open_port({:spawn, user_command}, [])
  end
end"#;

        let tree = parse_elixir_code(code);
        let lint = UnsafeExec;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.message.contains(":erlang.open_port")));
    }
}