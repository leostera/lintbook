use anyhow::Result;
use lintbook_lang::Grammar;

#[derive(Debug, Default)]
pub struct QueryGenerator;

impl QueryGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_query_from_response(&self, response: &str) -> String {
        if let Some(start) = response.find("```") {
            let rest = &response[start + 3..];
            let rest = rest
                .strip_prefix("query")
                .or_else(|| rest.strip_prefix("scheme"))
                .unwrap_or(rest);

            if let Some(end) = rest.find("```") {
                return rest[..end].trim().to_string();
            }
        }

        response.trim().to_string()
    }

    pub fn validate_query(&self, language: &str, query: &str) -> Result<()> {
        let grammar = Grammar::from_name(language)?;
        let language = grammar.to_tree_sitter_language()?;
        tree_sitter::Query::new(&language, query)?;
        Ok(())
    }
}
