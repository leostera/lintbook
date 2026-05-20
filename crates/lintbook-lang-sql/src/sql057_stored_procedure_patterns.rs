use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct StoredProcedurePatterns;

impl Rule for StoredProcedurePatterns {
    fn id(&self) -> &'static str {
        "SQL057"
    }

    fn name(&self) -> &'static str {
        "stored-procedure-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce stored procedure and function best practices"
    }

    fn explanation(&self) -> &'static str {
        "Stored procedures and functions should follow best practices for maintainability,
        performance, and security. This includes parameter validation, proper naming, and appropriate usage patterns."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_procedure_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl StoredProcedurePatterns {
    fn check_procedure_patterns(
        &self,
        node: tree_sitter::Node,
        source: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let node_text = &source[node.start_byte()..node.end_byte()];
        let lines: Vec<&str> = node_text.split('\n').collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let lower_line = line.to_lowercase();

            // Skip comments
            if line.trim().starts_with("--") {
                continue;
            }

            // Check stored procedure definitions
            if lower_line.contains("create procedure") || lower_line.contains("create proc") {
                self.check_procedure_definition(&lower_line, line_idx, node, violations);
                self.check_procedure_naming(line, line_idx, node, violations);
            }

            // Check function definitions
            if lower_line.contains("create function") {
                self.check_function_definition(&lower_line, line_idx, node, violations);
                self.check_function_naming(line, line_idx, node, violations);
            }

            // Check parameter patterns
            if lower_line.contains("@")
                && (lower_line.contains("varchar")
                    || lower_line.contains("int")
                    || lower_line.contains("datetime"))
            {
                self.check_parameter_patterns(&lower_line, line_idx, node, violations);
            }

            // Check return patterns
            if lower_line.contains("return") {
                self.check_return_patterns(&lower_line, line_idx, node, violations);
            }

            // Check procedure calls
            if lower_line.contains("exec ") || lower_line.contains("execute ") {
                self.check_procedure_calls(&lower_line, line_idx, node, violations);
            }
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_procedure_patterns(child, source, violations);
            }
        }
    }

    fn check_procedure_definition(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for procedures without parameters (potential code smell)
        if lower_line.contains("create procedure") && !lower_line.contains("@") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Stored procedure without parameters. Consider if this logic belongs in a view or function instead".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for WITH RECOMPILE (performance impact)
        if lower_line.contains("with recompile") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "WITH RECOMPILE prevents plan caching. Use only when necessary for parameter sniffing issues".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for WITH ENCRYPTION (debugging difficulty)
        if lower_line.contains("with encryption") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "WITH ENCRYPTION makes procedure code unreadable. Consider security implications vs maintainability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_procedure_naming(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();

        if lower_line.contains("create procedure") {
            // Extract procedure name
            if let Some(proc_start) = lower_line.find("create procedure") {
                let after_proc = &line[proc_start + 16..].trim_start();
                if let Some(space_pos) = after_proc.find(' ') {
                    let proc_name = &after_proc[..space_pos].trim();

                    // Check for sp_ prefix (reserved for system procedures)
                    if proc_name.starts_with("sp_") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + proc_start + 16,
                            message: format!(
                                "Procedure '{}' uses sp_ prefix reserved for system procedures. Use different naming convention",
                                proc_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }

                    // Check for Hungarian notation
                    if proc_name.starts_with("proc_") || proc_name.starts_with("p_") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + proc_start + 16,
                            message: format!(
                                "Procedure '{}' uses Hungarian notation. Use descriptive names without type prefixes",
                                proc_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }

                    // Check for very long names (readability)
                    if proc_name.len() > 50 {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + proc_start + 16,
                            message: format!(
                                "Procedure name '{}' is very long ({} chars). Consider shorter, more concise naming",
                                proc_name, proc_name.len()
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }

    fn check_function_definition(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for scalar functions without RETURNS specification
        if lower_line.contains("create function") && !lower_line.contains("returns") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Function without RETURNS clause. Specify return type for clarity"
                    .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for functions returning VARCHAR without length
        if lower_line.contains("returns varchar") && !lower_line.contains("returns varchar(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Function returns VARCHAR without length specification. Specify length for predictability".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for table-valued functions (performance consideration)
        if lower_line.contains("returns table") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Table-valued function detected. Consider inline table-valued functions for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_function_naming(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        let lower_line = line.to_lowercase();

        if lower_line.contains("create function") {
            // Extract function name
            if let Some(func_start) = lower_line.find("create function") {
                let after_func = &line[func_start + 15..].trim_start();
                if let Some(paren_pos) = after_func.find('(') {
                    let func_name = &after_func[..paren_pos].trim();

                    // Check for fn_ prefix (common convention)
                    if !func_name.starts_with("fn_") && !func_name.starts_with("func_") {
                        let start_pos = node.start_position();
                        violations.push(LintViolation {
                            line: start_pos.row + line_idx + 1,
                            column: start_pos.column + func_start + 15,
                            message: format!(
                                "Function '{}' doesn't follow naming convention. Consider fn_ or func_ prefix",
                                func_name
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }
    }

    fn check_parameter_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for parameters without default values
        if lower_line.contains("@") && !lower_line.contains("=") && !lower_line.contains("output") {
            // Simple heuristic for required parameters
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Parameter without default value. Consider providing default values for optional parameters".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for output parameters (can complicate testing)
        if lower_line.contains("output") || lower_line.contains("out") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "OUTPUT parameter detected. Consider returning result sets or return values for simpler interface".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for large VARCHAR parameters (performance)
        if lower_line.contains("varchar(max)") || lower_line.contains("varchar(8000)") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Large VARCHAR parameter may impact performance. Consider alternative data passing methods for large data".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_return_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for return without value (potential bug)
        if lower_line.trim() == "return" {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "RETURN without value. Specify return value or use RETURN 0 for success"
                    .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for magic return numbers
        if lower_line.contains("return -1") || lower_line.contains("return 99") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Magic number in RETURN. Use meaningful constants or document the meaning of return codes".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for inconsistent return patterns
        if lower_line.contains("return") && !lower_line.contains("return 0") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Non-zero return value. Ensure consistent return code conventions (0 = success, non-zero = error)".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_procedure_calls(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for procedure calls without parameter names
        if (lower_line.contains("exec ") || lower_line.contains("execute "))
            && lower_line.contains("'")
        {
            if !lower_line.contains("@") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Procedure call with positional parameters. Use named parameters for clarity and maintainability".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for dynamic procedure calls (security risk)
        if lower_line.contains("exec (") || lower_line.contains("execute (") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Dynamic procedure execution. Ensure proper input validation to prevent SQL injection".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}
