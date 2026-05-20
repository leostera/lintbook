use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct LetUnderscoreLock;

impl Rule for LetUnderscoreLock {
    fn id(&self) -> &'static str {
        "RS029"
    }

    fn name(&self) -> &'static str {
        "let-underscore-lock"
    }

    fn description(&self) -> &'static str {
        "Detects `let _ = lock` patterns that immediately drop locks"
    }

    fn explanation(&self) -> &'static str {
        "Using `let _ = mutex.lock()` immediately drops the lock guard, providing no \
         synchronization. Use `let _guard = mutex.lock()` or assign to a named variable \
         to keep the lock held for the desired scope."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl LetUnderscoreLock {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for let statements
        if node.kind() == "let_declaration" {
            if let Some(pattern) = node.child_by_field_name("pattern") {
                // Check if the pattern is just "_"
                if pattern.kind() == "_" || self.is_underscore_pattern(pattern, source) {
                    if let Some(value) = node.child_by_field_name("value") {
                        if self.is_lock_call(value, source) {
                            let position = node.start_position();
                            violations.push(LintViolation {
                                line: position.row + 1,
                                column: position.column + 1,
                                message: "Immediately dropping a lock guard with `let _` provides no synchronization. Use `let _guard =` or a named variable".to_string(),
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

    fn is_underscore_pattern(&self, pattern: Node, source: &str) -> bool {
        let pattern_text = &source[pattern.byte_range()];
        pattern_text.trim() == "_"
    }

    fn is_lock_call(&self, value_node: Node, source: &str) -> bool {
        let value_text = &source[value_node.byte_range()];

        // Check for various lock-related method calls
        if value_node.kind() == "call_expression" {
            if let Some(function) = value_node.child_by_field_name("function") {
                if function.kind() == "field_expression" {
                    if let Some(field) = function.child_by_field_name("field") {
                        let method_name = &source[field.byte_range()];

                        // Check for common lock methods
                        if matches!(
                            method_name,
                            "lock"
                                | "try_lock"
                                | "read"
                                | "write"
                                | "try_read"
                                | "try_write"
                                | "blocking_lock"
                                | "blocking_read"
                                | "blocking_write"
                        ) {
                            // Verify the object looks like a mutex/lock type
                            if let Some(object) = function.child_by_field_name("object") {
                                return self.looks_like_lock_object(object, source);
                            }
                        }
                    }
                }
            }
        }

        // Also check for await expressions on lock calls
        if value_node.kind() == "await_expression" {
            if let Some(operand) = value_node.child_by_field_name("operand") {
                return self.is_lock_call(operand, source);
            }
        }

        // Check for try expressions (?)
        if value_node.kind() == "try_expression" {
            if let Some(operand) = value_node.child_by_field_name("operand") {
                return self.is_lock_call(operand, source);
            }
        }

        // Check for method call that returns Result<Guard, _> and is unwrapped
        if value_text.contains(".lock()")
            || value_text.contains(".try_lock()")
            || value_text.contains(".read()")
            || value_text.contains(".write()")
            || value_text.contains(".try_read()")
            || value_text.contains(".try_write()")
            || value_text.contains(".blocking_lock()")
            || value_text.contains(".blocking_read()")
            || value_text.contains(".blocking_write()")
        {
            return true;
        }

        false
    }

    fn looks_like_lock_object(&self, object: Node, source: &str) -> bool {
        let object_text = &source[object.byte_range()];

        // Check for common lock/mutex types and variable names
        object_text.contains("mutex") ||
        object_text.contains("Mutex") ||
        object_text.contains("RwLock") ||
        object_text.contains("rwlock") ||
        object_text.contains("lock") ||
        object_text.contains("Lock") ||
        object_text.contains("LOCK") ||
        object_text.ends_with("_lock") ||
        object_text.ends_with("_mutex") ||
        object_text.starts_with("lock_") ||
        object_text.starts_with("mutex_") ||
        // Check for std library paths
        object_text.contains("std::sync::") ||
        object_text.contains("tokio::sync::") ||
        object_text.contains("parking_lot::") ||
        // Check for Arc<Mutex<>> patterns
        object_text.contains("Arc<") ||
        // Variable names that suggest locks
        matches!(object_text, "m" | "mtx" | "guard" | "lck")
    }
}
