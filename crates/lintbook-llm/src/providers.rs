use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lintbook_config::LlmProviderConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync + std::fmt::Debug {
    async fn generate_query(&self, language: &str, match_prompt: &str) -> Result<String>;
    async fn validate_match(&self, code: &str, error: &str) -> Result<String>;
}

pub fn create_provider(config: &LlmProviderConfig) -> Result<Box<dyn LlmProvider>> {
    match config.provider.as_str() {
        "mock" => Ok(Box::new(MockProvider)),
        provider => Err(anyhow!("Unsupported LLM provider: {}", provider)),
    }
}

#[derive(Debug)]
struct MockProvider;

#[async_trait]
impl LlmProvider for MockProvider {
    async fn generate_query(&self, _language: &str, _match_prompt: &str) -> Result<String> {
        Ok("(module) @root".to_string())
    }

    async fn validate_match(&self, _code: &str, _error: &str) -> Result<String> {
        Ok("false".to_string())
    }
}
