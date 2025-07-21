use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct RcCloneInVecInit;

impl Rule for RcCloneInVecInit {
    fn id(&self) -> &'static str {
        "RS123"
    }

    fn name(&self) -> &'static str {
        "rc-clone-in-vec-init"
    }

    fn description(&self) -> &'static str {
        "Checks for Rc cloning in vec initialization"
    }

    fn explanation(&self) -> &'static str {
        "Using Rc::clone() in vec![rc.clone(); n] creates shared references. \
         Consider if this is intended or if you need separate instances."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl RcCloneInVecInit {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "macro_invocation" {
            let macro_text = source[node.byte_range()].trim();
            
            if macro_text.starts_with("vec!") && 
               macro_text.contains(".clone()") && 
               macro_text.contains(";") {
                
                // Check if it's cloning an Rc
                if macro_text.contains("Rc::") || 
                   macro_text.contains("Arc::") ||
                   is_likely_rc_clone(macro_text) {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "Cloning Rc/Arc in vec initialization creates shared references - verify this is intended".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn is_likely_rc_clone(text: &str) -> bool {
    // Look for patterns that suggest Rc/Arc cloning
    text.contains("rc.clone()") ||
    text.contains("arc.clone()") ||
    text.contains("shared.clone()") ||
    text.contains("ref_counted.clone()")
}