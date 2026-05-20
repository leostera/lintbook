use tree_sitter::{Node, Tree, TreeCursor};
use lintbook_core::{LintViolation, Rule};

pub struct IExPry;

impl Rule for IExPry {
    fn id(&self) -> &'static str {
        "EX3001"
    }

    fn name(&self) -> &'static str {
        "iex-pry"
    }

    fn description(&self) -> &'static str {
        "Detect leftover IEx.pry/0 calls"
    }

    fn explanation(&self) -> &'static str {
        "IEx.pry/0 calls are debugging tools that should not be left in production code. \
        They cause the code execution to stop and start an interactive debugging session, \
        which can cause applications to hang in production environments. These calls \
        should be removed before committing code."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut cursor = tree.walk();

        self.traverse_node(tree.root_node(), &mut cursor, source, &mut violations);
        violations
    }
}

impl IExPry {
    fn traverse_node(
        &self,
        node: Node,
        cursor: &mut TreeCursor,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for function call patterns that might be IEx.pry
        if node.kind() == "call" {
            if self.is_iex_pry_call(node, source) {
                let start_position = node.start_position();
                violations.push(LintViolation {
                    line: start_position.row + 1,
                    column: start_position.column + 1,
                    message: "Remove leftover IEx.pry/0 call before committing".to_string(),
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

    fn is_iex_pry_call(&self, node: Node, source: &str) -> bool {
        // Check if this is a call to IEx.pry()
        if node.kind() != "call" {
            return false;
        }

        // Get the function being called
        if let Some(function_node) = node.child(0) {
            let function_text = &source[function_node.start_byte()..function_node.end_byte()];

            // Check for various patterns of IEx.pry calls
            if function_text == "IEx.pry" {
                // Check if it's a zero-arity call: IEx.pry() or IEx.pry
                return self.is_zero_arity_call(node, source);
            }

            // Also check for qualified calls that might be aliased
            if function_node.kind() == "dot" {
                return self.is_qualified_iex_pry(function_node, source);
            }

            // Check for simple pry calls (if IEx is imported)
            if function_text == "pry" {
                return self.is_zero_arity_call(node, source);
            }
        }

        false
    }

    fn is_qualified_iex_pry(&self, dot_node: Node, source: &str) -> bool {
        // Check if this is a qualified call like IEx.pry
        if dot_node.kind() != "dot" {
            return false;
        }

        let dot_text = &source[dot_node.start_byte()..dot_node.end_byte()];
        dot_text == "IEx.pry"
    }

    fn is_zero_arity_call(&self, call_node: Node, source: &str) -> bool {
        // Check if this is a zero-arity function call (no arguments)
        // Look for arguments node
        for i in 0..call_node.child_count() {
            if let Some(child) = call_node.child(i) {
                if child.kind() == "arguments" {
                    // Check if arguments are empty
                    let args_text = &source[child.start_byte()..child.end_byte()];
                    return args_text == "()" || args_text.trim().is_empty();
                }
            }
        }

        // If no arguments node found, it might be a bare function call
        true
    }
}
