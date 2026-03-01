use crate::domain::repository::EmbeddingRepository;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaEmbedding {
    client: Client,
    endpoint: String,
    model: String,
}

impl OllamaEmbedding {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            model,
        }
    }
}

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

#[async_trait::async_trait]
impl EmbeddingRepository for OllamaEmbedding {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let req_body = EmbeddingRequest {
            model: &self.model,
            prompt: text,
        };

        let url = format!("{}/api/embeddings", self.endpoint);
        let response = self.client.post(&url).json(&req_body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama embeddings error {}: {}", status, body));
        }

        let res: EmbeddingResponse = response.json().await?;
        Ok(res.embedding)
    }
}
