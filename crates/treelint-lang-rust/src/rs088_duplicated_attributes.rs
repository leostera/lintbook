use tree_sitter::{Node, Tree};
use treelint_core::*;
use std::collections::HashMap;

pub struct DuplicatedAttributes;

impl Rule for DuplicatedAttributes {
    fn id(&self) -> &'static str {
        "RS088"
    }

    fn name(&self) -> &'static str {
        "duplicated-attributes"
    }

    fn description(&self) -> &'static str {
        "Checks for duplicated attributes on items"
    }

    fn explanation(&self) -> &'static str {
        "Duplicated attributes are redundant and may indicate copy-paste errors. \
         Each attribute should only be applied once per item."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl DuplicatedAttributes {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for items that can have attributes
        if matches!(node.kind(), 
            "function_item" | "struct_item" | "enum_item" | "impl_item" | 
            "trait_item" | "mod_item" | "const_item" | "static_item" |
            "type_item" | "field_declaration" | "parameter"
        ) {
            self.check_attributes_for_item(node, source, violations);
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }

    fn check_attributes_for_item(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut attribute_counts: HashMap<String, Vec<Node>> = HashMap::new();
        
        // Collect all attributes for this item
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "attribute_item" {
                if let Some(attr_name) = extract_attribute_name(child, source) {
                    attribute_counts.entry(attr_name).or_insert_with(Vec::new).push(child);
                }
            }
        }

        // Report duplicates
        for (attr_name, nodes) in attribute_counts {
            if nodes.len() > 1 {
                for node in nodes.iter().skip(1) { // Skip the first occurrence
                    let position = node.start_position();
                    violations.push(LintViolation {
                        line: position.row + 1,
                        column: position.column + 1,
                        message: format!(
                            "Duplicate attribute `{}` found. This attribute was already applied to this item",
                            attr_name
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }
    }
}

fn extract_attribute_name(attr_node: Node, source: &str) -> Option<String> {
    // Look for the attribute path inside the attribute
    let mut cursor = attr_node.walk();
    for child in attr_node.children(&mut cursor) {
        if child.kind() == "meta_item" {
            // Get the path part of the meta item
            let mut meta_cursor = child.walk();
            for meta_child in child.children(&mut meta_cursor) {
                if meta_child.kind() == "identifier" || meta_child.kind() == "scoped_identifier" {
                    return Some(source[meta_child.byte_range()].to_string());
                }
            }
        } else if child.kind() == "identifier" || child.kind() == "scoped_identifier" {
            return Some(source[child.byte_range()].to_string());
        }
    }
    
    // Fallback: try to extract the whole attribute content
    let attr_text = &source[attr_node.byte_range()];
    if attr_text.starts_with("#[") && attr_text.ends_with("]") {
        let content = &attr_text[2..attr_text.len()-1];
        // Get the first word/identifier
        if let Some(first_word) = content.split_whitespace().next() {
            return Some(first_word.split('(').next().unwrap_or(first_word).to_string());
        }
    }
    
    None
}