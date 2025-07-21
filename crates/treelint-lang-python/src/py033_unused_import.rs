use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};
use std::collections::HashSet;

pub struct UnusedImport;

#[derive(Debug, Clone)]
struct ImportInfo {
    name: String,
    alias: Option<String>,
    line: usize,
    _node_id: usize,
}

impl Rule for UnusedImport {
    fn id(&self) -> &'static str {
        "PY033"
    }

    fn name(&self) -> &'static str {
        "unused-import"
    }

    fn description(&self) -> &'static str {
        "Module imported but unused"
    }

    fn explanation(&self) -> &'static str {
        "Imported modules should be used in the code. Unused imports clutter the namespace and can slow down module loading."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        // First pass: collect all imports
        let imports = self.collect_imports(root_node, source);
        
        // Second pass: find usage of imported names
        let used_names = self.collect_used_names(root_node, source);
        
        // Check which imports are unused
        for import in &imports {
            let name_to_check = import.alias.as_ref().unwrap_or(&import.name);
            if !used_names.contains(name_to_check) {
                violations.push(LintViolation {
                    line: import.line,
                    column: 1,
                    message: format!("'{}' imported but unused", import.name),
                    lint_id: self.id().to_string(),
                    lint_name: self.name().to_string(),
                });
            }
        }
        
        violations
    }
}

impl UnusedImport {
    fn collect_imports(&self, node: Node, source: &str) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        self.visit_for_imports(node, source, &mut imports);
        imports
    }

    fn visit_for_imports(&self, node: Node, source: &str, imports: &mut Vec<ImportInfo>) {
        match node.kind() {
            "import_statement" => {
                self.process_import_statement(node, source, imports);
            },
            "import_from_statement" => {
                self.process_import_from_statement(node, source, imports);
            },
            _ => {}
        }

        // Continue traversal
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_for_imports(child, source, imports);
            }
        }
    }

    fn process_import_statement(&self, node: Node, source: &str, imports: &mut Vec<ImportInfo>) {
        // Handle: import module, import module as alias
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "dotted_as_names" || child.kind() == "dotted_as_name" {
                    self.process_dotted_as_names(child, source, imports);
                } else if child.kind() == "dotted_name" || child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        imports.push(ImportInfo {
                            name: name.to_string(),
                            alias: None,
                            line: child.start_position().row + 1,
                            _node_id: child.id(),
                        });
                    }
                }
            }
        }
    }

    fn process_import_from_statement(&self, node: Node, source: &str, imports: &mut Vec<ImportInfo>) {
        // Handle: from module import name, from module import name as alias
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "import_list" {
                    self.process_import_list(child, source, imports);
                }
            }
        }
    }

    fn process_dotted_as_names(&self, node: Node, source: &str, imports: &mut Vec<ImportInfo>) {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "dotted_as_name" {
                    self.process_single_as_name(child, source, imports);
                } else if child.kind() == "dotted_name" || child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        imports.push(ImportInfo {
                            name: name.to_string(),
                            alias: None,
                            line: child.start_position().row + 1,
                            _node_id: child.id(),
                        });
                    }
                }
            }
        }
    }

    fn process_import_list(&self, node: Node, source: &str, imports: &mut Vec<ImportInfo>) {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "aliased_import" => {
                        self.process_single_as_name(child, source, imports);
                    },
                    "identifier" => {
                        if let Ok(name) = child.utf8_text(source.as_bytes()) {
                            imports.push(ImportInfo {
                                name: name.to_string(),
                                alias: None,
                                line: child.start_position().row + 1,
                                _node_id: child.id(),
                            });
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn process_single_as_name(&self, node: Node, source: &str, imports: &mut Vec<ImportInfo>) {
        let mut name = None;
        let mut alias = None;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "dotted_name" | "identifier" => {
                        if name.is_none() {
                            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                                name = Some(text.to_string());
                            }
                        } else if alias.is_none() {
                            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                                alias = Some(text.to_string());
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        if let Some(import_name) = name {
            imports.push(ImportInfo {
                name: import_name,
                alias,
                line: node.start_position().row + 1,
                _node_id: node.id(),
            });
        }
    }

    fn collect_used_names(&self, node: Node, source: &str) -> HashSet<String> {
        let mut used_names = HashSet::new();
        self.visit_for_usage(node, source, &mut used_names);
        used_names
    }

    fn visit_for_usage(&self, node: Node, source: &str, used_names: &mut HashSet<String>) {
        match node.kind() {
            "identifier" => {
                // Skip identifiers in import statements
                if !self.is_in_import_context(node) {
                    if let Ok(name) = node.utf8_text(source.as_bytes()) {
                        used_names.insert(name.to_string());
                    }
                }
            },
            "attribute" => {
                // Handle module.attribute access
                if let Some(object) = node.child(0) {
                    if object.kind() == "identifier" {
                        if let Ok(name) = object.utf8_text(source.as_bytes()) {
                            used_names.insert(name.to_string());
                        }
                    }
                }
            },
            _ => {}
        }

        // Continue traversal
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_for_usage(child, source, used_names);
            }
        }
    }

    fn is_in_import_context(&self, node: Node) -> bool {
        let mut current = Some(node);
        while let Some(n) = current {
            match n.kind() {
                "import_statement" | "import_from_statement" => return true,
                "module" => return false,
                _ => current = n.parent(),
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_python(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_unused_simple_import() {
        let source = r#"
import os
import sys

print("Hello")
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both os and sys are unused
        assert!(violations.iter().any(|v| v.message.contains("os")));
        assert!(violations.iter().any(|v| v.message.contains("sys")));
    }

    #[test]
    fn test_used_import() {
        let source = r#"
import os
import sys

current_path = os.getcwd()
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Only sys is unused
        assert!(violations.iter().any(|v| v.message.contains("sys")));
        assert!(!violations.iter().any(|v| v.message.contains("os")));
    }

    #[test]
    fn test_unused_aliased_import() {
        let source = r#"
import os as operating_system
import sys as system

print("Hello")
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both aliases are unused
        assert!(violations.iter().any(|v| v.message.contains("os")));
        assert!(violations.iter().any(|v| v.message.contains("sys")));
    }

    #[test]
    fn test_used_aliased_import() {
        let source = r#"
import os as operating_system

path = operating_system.getcwd()
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // Alias is used
    }

    #[test]
    fn test_unused_from_import() {
        let source = r#"
from os import path
from sys import argv

print("Hello")
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        
        // Debug: check what imports are collected
        let imports = rule.collect_imports(tree.root_node(), source);
        println!("Collected imports: {:?}", imports);
        
        let used_names = rule.collect_used_names(tree.root_node(), source);
        println!("Used names: {:?}", used_names);
        
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // Both path and argv are unused
        assert!(violations.iter().any(|v| v.message.contains("path")));
        assert!(violations.iter().any(|v| v.message.contains("argv")));
    }

    #[test]
    fn test_used_from_import() {
        let source = r#"
from os import path
from sys import argv

current_dir = path.dirname(__file__)
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Only argv is unused
        assert!(violations.iter().any(|v| v.message.contains("argv")));
        assert!(!violations.iter().any(|v| v.message.contains("path")));
    }

    #[test]
    fn test_mixed_used_unused() {
        let source = r#"
import json
import os
from datetime import datetime

data = json.loads('{"key": "value"}')
now = datetime.now()
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Only os is unused
        assert!(violations.iter().any(|v| v.message.contains("os")));
    }

    #[test]
    fn test_import_used_in_function() {
        let source = r#"
import math

def calculate():
    return math.sqrt(16)
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // math is used in function
    }

    #[test]
    fn test_multiple_imports_same_line() {
        let source = r#"
import os, sys, json

current_path = os.getcwd()
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2); // sys and json are unused
        assert!(violations.iter().any(|v| v.message.contains("sys")));
        assert!(violations.iter().any(|v| v.message.contains("json")));
        assert!(!violations.iter().any(|v| v.message.contains("os")));
    }

    #[test]
    fn test_import_used_in_type_annotation() {
        let source = r#"
from typing import List

def process_items(items: List[str]) -> None:
    pass
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // List is used in type annotation
    }

    #[test]
    fn test_no_imports() {
        let source = r#"
def hello():
    print("Hello world")
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // No imports to check
    }

    #[test]
    fn test_import_used_in_class() {
        let source = r#"
import datetime

class TimestampedClass:
    def __init__(self):
        self.created = datetime.datetime.now()
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // datetime is used in class
    }

    #[test]
    fn test_aliased_from_import() {
        let source = r#"
from json import loads as json_loads

data = json_loads('{"key": "value"}')
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0); // Aliased import is used
    }

    #[test]
    fn test_unused_aliased_from_import() {
        let source = r#"
from json import loads as json_loads

print("Hello")
"#;
        let tree = parse_python(source);
        let rule = UnusedImport;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1); // Aliased import is unused
        assert!(violations[0].message.contains("loads"));
    }
}