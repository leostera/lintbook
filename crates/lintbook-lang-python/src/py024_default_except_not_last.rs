use tree_sitter::{Node, Tree};
use lintbook_core::{LintViolation, Rule};

pub struct DefaultExceptNotLast;

impl Rule for DefaultExceptNotLast {
    fn id(&self) -> &'static str {
        "PY024"
    }

    fn name(&self) -> &'static str {
        "default-except-not-last"
    }

    fn description(&self) -> &'static str {
        "Default except must be last"
    }

    fn explanation(&self) -> &'static str {
        "Bare except clauses (except:) and general Exception catches should be placed last, as they catch all exceptions and make subsequent except clauses unreachable."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(root_node, source, &mut violations);
        violations
    }
}

impl DefaultExceptNotLast {
    fn visit_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        // Look for try statements
        if node.kind() == "try_statement" {
            self.check_try_statement(node, source, violations);
        }

        // Recursively visit child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, violations);
            }
        }
    }

    fn check_try_statement(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        let mut except_clauses = Vec::new();

        // Collect all except clauses
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "except_clause" {
                    except_clauses.push(child);
                }
            }
        }

        // Check the ordering of except clauses
        let mut found_bare_except = false;
        let mut found_exception_catch = false;

        for (_index, except_clause) in except_clauses.iter().enumerate() {
            let exception_type = self.get_exception_type(*except_clause, source);

            match exception_type {
                ExceptionType::Bare => {
                    found_bare_except = true;
                }
                ExceptionType::Exception => {
                    // Check if Exception comes after bare except
                    if found_bare_except {
                        let start_point = except_clause.start_position();
                        violations.push(LintViolation {
                            line: start_point.row + 1,
                            column: start_point.column + 1,
                            message:
                                "Exception catch after bare except. Bare except should be last."
                                    .to_string(),
                            lint_id: self.id().to_string(),
                            lint_name: self.name().to_string(),
                        });
                    } else {
                        found_exception_catch = true;
                    }
                }
                ExceptionType::Specific => {
                    // Check if this specific exception comes after a bare except
                    if found_bare_except {
                        let start_point = except_clause.start_position();
                        violations.push(LintViolation {
                            line: start_point.row + 1,
                            column: start_point.column + 1,
                            message: "Specific except clause after bare except. Bare except should be last.".to_string(),
                            lint_id: self.id().to_string(),
                            lint_name: self.name().to_string(),
                        });
                    }
                    // Check if this specific exception comes after Exception catch
                    else if found_exception_catch {
                        let start_point = except_clause.start_position();
                        violations.push(LintViolation {
                            line: start_point.row + 1,
                            column: start_point.column + 1,
                            message: "Specific except clause after Exception. Exception should be last (before bare except).".to_string(),
                            lint_id: self.id().to_string(),
                            lint_name: self.name().to_string(),
                        });
                    }
                }
            }
        }
    }

    fn get_exception_type(&self, except_clause: Node, source: &str) -> ExceptionType {
        for i in 0..except_clause.child_count() {
            if let Some(child) = except_clause.child(i) {
                if child.kind() == "block" {
                    break;
                }

                let result = self.find_exception_identifier(child, source);
                if result != ExceptionType::Bare {
                    return result;
                }
            }
        }

        ExceptionType::Bare
    }

    fn find_exception_identifier(&self, node: Node, source: &str) -> ExceptionType {
        // Check if this node is an identifier
        if node.kind() == "identifier" {
            let identifier = node.utf8_text(source.as_bytes()).unwrap_or("");
            if identifier == "Exception" {
                return ExceptionType::Exception;
            } else if identifier == "except" {
                // Skip the "except" keyword
            } else {
                return ExceptionType::Specific;
            }
        }

        // Check children recursively
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let result = self.find_exception_identifier(child, source);
                if result != ExceptionType::Bare {
                    return result;
                }
            }
        }

        // If no exception type found, it's a bare except
        ExceptionType::Bare
    }
}

#[derive(Debug, PartialEq)]
enum ExceptionType {
    Bare,      // except:
    Exception, // except Exception:
    Specific,  // except ValueError:, except (ValueError, TypeError):, etc.
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_python(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_bare_except_not_last() {
        let source = r#"
try:
    risky_operation()
except:  # Bare except should be last
    handle_general_error()
except ValueError:  # Specific except after bare except - wrong
    handle_value_error()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY024");
        assert!(violations[0]
            .message
            .contains("Specific except clause after bare except"));
    }

    #[test]
    fn test_exception_after_bare_except() {
        let source = r#"
try:
    operation()
except:  # Bare except should be last
    log_error()
except Exception as e:  # Exception after bare except
    handle_exception(e)
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY024");
        assert!(violations[0]
            .message
            .contains("Exception catch after bare except"));
    }

    #[test]
    fn test_specific_after_exception() {
        let source = r#"
try:
    another_operation()
except Exception:  # General exception
    handle_exception()
except TypeError:  # Specific except after general exception - wrong
    handle_type_error()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].lint_id, "PY024");
        assert!(violations[0]
            .message
            .contains("Specific except clause after Exception"));
    }

    #[test]
    fn test_correct_order() {
        let source = r#"
try:
    risky_operation()
except ValueError:  # Most specific first
    handle_value_error()
except TypeError:  # Another specific exception
    handle_type_error()
except Exception:  # General exception
    handle_exception()
except:  # Bare except last
    handle_any_error()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_only_specific_exceptions() {
        let source = r#"
try:
    specific_operation()
except ValueError:
    handle_value_error()
except TypeError:
    handle_type_error()
except KeyError:
    handle_key_error()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_only_bare_except() {
        let source = r#"
try:
    simple_operation()
except:
    handle_any_error()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_only_exception() {
        let source = r#"
try:
    operation()
except Exception as e:
    handle_exception(e)
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_violations() {
        let source = r#"
try:
    complex_operation()
except:  # Bare except should be last
    handle_any_error()
except ValueError:  # Specific after bare except
    handle_value_error()
except Exception:  # Exception after bare except
    handle_exception()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.lint_id == "PY024"));
    }

    #[test]
    fn test_tuple_exceptions() {
        let source = r#"
try:
    operation()
except (ValueError, TypeError):  # Tuple is specific
    handle_multiple_types()
except Exception:  # General exception last - correct
    handle_exception()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_nested_try_except() {
        let source = r#"
try:
    outer_operation()
    try:
        inner_operation()
    except ValueError:  # Inner try block - fine
        handle_inner_value_error()
    except:  # Inner bare except last - fine
        handle_inner_any_error()
except TypeError:  # Outer try block - fine
    handle_outer_type_error()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_complex_wrong_order() {
        let source = r#"
try:
    complex_scenario()
except (ValueError, TypeError):  # Specific exceptions
    handle_multiple()
except:  # Bare except - should be last
    handle_any()
except Exception:  # Exception after bare except - wrong
    handle_exception()
except RuntimeError:  # Specific after bare except - wrong
    handle_runtime_error()
"#;
        let tree = parse_python(source);
        let rule = DefaultExceptNotLast;
        let violations = rule.check(&tree, source);

        assert_eq!(violations.len(), 2);
    }
}
