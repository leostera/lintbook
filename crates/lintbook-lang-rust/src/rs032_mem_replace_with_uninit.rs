use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct MemReplaceWithUninit;

impl Rule for MemReplaceWithUninit {
    fn id(&self) -> &'static str {
        "RS032"
    }

    fn name(&self) -> &'static str {
        "mem-replace-with-uninit"
    }

    fn description(&self) -> &'static str {
        "Detects `mem::replace` with `mem::uninitialized()` which is dangerous"
    }

    fn explanation(&self) -> &'static str {
        "Using `mem::replace(x, mem::uninitialized())` is dangerous because it leaves \
         uninitialized memory in place. Use `mem::take()` or `ptr::read()` instead for safe alternatives."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl MemReplaceWithUninit {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for call expressions
        if node.kind() == "call_expression" {
            if let Some(function) = node.child_by_field_name("function") {
                // Check if this is a mem::replace call
                if self.is_mem_replace_call(function, source) {
                    if let Some(args) = node.child_by_field_name("arguments") {
                        if self.has_uninit_argument(args, source) {
                            let position = node.start_position();
                            violations.push(LintViolation {
                                line: position.row + 1,
                                column: position.column + 1,
                                message: "Using `mem::replace` with `mem::uninitialized()` is dangerous. Use `mem::take()` or `ptr::read()` instead".to_string(),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
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

    fn is_mem_replace_call(&self, function_node: Node, source: &str) -> bool {
        let function_text = &source[function_node.byte_range()];

        // Check for various ways to call mem::replace
        function_text == "mem::replace" ||
        function_text == "std::mem::replace" ||
        function_text == "core::mem::replace" ||
        // Check for use statements that import mem::replace
        (function_text == "replace" && self.likely_in_mem_context(function_node, source))
    }

    fn likely_in_mem_context(&self, _function_node: Node, _source: &str) -> bool {
        // This is a simplified check - in a real implementation, we'd need to
        // track imports and scope to be more accurate
        true
    }

    fn has_uninit_argument(&self, args_node: Node, source: &str) -> bool {
        let mut cursor = args_node.walk();
        for child in args_node.children(&mut cursor) {
            if child.kind() == "call_expression" {
                let arg_text = &source[child.byte_range()];

                // Check for various ways to call mem::uninitialized
                if arg_text.contains("mem::uninitialized()")
                    || arg_text.contains("std::mem::uninitialized()")
                    || arg_text.contains("core::mem::uninitialized()")
                    || arg_text.contains("uninitialized()")
                    || arg_text.contains("MaybeUninit::uninit().assume_init()")
                {
                    return true;
                }
            }

            // Also check nested expressions
            if self.has_uninit_argument(child, source) {
                return true;
            }
        }

        false
    }
}
