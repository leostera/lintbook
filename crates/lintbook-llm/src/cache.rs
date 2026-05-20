use anyhow::Result;
use sha2::{Digest, Sha256};
use std::future::Future;
use std::path::PathBuf;

#[derive(Debug)]
pub struct LlmCache {
    cache_dir: PathBuf,
}

impl LlmCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    pub async fn get_or_compute<F, Fut>(&self, key: &str, compute: F) -> Result<String>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String>>,
    {
        let path = self.cache_dir.join(cache_key(key));
        if path.exists() {
            return Ok(std::fs::read_to_string(path)?);
        }

        let value = compute().await?;
        std::fs::write(path, &value)?;
        Ok(value)
    }
}

fn cache_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{}.txt", hex::encode(hasher.finalize()))
}
