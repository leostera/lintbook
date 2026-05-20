use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct TemporalPatterns;

impl Rule for TemporalPatterns {
    fn id(&self) -> &'static str {
        "SQL055"
    }

    fn name(&self) -> &'static str {
        "temporal-patterns"
    }

    fn description(&self) -> &'static str {
        "Enforce best practices for date, time, and temporal data handling"
    }

    fn explanation(&self) -> &'static str {
        "Proper handling of temporal data is crucial for data integrity and query performance.
        This rule checks for timezone awareness, appropriate data types, and common temporal anti-patterns."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_temporal_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl TemporalPatterns {
    fn check_temporal_patterns(
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

            // Check date/time data types and usage
            self.check_datetime_types(&lower_line, line_idx, node, violations);

            // Check for timezone issues
            self.check_timezone_awareness(&lower_line, line_idx, node, violations);

            // Check date arithmetic and functions
            self.check_date_arithmetic(&lower_line, line_idx, node, violations);

            // Check date formatting and parsing
            self.check_date_formatting(&lower_line, line_idx, node, violations);

            // Check temporal range queries
            self.check_temporal_ranges(&lower_line, line_idx, node, violations);

            // Check for hardcoded dates
            self.check_hardcoded_dates(line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_temporal_patterns(child, source, violations);
            }
        }
    }

    fn check_datetime_types(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for deprecated datetime types
        if lower_line.contains("create table") || lower_line.contains("alter table") {
            // Prefer DATETIME2 over DATETIME in SQL Server
            if lower_line.contains(" datetime ") && !lower_line.contains(" datetime2") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Consider using DATETIME2 instead of DATETIME for better precision and range (SQL Server)".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Check for TIMESTAMP without timezone
            if lower_line.contains(" timestamp ") && !lower_line.contains("with time zone") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "TIMESTAMP without timezone. Consider TIMESTAMP WITH TIME ZONE for better timezone handling".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }

            // Check for storing dates as strings
            if (lower_line.contains("date") || lower_line.contains("time"))
                && lower_line.contains("varchar")
            {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Storing date/time as VARCHAR. Use proper temporal data types (DATE, TIME, DATETIME) instead".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_timezone_awareness(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for timezone-naive operations
        if lower_line.contains("getdate()") || lower_line.contains("now()") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Using timezone-naive date function. Consider timezone-aware functions like GETUTCDATE() or specify timezone explicitly".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for timezone conversion functions
        if lower_line.contains("convert_tz") || lower_line.contains("at time zone") {
            // This is good - just noting it's timezone-aware
        } else if lower_line.contains("created_at") || lower_line.contains("updated_at") {
            if !lower_line.contains("utc") && !lower_line.contains("timezone") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Temporal column without timezone specification. Consider storing in UTC or adding timezone information".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_date_arithmetic(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for date arithmetic using string operations
        if lower_line.contains("concat")
            && (lower_line.contains("year")
                || lower_line.contains("month")
                || lower_line.contains("day"))
        {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Date arithmetic using string concatenation. Use proper date functions like DATEADD, DATEDIFF, or interval arithmetic".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for manual date calculations
        if lower_line.contains(" + 1") || lower_line.contains(" - 1") {
            if lower_line.contains("date") || lower_line.contains("time") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Manual date arithmetic with +/- numbers. Use DATEADD or INTERVAL for proper date calculations".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for leap year considerations
        if lower_line.contains("february") && lower_line.contains("29") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Hardcoded February 29th. Ensure leap year handling is correct"
                    .to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_date_formatting(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for ambiguous date formats
        let ambiguous_formats = ["mm/dd/yyyy", "dd/mm/yyyy", "yy-mm-dd"];
        for format in ambiguous_formats.iter() {
            if lower_line.contains(format) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Ambiguous date format '{}'. Use ISO 8601 format (YYYY-MM-DD) for clarity",
                        format
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for FORMAT function usage (can be slow)
        if lower_line.contains("format(")
            && (lower_line.contains("date") || lower_line.contains("time"))
        {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "FORMAT function can be slow for date formatting. Consider using CONVERT or application-level formatting".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_temporal_ranges(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for inefficient date range queries
        if lower_line.contains(" where ")
            && (lower_line.contains("year(")
                || lower_line.contains("month(")
                || lower_line.contains("day("))
        {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Date function in WHERE clause prevents index usage. Use date ranges instead (date >= '2023-01-01' AND date < '2024-01-01')".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for proper date range queries
        if lower_line.contains(" between ")
            && (lower_line.contains("date") || lower_line.contains("time"))
        {
            if !lower_line.contains("'") && !lower_line.contains("\"") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Date range comparison without quoted literals. Ensure proper date format and consider timezone implications".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Check for open-ended date ranges
        if lower_line.contains(">=") && (lower_line.contains("date") || lower_line.contains("time"))
        {
            if !lower_line.contains("and") && !lower_line.contains("<") {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: "Open-ended date range. Consider adding upper bound for better performance and query predictability".to_string(),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }

    fn check_hardcoded_dates(
        &self,
        line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Look for hardcoded dates in queries
        let date_patterns = [
            r"'20\d{2}-\d{2}-\d{2}'",
            r"'20\d{2}/\d{2}/\d{2}'",
            r"'\d{2}/\d{2}/20\d{2}'",
        ];

        for _pattern in date_patterns.iter() {
            // Simple pattern matching for demo - in real implementation would use regex
            if line.contains("'20") && (line.contains("-") || line.contains("/")) {
                if line.contains(" where ") || line.contains(" and ") || line.contains(" or ") {
                    let start_pos = node.start_position();
                    violations.push(LintViolation {
                        line: start_pos.row + line_idx + 1,
                        column: start_pos.column + 1,
                        message: "Hardcoded date in query. Consider using parameters or date functions for maintainability".to_string(),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                    break;
                }
            }
        }

        // Check for magic date values
        let magic_dates = ["'1900-01-01'", "'9999-12-31'", "'1970-01-01'"];
        for magic_date in magic_dates.iter() {
            if line.contains(magic_date) {
                let start_pos = node.start_position();
                violations.push(LintViolation {
                    line: start_pos.row + line_idx + 1,
                    column: start_pos.column + 1,
                    message: format!(
                        "Magic date {} detected. Use NULL or proper sentinel values, or document the meaning",
                        magic_date
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }
    }
}
