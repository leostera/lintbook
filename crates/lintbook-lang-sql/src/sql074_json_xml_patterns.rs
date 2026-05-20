use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct JsonXmlPatterns;

impl Rule for JsonXmlPatterns {
    fn id(&self) -> &'static str {
        "SQL074"
    }

    fn name(&self) -> &'static str {
        "json-xml-patterns"
    }

    fn description(&self) -> &'static str {
        "Optimize JSON and XML data handling patterns"
    }

    fn explanation(&self) -> &'static str {
        "JSON and XML operations can be resource-intensive and require special considerations
        for indexing and querying. This rule identifies optimization opportunities."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        self.check_json_xml_patterns(tree.root_node(), source, &mut violations);

        violations
    }
}

impl JsonXmlPatterns {
    fn check_json_xml_patterns(
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

            // Check JSON operations
            self.check_json_operations(&lower_line, line_idx, node, violations);

            // Check XML operations
            self.check_xml_operations(&lower_line, line_idx, node, violations);

            // Check indexing opportunities
            self.check_indexing_opportunities(&lower_line, line_idx, node, violations);

            // Check performance patterns
            self.check_performance_patterns(&lower_line, line_idx, node, violations);
        }

        // Recursively check child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.check_json_xml_patterns(child, source, violations);
            }
        }
    }

    fn check_json_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for JSON_VALUE usage
        if lower_line.contains("json_value(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON_VALUE extraction. Consider computed columns or indexing for frequently accessed JSON paths".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for JSON_QUERY usage
        if lower_line.contains("json_query(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON_QUERY usage. Ensure JSON path expressions are optimized and consider result caching".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for OPENJSON without schema
        if lower_line.contains("openjson(") && !lower_line.contains("with (") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "OPENJSON without explicit schema. Specify WITH clause for better performance and type safety".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for JSON path expressions in WHERE clauses
        if lower_line.contains("json_value") && lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON_VALUE in WHERE clause. Consider computed columns with indexes for better query performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for complex JSON path expressions
        if lower_line.contains("$") && (lower_line.contains("[") || lower_line.contains("*")) &&
           (lower_line.contains("json_value") || lower_line.contains("json_query")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex JSON path expression with arrays or wildcards. May impact performance with large JSON documents".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for JSON modification functions
        if lower_line.contains("json_modify(") || lower_line.contains("json_set(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON modification in database. Consider if application-level JSON handling would be more appropriate".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_xml_operations(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for XML column queries
        if lower_line.contains(".query(") || lower_line.contains(".value(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "XML query/value method. Consider XML indexes for frequently accessed XPath expressions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for XPath expressions in WHERE clauses
        if lower_line.contains(".exist(") && lower_line.contains("where") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "XML exist() method in WHERE clause. Ensure appropriate XML indexes exist for the XPath expression".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for XML modify operations
        if lower_line.contains(".modify(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "XML modify operation. These can be expensive on large XML documents - consider performance impact".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for OPENXML usage
        if lower_line.contains("openxml(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "OPENXML usage detected. Consider newer XML methods or JSON alternatives for better performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for FOR XML without optimization
        if lower_line.contains("for xml") && !lower_line.contains("type") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "FOR XML without TYPE directive. Add TYPE for better performance with large result sets".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for complex XPath expressions
        if lower_line.contains("//") || (lower_line.contains("[") && lower_line.contains("@")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Complex XPath expression with descendant axis or predicates. May impact performance on large XML".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_indexing_opportunities(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for frequently accessed JSON properties
        if lower_line.contains("json_value") && lower_line.contains("$.") &&
           !lower_line.contains("computed") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Frequent JSON property access. Consider computed columns with indexes for commonly accessed paths".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for JSON arrays without proper indexing
        if lower_line.contains("json_query") && lower_line.contains("$[") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON array access detected. Consider if array elements need separate indexing strategy".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for XML without indexing
        if (lower_line.contains(".value(") || lower_line.contains(".query(")) &&
           !lower_line.contains("index") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "XML querying without mention of indexes. Consider XML indexes for frequently accessed XPath expressions".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }

    fn check_performance_patterns(
        &self,
        lower_line: &str,
        line_idx: usize,
        node: tree_sitter::Node,
        violations: &mut Vec<LintViolation>,
    ) {
        // Check for JSON/XML in GROUP BY
        if (lower_line.contains("json_value") || lower_line.contains(".value(")) &&
           lower_line.contains("group by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON/XML extraction in GROUP BY. Consider computed columns to avoid repeated parsing".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for JSON/XML in ORDER BY
        if (lower_line.contains("json_value") || lower_line.contains(".value(")) &&
           lower_line.contains("order by") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON/XML extraction in ORDER BY. Consider computed columns with indexes for sorting".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for large JSON/XML document handling
        if (lower_line.contains("json") || lower_line.contains("xml")) &&
           (lower_line.contains("varchar(max)") || lower_line.contains("nvarchar(max)")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Large JSON/XML document storage. Consider document size limits and compression options".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for JSON/XML conversion functions
        if lower_line.contains("for json") || lower_line.contains("for xml") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON/XML serialization in database. Consider if application-level serialization would be more efficient".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for nested JSON/XML operations
        if (lower_line.contains("json_value(json_query") ||
            lower_line.contains(".value(") && lower_line.contains(".query(")) {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "Nested JSON/XML operations detected. Consider flattening or caching intermediate results".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }

        // Check for JSON/XML validation
        if lower_line.contains("isjson(") || lower_line.contains("try_parse(") {
            let start_pos = node.start_position();
            violations.push(LintViolation {
                line: start_pos.row + line_idx + 1,
                column: start_pos.column + 1,
                message: "JSON/XML validation in query. Consider CHECK constraints or validation at application level for performance".to_string(),
                lint_name: self.name().to_string(),
                lint_id: self.id().to_string(),
            });
        }
    }
}