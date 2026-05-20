use std::collections::HashMap;
use tree_sitter::{Node, Tree};
use lintbook_core::*;

pub struct DuplicateMod;

impl Rule for DuplicateMod {
    fn id(&self) -> &'static str {
        "RS087"
    }

    fn name(&self) -> &'static str {
        "duplicate-mod"
    }

    fn description(&self) -> &'static str {
        "Checks for duplicate module declarations"
    }

    fn explanation(&self) -> &'static str {
        "Duplicate module declarations are not allowed and will cause compilation errors."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let mut module_names: HashMap<String, Vec<Node>> = HashMap::new();

        self.collect_modules(tree.root_node(), source, &mut module_names);

        for (name, nodes) in module_names {
            if nodes.len() > 1 {
                for node in nodes.iter().skip(1) {
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: format!("Duplicate module declaration: '{}'", name),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        violations
    }
}

impl DuplicateMod {
    fn collect_modules<'a>(
        &self,
        node: Node<'a>,
        source: &str,
        modules: &mut HashMap<String, Vec<Node<'a>>>,
    ) {
        if node.kind() == "mod_item" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let module_name = source[name_node.byte_range()].to_string();
                modules
                    .entry(module_name)
                    .or_insert_with(Vec::new)
                    .push(node);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_modules(child, source, modules);
        }
    }
}
