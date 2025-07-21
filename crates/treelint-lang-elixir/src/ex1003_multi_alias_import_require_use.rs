use tree_sitter::{Node, Tree};
use treelint_core::{LintViolation, Rule};

pub struct MultiAliasImportRequireUse;

impl Rule for MultiAliasImportRequireUse {
    fn id(&self) -> &'static str {
        "EX1003"
    }

    fn name(&self) -> &'static str {
        "multi_alias_import_require_use"
    }

    fn description(&self) -> &'static str {
        "Consistent style for multi-alias vs single-alias syntax"
    }

    fn explanation(&self) -> &'static str {
        "When aliasing multiple modules from the same namespace, prefer multi-alias syntax \
        over multiple single aliases. Use `alias MyApp.{User, Post, Comment}` instead of \
        multiple `alias MyApp.User`, `alias MyApp.Post`, etc. This makes the code more \
        concise and clearly shows the relationship between the modules."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        // Find all alias/import/require/use statements and group them by base module
        let statements = self.collect_statements(tree.root_node(), source);
        
        // Group statements by base module and check for multi-alias opportunities
        for (base_module, statement_groups) in self.group_by_base_module(&statements) {
            if statement_groups.len() > 1 {
                // Found multiple statements for the same base module
                let first_statement = &statement_groups[0];
                violations.push(LintViolation {
                    line: first_statement.line,
                    column: first_statement.column,
                    message: format!(
                        "Consider using multi-{} syntax: `{} {}.{{{}}}` instead of multiple statements",
                        first_statement.statement_type,
                        first_statement.statement_type,
                        base_module,
                        statement_groups.iter()
                            .map(|s| s.alias_name.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
        
        violations
    }
}

#[derive(Debug, Clone)]
struct AliasStatement {
    statement_type: String, // "alias", "import", "require", "use"
    base_module: String,    // "MyApp"
    alias_name: String,     // "User"
    line: usize,
    column: usize,
}

impl MultiAliasImportRequireUse {
    fn collect_statements(&self, node: Node, source: &str) -> Vec<AliasStatement> {
        let mut statements = Vec::new();
        self.traverse_and_collect(node, source, &mut statements);
        statements
    }
    
    fn traverse_and_collect(&self, node: Node, source: &str, statements: &mut Vec<AliasStatement>) {
        // Check if this is an alias/import/require/use call
        if node.kind() == "call" {
            if let Some(statement) = self.parse_statement(node, source) {
                statements.push(statement);
            }
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.traverse_and_collect(child, source, statements);
            }
        }
    }
    
    fn parse_statement(&self, node: Node, source: &str) -> Option<AliasStatement> {
        // Check if this is a call node with appropriate function name
        if let Some(function_node) = node.child(0) {
            if function_node.kind() == "identifier" {
                let function_name = &source[function_node.start_byte()..function_node.end_byte()];
                
                if matches!(function_name, "alias" | "import" | "require" | "use") {
                    // Find the arguments node
                    if let Some(args_node) = node.child(1) {
                        if args_node.kind() == "arguments" {
                            return self.parse_module_argument(args_node, source, function_name);
                        }
                    }
                }
            }
        }
        
        None
    }
    
    fn parse_module_argument(&self, args_node: Node, source: &str, statement_type: &str) -> Option<AliasStatement> {
        // Look for the first argument which should be a module reference
        for i in 0..args_node.child_count() {
            if let Some(arg) = args_node.child(i) {
                if let Some(module_path) = self.extract_module_path(arg, source) {
                    // Split module path into base and alias name
                    if let Some((base_module, alias_name)) = self.split_module_path(&module_path) {
                        let position = arg.start_position();
                        return Some(AliasStatement {
                            statement_type: statement_type.to_string(),
                            base_module,
                            alias_name,
                            line: position.row + 1,
                            column: position.column + 1,
                        });
                    }
                }
            }
        }
        
        None
    }
    
    fn extract_module_path(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "alias" => {
                // Handle dot notation like MyApp.User - extract the full text
                Some(source[node.start_byte()..node.end_byte()].to_string())
            }
            "identifier" => {
                // Handle simple module names
                Some(source[node.start_byte()..node.end_byte()].to_string())
            }
            _ => None,
        }
    }
    
    
    fn split_module_path(&self, module_path: &str) -> Option<(String, String)> {
        // Split "MyApp.User.Profile" into ("MyApp", "User.Profile")
        // or "MyApp.User" into ("MyApp", "User")
        if let Some(dot_pos) = module_path.find('.') {
            let base = module_path[..dot_pos].to_string();
            let rest = module_path[dot_pos + 1..].to_string();
            // For alias name, use just the last part
            let alias_name = rest.split('.').last().unwrap_or(&rest).to_string();
            Some((base, alias_name))
        } else {
            None // Single module name, can't be multi-aliased
        }
    }
    
    fn group_by_base_module(&self, statements: &[AliasStatement]) -> std::collections::HashMap<String, Vec<AliasStatement>> {
        let mut groups = std::collections::HashMap::new();
        
        for statement in statements {
            groups.entry(statement.base_module.clone())
                .or_insert_with(Vec::new)
                .push(statement.clone());
        }
        
        // Only return groups that have more than one statement of the same type
        groups.into_iter()
            .filter(|(_, statements)| {
                // Group by statement type within each base module
                let mut type_groups = std::collections::HashMap::new();
                for stmt in statements {
                    type_groups.entry(&stmt.statement_type)
                        .or_insert_with(Vec::new)
                        .push(stmt);
                }
                // Check if any statement type has multiple entries
                type_groups.values().any(|group| group.len() > 1)
            })
            .map(|(base, statements)| {
                // Return only the statements that are part of multi-statement groups
                let mut type_groups = std::collections::HashMap::new();
                for stmt in statements {
                    type_groups.entry(stmt.statement_type.clone())
                        .or_insert_with(Vec::new)
                        .push(stmt);
                }
                let filtered_statements: Vec<AliasStatement> = type_groups
                    .into_values()
                    .filter(|group| group.len() > 1)
                    .flatten()
                    .collect();
                (base, filtered_statements)
            })
            .filter(|(_, statements)| !statements.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_elixir_code(code: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_detects_multiple_aliases_from_same_module() {
        let code = r#"
defmodule MyModule do
  alias MyApp.User
  alias MyApp.Post
  alias MyApp.Comment
  
  def something do
    User.create()
    Post.publish()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = MultiAliasImportRequireUse;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].lint_id, "EX1003");
        assert!(violations[0].message.contains("multi-alias syntax"));
        assert!(violations[0].message.contains("MyApp.{User, Post, Comment}"));
    }

    #[test]
    fn test_allows_single_aliases() {
        let code = r#"
defmodule MyModule do
  alias MyApp.User
  alias OtherApp.Post
  alias ThirdApp.Comment
  
  def something do
    User.create()
    Post.publish()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = MultiAliasImportRequireUse;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_existing_multi_alias() {
        let code = r#"
defmodule MyModule do
  alias MyApp.{User, Post, Comment}
  
  def something do
    User.create()
    Post.publish()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = MultiAliasImportRequireUse;
        let violations = lint.check(&tree, code);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_multiple_imports() {
        let code = r#"
defmodule MyModule do
  import MyApp.Helpers
  import MyApp.Utils
  import MyApp.Constants
  
  def something do
    helper_function()
    utility_function()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = MultiAliasImportRequireUse;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("multi-import syntax"));
    }

    #[test]
    fn test_handles_mixed_statement_types() {
        let code = r#"
defmodule MyModule do
  alias MyApp.User
  import MyApp.Post
  alias MyApp.Comment
  
  def something do
    User.create()
    publish_post()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = MultiAliasImportRequireUse;
        let violations = lint.check(&tree, code);

        // Should detect the two alias statements but not mix with import
        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("multi-alias syntax"));
        assert!(violations[0].message.contains("MyApp.{User, Comment}"));
    }

    #[test]
    fn test_nested_modules() {
        let code = r#"
defmodule MyModule do
  alias MyApp.Auth.User
  alias MyApp.Auth.Session
  alias MyApp.Auth.Token
  
  def something do
    User.authenticate()
    Session.create()
  end
end
"#;

        let tree = parse_elixir_code(code);
        let lint = MultiAliasImportRequireUse;
        let violations = lint.check(&tree, code);

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("MyApp.{User, Session, Token}"));
    }
}