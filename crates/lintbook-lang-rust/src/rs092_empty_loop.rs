use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct EmptyLoop;

impl Rule for EmptyLoop {
    fn id(&self) -> &'static str {
        "RS092"
    }

    fn name(&self) -> &'static str {
        "empty-loop"
    }

    fn description(&self) -> &'static str {
        "Checks for empty loop bodies"
    }

    fn explanation(&self) -> &'static str {
        "Empty loops without any body statements usually indicate unfinished code or busy-waiting. \
         For busy-waiting, consider using std::thread::yield_now() or std::hint::spin_loop()."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl EmptyLoop {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        match node.kind() {
            "loop_expression" | "while_expression" | "for_expression" => {
                if let Some(body_node) = node.child_by_field_name("body") {
                    if is_empty_block(body_node, source) {
                        let position = node.start_position();
                        let loop_kind = match node.kind() {
                            "loop_expression" => "loop",
                            "while_expression" => "while",
                            "for_expression" => "for",
                            _ => "loop",
                        };

                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: format!(
                                "Empty {} loop body - consider using std::thread::yield_now() or std::hint::spin_loop() for busy-waiting",
                                loop_kind
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
            _ => {}
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_empty_block(node: Node, source: &str) -> bool {
    if node.kind() != "block" {
        return false;
    }

    let mut cursor = node.walk();
    let mut has_statements = false;

    for child in node.children(&mut cursor) {
        // Skip braces and whitespace, look for actual statements
        if !matches!(child.kind(), "{" | "}" | "comment") {
            let child_text = &source[child.byte_range()].trim();
            if !child_text.is_empty() {
                has_statements = true;
                break;
            }
        }
    }

    !has_statements
}
