use std::collections::HashMap;
use tree_sitter::Tree;
use lintbook_core::{LintViolation, Rule};

pub struct LineEndings;

impl Rule for LineEndings {
    fn id(&self) -> &'static str {
        "EX1002"
    }

    fn name(&self) -> &'static str {
        "line-endings"
    }

    fn description(&self) -> &'static str {
        "Consistent line endings across all files"
    }

    fn explanation(&self) -> &'static str {
        "All files should use consistent line endings - either Unix-style (LF: \\n) or \
        Windows-style (CRLF: \\r\\n). Mixed line endings can cause issues with version \
        control systems and development tools across different platforms."
    }

    fn check(&self, _tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let line_ending_stats = self.analyze_line_endings(source);

        if let Some(dominant_style) = self.find_dominant_line_ending(&line_ending_stats) {
            let lines: Vec<&str> = source.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                let line_with_ending = if line_num < lines.len() - 1 {
                    // Get the actual line with its ending from the source
                    self.get_line_with_ending(source, line_num)
                } else {
                    // Last line may not have an ending
                    continue;
                };

                let detected_ending = self.detect_line_ending(&line_with_ending);

                if let Some(ending) = detected_ending {
                    if ending != dominant_style {
                        violations.push(LintViolation {
                            line: line_num + 1,
                            column: line.len() + 1,
                            message: format!(
                                "Line ending style '{}' does not match dominant style '{}'. \
                                Expected {} line endings.",
                                self.ending_name(&ending),
                                self.ending_name(&dominant_style),
                                self.ending_name(&dominant_style)
                            ),
                            lint_name: self.name().to_string(),
                            lint_id: self.id().to_string(),
                        });
                    }
                }
            }
        }

        violations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LineEndingType {
    Unix,    // LF: \n
    Windows, // CRLF: \r\n
}

impl LineEndings {
    fn analyze_line_endings(&self, source: &str) -> HashMap<LineEndingType, usize> {
        let mut stats = HashMap::new();
        let lines = source.split_inclusive('\n');

        for line in lines {
            if line.ends_with('\n') {
                if line.ends_with("\r\n") {
                    *stats.entry(LineEndingType::Windows).or_insert(0) += 1;
                } else {
                    *stats.entry(LineEndingType::Unix).or_insert(0) += 1;
                }
            }
        }

        stats
    }

    fn find_dominant_line_ending(
        &self,
        stats: &HashMap<LineEndingType, usize>,
    ) -> Option<LineEndingType> {
        stats
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ending_type, _)| *ending_type)
    }

    fn get_line_with_ending(&self, source: &str, line_num: usize) -> String {
        let lines: Vec<&str> = source.split_inclusive('\n').collect();
        if line_num < lines.len() {
            lines[line_num].to_string()
        } else {
            String::new()
        }
    }

    fn detect_line_ending(&self, line: &str) -> Option<LineEndingType> {
        if line.ends_with("\r\n") {
            Some(LineEndingType::Windows)
        } else if line.ends_with('\n') {
            Some(LineEndingType::Unix)
        } else {
            None
        }
    }

    fn ending_name(&self, ending: &LineEndingType) -> &'static str {
        match ending {
            LineEndingType::Unix => "Unix (LF)",
            LineEndingType::Windows => "Windows (CRLF)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_unix_line_endings() {
        let code = "defmodule Test do\n  def hello do\n    :world\n  end\nend\n";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();

        let lint = LineEndings;
        let violations = lint.check(&tree, code);

        // Should have no violations since all lines use Unix endings
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_consistent_windows_line_endings() {
        let code = "defmodule Test do\r\n  def hello do\r\n    :world\r\n  end\r\nend\r\n";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();

        let lint = LineEndings;
        let violations = lint.check(&tree, code);

        // Should have no violations since all lines use Windows endings consistently
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mixed_line_endings() {
        let code = "defmodule Test do\n  def hello do\r\n    :world\n  end\r\nend\n";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();

        let lint = LineEndings;
        let violations = lint.check(&tree, code);

        // Should detect mixed line endings
        assert!(violations.len() > 0);
        assert_eq!(violations[0].lint_id, "EX1002");
    }

    #[test]
    fn test_no_line_endings() {
        let code = "defmodule Test do end";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_elixir::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();

        let lint = LineEndings;
        let violations = lint.check(&tree, code);

        // Should have no violations since there are no line endings to check
        assert_eq!(violations.len(), 0);
    }
}
