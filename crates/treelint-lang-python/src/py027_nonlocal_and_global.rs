use treelint_core::{LintViolation, Rule};
use tree_sitter::{Node, Tree};
use std::collections::HashMap;

pub struct NonlocalAndGlobal;

impl Rule for NonlocalAndGlobal {
    fn id(&self) -> &'static str {
        "PY027"
    }

    fn name(&self) -> &'static str {
        "nonlocal-and-global"
    }

    fn description(&self) -> &'static str {
        "Name is both nonlocal and global"
    }

    fn explanation(&self) -> &'static str {
        "A variable cannot be declared as both global and nonlocal in the same scope. Use either global or nonlocal, but not both."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();
        
        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl NonlocalAndGlobal {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for function definitions
        if node.kind() == "function_definition" || node.kind() == "async_function_definition" {
            self.check_function_scope(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_function_scope(&self, function_node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut global_vars: HashMap<String, Node> = HashMap::new();
        let mut nonlocal_vars: HashMap<String, Node> = HashMap::new();
        
        // Find the function body
        if let Some(body) = self.get_function_body(function_node) {
            self.collect_declarations(body, source, &mut global_vars, &mut nonlocal_vars);
            
            // Check for conflicts
            for (var_name, nonlocal_node) in &nonlocal_vars {
                if let Some(global_node) = global_vars.get(var_name) {
                    // Report both declarations as violations
                    let global_start = global_node.start_position();
                    let nonlocal_start = nonlocal_node.start_position();
                    
                    violations.push(LintViolation {
                        line: global_start.row + 1,
                        column: global_start.column + 1,
                        message: format!("Variable '{}' is declared as both global and nonlocal in the same scope.", var_name),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                    
                    violations.push(LintViolation {
                        line: nonlocal_start.row + 1,
                        column: nonlocal_start.column + 1,
                        message: format!("Variable '{}' is declared as both global and nonlocal in the same scope.", var_name),
                        lint_id: self.id().to_string(),
                        lint_name: self.name().to_string(),
                    });
                }
            }
        }
    }

    fn get_function_body<'a>(&self, function_node: Node<'a>) -> Option<Node<'a>> {
        // Look for the block (function body)
        for i in 0..function_node.child_count() {
            if let Some(child) = function_node.child(i) {
                if child.kind() == "block" {
                    return Some(child);
                }
            }
        }
        None
    }

    fn collect_declarations<'a>(&self, node: Node<'a>, source: &str, global_vars: &mut HashMap<String, Node<'a>>, nonlocal_vars: &mut HashMap<String, Node<'a>>) {
        match node.kind() {
            "global_statement" => {
                self.collect_global_variables(node, source, global_vars);
            },
            "nonlocal_statement" => {
                self.collect_nonlocal_variables(node, source, nonlocal_vars);
            },
            "function_definition" | "async_function_definition" => {
                // Don't recurse into nested function definitions - they have their own scope
                return;
            },
            _ => {}
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_declarations(child, source, global_vars, nonlocal_vars);
            }
        }
    }

    fn collect_global_variables<'a>(&self, global_statement: Node<'a>, source: &str, global_vars: &mut HashMap<String, Node<'a>>) {
        for i in 0..global_statement.child_count() {
            if let Some(child) = global_statement.child(i) {
                if child.kind() == "identifier" {
                    if let Ok(var_name) = child.utf8_text(source.as_bytes()) {
                        global_vars.insert(var_name.to_string(), global_statement);
                    }
                }
            }
        }
    }

    fn collect_nonlocal_variables<'a>(&self, nonlocal_statement: Node<'a>, source: &str, nonlocal_vars: &mut HashMap<String, Node<'a>>) {
        for i in 0..nonlocal_statement.child_count() {
            if let Some(child) = nonlocal_statement.child(i) {
                if child.kind() == "identifier" {
                    if let Ok(var_name) = child.utf8_text(source.as_bytes()) {
                        nonlocal_vars.insert(var_name.to_string(), nonlocal_statement);
                    }
                }
            }
        }
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
    fn test_variable_both_global_and_nonlocal() {
        let source = r#"
def outer_function():
    x = 10
    
    def inner_function():
        global x      # Wrong: x cannot be both global and nonlocal
        nonlocal x    # Wrong: x cannot be both global and nonlocal
        x = 20
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY027"));
        assert!(violations.iter().all(|v| v.message.contains("'x'")));
    }

    #[test]
    fn test_multiple_variables_with_conflicts() {
        let source = r#"
def complex_scope():
    a = 1
    b = 2
    
    def nested():
        global a, b     # Wrong: a declared as both global and nonlocal
        nonlocal a, c   # Wrong: a conflicts, c is OK
        a = 10
        b = 20
        c = 30
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        // Should detect conflict for variable 'a' (2 violations)
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY027"));
        assert!(violations.iter().all(|v| v.message.contains("'a'")));
    }

    #[test]
    fn test_only_global_declaration() {
        let source = r#"
def only_global():
    def inner():
        global global_var
        global_var = 42
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_only_nonlocal_declaration() {
        let source = r#"
def only_nonlocal():
    local_var = 10
    
    def inner():
        nonlocal local_var
        local_var = 20
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_different_variables() {
        let source = r#"
def different_variables():
    local_var = 10
    
    def inner():
        global global_var     # Different variable - OK
        nonlocal local_var    # Different variable - OK
        global_var = 42
        local_var = 20
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_separate_function_scopes() {
        let source = r#"
def separate_functions():
    value = 42
    
    def func1():
        global value   # This is OK - separate scope
        value = 100
    
    def func2():
        nonlocal value # This is OK - separate scope
        value = 200
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        // These are separate function scopes, so no conflict
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_different_order_declarations() {
        let source = r#"
def different_order():
    value = 42
    
    def modify_value():
        nonlocal value  # First declaration
        global value    # Wrong: conflicts with nonlocal above
        value = 100
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY027"));
    }

    #[test]
    fn test_multiple_variables_same_type() {
        let source = r#"
def multiple_variables_same_type():
    a, b, c = 1, 2, 3
    
    def inner():
        nonlocal a, b, c  # All nonlocal - OK
        a = 10
        b = 20
        c = 30
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_nested_scopes_different_variables() {
        let source = r#"
def nested_scopes():
    x = 1
    
    def level1():
        y = 2
        nonlocal x  # OK: x from outer scope
        
        def level2():
            global z      # OK: z is global
            nonlocal y    # OK: y from level1 scope
            z = 3
            y = 4
            return x + y + z
        
        return level2()
    
    return level1()
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_nested_conflict() {
        let source = r#"
def complex_nested():
    outer_var = 1
    
    def middle():
        middle_var = 2
        
        def inner():
            global outer_var     # Wrong: conflicts with nonlocal
            nonlocal outer_var   # Wrong: conflicts with global above
            nonlocal middle_var  # This part is OK
            outer_var = 10
            middle_var = 20
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        // Should detect conflict for 'outer_var'
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.message.contains("'outer_var'")));
    }

    #[test]
    fn test_no_declarations() {
        let source = r#"
def no_declarations():
    x = 10
    
    def inner():
        # No global or nonlocal declarations
        print(x)  # This is fine, reads from enclosing scope
"#;
        let tree = parse_python(source);
        let rule = NonlocalAndGlobal;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }
}