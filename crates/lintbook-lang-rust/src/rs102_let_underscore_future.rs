use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct LetUnderscoreFuture;

impl Rule for LetUnderscoreFuture {
    fn id(&self) -> &'static str {
        "RS102"
    }

    fn name(&self) -> &'static str {
        "let-underscore-future"
    }

    fn description(&self) -> &'static str {
        "Checks for let _ = future patterns"
    }

    fn explanation(&self) -> &'static str {
        "Assigning a Future to let _ means the future is never polled and will never run. \
         Use .await or spawn the future to execute it."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl LetUnderscoreFuture {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "let_declaration" {
            if let Some(pattern_node) = node.child_by_field_name("pattern") {
                let pattern_text = &source[pattern_node.byte_range()];

                if pattern_text == "_" {
                    if let Some(value_node) = node.child_by_field_name("value") {
                        let value_text = &source[value_node.byte_range()];

                        if is_future_expression(value_text) {
                            let position = node.start_position();
                            violations.push(LintViolation {
                                line: position.row + 1,
                                column: position.column + 1,
                                message: "Future assigned to `let _` will never be polled. Use .await or spawn it".to_string(),
                                lint_name: self.name().to_string(),
                                lint_id: self.id().to_string(),
                            });
                        }
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_future_expression(expr: &str) -> bool {
    // Look for async blocks or calls that return futures
    expr.contains("async") && (expr.contains("move") || expr.contains("{")) ||
    expr.contains("Future") ||
    expr.contains("futures::") ||
    expr.contains("tokio::") ||
    expr.contains("async_std::") ||
    // Common async function patterns
    expr.contains("_async(") ||
    expr.ends_with("_async()") ||
    // Async method calls without .await
    (expr.contains(".") && expr.contains("async") && !expr.contains(".await"))
}
