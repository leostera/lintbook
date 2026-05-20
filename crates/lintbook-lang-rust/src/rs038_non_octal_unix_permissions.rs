use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct NonOctalUnixPermissions;

impl Rule for NonOctalUnixPermissions {
    fn id(&self) -> &'static str {
        "RS038"
    }

    fn name(&self) -> &'static str {
        "non-octal-unix-permissions"
    }

    fn description(&self) -> &'static str {
        "Checks for non-octal values used to set Unix file permissions"
    }

    fn explanation(&self) -> &'static str {
        "They will be converted into octal, creating potentially unintended file permissions. \
        Use octal notation like 0o644 instead of decimal 644."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl NonOctalUnixPermissions {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for method calls that might be setting Unix permissions
        if node.kind() == "call_expression" {
            if let Some(function) = node.child_by_field_name("function") {
                if self.is_permission_setting_method(function, source) {
                    if let Some(args) = node.child_by_field_name("arguments") {
                        self.check_permission_arguments(args, source, violations);
                    }
                }
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn is_permission_setting_method(&self, function_node: Node, source: &str) -> bool {
        let function_text = &source[function_node.byte_range()];

        // Check for common Unix permission setting methods
        function_text.ends_with(".mode") ||
        function_text.ends_with(".create_with_mode") ||
        function_text.ends_with(".set_mode") ||
        function_text == "mode" ||
        function_text == "create_with_mode" ||
        function_text == "set_mode" ||

        // Check for DirBuilder methods
        function_text.ends_with(".mode") ||

        // Check for std::fs methods that take mode parameters
        self.is_fs_method_with_mode(function_text)
    }

    fn is_fs_method_with_mode(&self, function_text: &str) -> bool {
        // Common std::fs and std::os::unix methods that take mode parameters
        function_text.contains("create_dir_all") ||
        function_text.contains("create_dir") ||
        function_text.contains("File::create") ||
        function_text.contains("OpenOptions") ||
        function_text.contains("DirBuilder") ||
        function_text.contains("Permissions::from_mode") ||
        function_text.contains("set_permissions") ||

        // Unix-specific extensions
        function_text.contains("unix::fs") ||
        function_text.contains("unix::os")
    }

    fn check_permission_arguments(
        &self,
        args_node: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut cursor = args_node.walk();
        for child in args_node.children(&mut cursor) {
            if child.kind() == "integer_literal" {
                let literal_text = &source[child.byte_range()];

                if self.is_likely_unix_permission(literal_text) {
                    let position = child.start_position();
                    let suggestion = self.suggest_octal_notation(literal_text);

                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: format!(
                            "Non-octal value {} used for Unix permissions. Use {} instead to avoid confusion",
                            literal_text, suggestion
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }

            // Recursively check nested expressions
            self.check_permission_arguments(child, source, violations);
        }
    }

    fn is_likely_unix_permission(&self, literal: &str) -> bool {
        // Remove any underscores and suffixes for analysis
        let cleaned = literal.replace('_', "");
        let cleaned = cleaned.trim_end_matches(|c: char| c.is_alphabetic()); // Remove type suffixes

        // Skip if it's already in octal notation
        if cleaned.starts_with("0o") || cleaned.starts_with("0O") {
            return false;
        }

        // Skip if it's hex or binary
        if cleaned.starts_with("0x")
            || cleaned.starts_with("0X")
            || cleaned.starts_with("0b")
            || cleaned.starts_with("0B")
        {
            return false;
        }

        // Check if it looks like a common Unix permission pattern
        if let Ok(num) = cleaned.parse::<u32>() {
            // First check for common decimal equivalents of octal permissions
            match num {
                420 => true, // 0o644 in decimal
                436 => true, // 0o664 in decimal
                448 => true, // 0o700 in decimal
                484 => true, // 0o744 in decimal
                493 => true, // 0o755 in decimal
                _ => {
                    // Then check for 3-4 digit patterns that look like octal
                    match num {
                        // 3-digit patterns (user, group, other)
                        000..=777 => {
                            // Check if each digit is valid octal (0-7)
                            let digits: Vec<char> = num.to_string().chars().collect();
                            if digits.len() <= 3 {
                                digits.iter().all(|&d| d >= '0' && d <= '7')
                            } else {
                                false
                            }
                        }
                        // 4-digit patterns (special bits + user, group, other)
                        1000..=7777 => {
                            let digits: Vec<char> = num.to_string().chars().collect();
                            if digits.len() == 4 {
                                digits.iter().all(|&d| d >= '0' && d <= '7')
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                }
            }
        } else {
            false
        }
    }

    fn suggest_octal_notation(&self, literal: &str) -> String {
        let cleaned = literal.replace('_', "");
        let cleaned = cleaned.trim_end_matches(|c: char| c.is_alphabetic());

        if let Ok(num) = cleaned.parse::<u32>() {
            // If it's a 3 or 4 digit number that could be octal, suggest the octal notation
            if num <= 7777 {
                let digits: Vec<char> = num.to_string().chars().collect();
                if digits.iter().all(|&d| d >= '0' && d <= '7') {
                    return format!("0o{}", num);
                }
            }

            // For other cases, convert the decimal to what it would be in octal
            match num {
                420 => "0o644".to_string(),
                436 => "0o664".to_string(),
                448 => "0o700".to_string(),
                484 => "0o744".to_string(),
                493 => "0o755".to_string(),
                _ => format!("0o{:o}", num), // Convert to octal representation
            }
        } else {
            format!("0o{}", literal)
        }
    }
}
