use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct PointersInNomemAsmBlock;

impl Rule for PointersInNomemAsmBlock {
    fn id(&self) -> &'static str {
        "RS121"
    }

    fn name(&self) -> &'static str {
        "pointers-in-nomem-asm-block"
    }

    fn description(&self) -> &'static str {
        "Checks for pointers used in nomem asm blocks"
    }

    fn explanation(&self) -> &'static str {
        "Using pointers in nomem asm blocks can be unsafe since the compiler \
         assumes no memory is accessed. Remove nomem or avoid pointer operations."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl PointersInNomemAsmBlock {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "macro_invocation" {
            let macro_text = source[node.byte_range()].trim();
            
            if macro_text.starts_with("asm!") && macro_text.contains("nomem") {
                // Check if the asm block contains pointer operations
                if contains_pointer_operations(macro_text) {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: "Pointers used in nomem asm block - this can be unsafe since compiler assumes no memory access".to_string(),
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

fn contains_pointer_operations(asm_text: &str) -> bool {
    // Look for pointer-related operations in asm
    asm_text.contains("*") && (
        asm_text.contains("ptr") ||
        asm_text.contains("&") ||
        asm_text.contains("as *") ||
        asm_text.contains("deref") ||
        asm_text.contains("addr_of")
    )
}