use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::entity::RankedIssue;
use crate::domain::repository::LlmRepository;

pub struct OllamaClient {
    client: Client,
    endpoint: String,
    model: String,
}

impl OllamaClient {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            model,
        }
    }
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[async_trait::async_trait]
impl LlmRepository for OllamaClient {
    #[tracing::instrument(name = "OllamaLLM Generation", skip(self, context_issues))]
    async fn generate_advice(&self, query: &str, context_issues: &[RankedIssue]) -> Result<String> {
        let mut context_text = String::new();
        for (i, ri) in context_issues.iter().enumerate() {
            context_text.push_str(&format!(
                "Issue {}: \n- Problem: {}\n- Solution: {}\n\n",
                i + 1,
                ri.issue.problem,
                ri.issue.solution
            ));
        }

        let prompt = format!(
            "You are a helpful software engineering assistant.\n\
             The user has a problem: {}\n\n\
             Here are some relevant GitHub issues and their solutions that might help:\n{}\n\
             Based on the above, please provide a clear and concise solution to the user's problem.",
            query, context_text
        );

        let req_body = GenerateRequest {
            model: &self.model,
            prompt: &prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.endpoint);
        let response = self
            .client
            .post(&url)
            .json(&req_body) // Use req_body instead of payload
            .send()
            .await?;

        if !response.status().is_success() {
            tracing::warn!("Ollama returned an error status: {}", response.status());
            return Ok("LLM is unavailable or model requires pulling (e.g., `ollama pull llama3`). Please check Ollama logs.".to_string());
        }

        let ollama_res: GenerateResponse = response.json().await.unwrap_or_else(|_| GenerateResponse {
            response: "LLM model not downloaded yet. Please run: docker exec inference-gateway-ollama-1 ollama pull llama3".to_string(),
        });

        Ok(ollama_res.response)
    }
}
