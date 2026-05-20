use crate::{LintViolation, Rule};
use tree_sitter::{Node, Tree};

pub struct AsyncYieldsAsync;

impl Rule for AsyncYieldsAsync {
    fn id(&self) -> &'static str {
        "RS004"
    }

    fn name(&self) -> &'static str {
        "async-yields-async"
    }

    fn description(&self) -> &'static str {
        "Detects async blocks that return awaitables without awaiting them"
    }

    fn explanation(&self) -> &'static str {
        "An async block that returns an awaitable (Future) without awaiting it is likely a mistake. \
         Either await the result with `.await` or remove the `async` from the block if you want to return the Future."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl AsyncYieldsAsync {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "async_block" {
            // Look for return expressions or last expressions that might be async
            if let Some(body) = node.child_by_field_name("body") {
                self.check_async_block_body(body, source, violations);
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn check_async_block_body(
        &self,
        body: Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "return_expression" {
                if let Some(value) = child.child_by_field_name("value") {
                    if is_likely_async_expression(value, source) {
                        let position = child.start_position();
                        violations.push(LintViolation {
                            line: position.row + 1,
                            column: position.column + 1,
                            message: "This async block returns a Future without awaiting it. Consider adding `.await` or removing `async`".to_string(),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            } else if child.kind() == "call_expression" && is_last_statement_in_block(child, body) {
                if is_likely_async_expression(child, source) {
                    let position = child.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "This async block returns a Future without awaiting it. Consider adding `.await` or removing `async`".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}

fn is_likely_async_expression(node: Node, source: &str) -> bool {
    let text = &source[node.byte_range()];

    // Check for common async function patterns
    text.contains("async")
        || text.contains("spawn")
        || text.contains("timeout")
        || text.contains("sleep")
        || text.ends_with("_async()")
        || text.contains("Future::")
}

fn is_last_statement_in_block(expr: Node, block: Node) -> bool {
    let mut last_child = None;
    let mut cursor = block.walk();
    for child in block.children(&mut cursor) {
        if child.kind() != "{" && child.kind() != "}" {
            last_child = Some(child);
        }
    }

    last_child.map_or(false, |last| last.id() == expr.id())
}
