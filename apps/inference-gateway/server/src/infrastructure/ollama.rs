use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::entity::RankedIssue;
use crate::domain::repository::LlmRepository;

pub struct OllamaClient {
    client: Client,
    endpoint: String,
    embed_model: String,
    gen_model: String,
    gen_num_ctx: Option<u32>,
    gen_num_predict: Option<i32>,
}

impl OllamaClient {
    pub fn new(
        endpoint: String,
        embed_model: String,
        gen_model: String,
        gen_num_ctx: Option<u32>,
        gen_num_predict: Option<i32>,
    ) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            embed_model,
            gen_model,
            gen_num_ctx,
            gen_num_predict,
        }
    }
}

#[derive(Serialize)]
struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerateOptions>,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[async_trait::async_trait]
impl crate::domain::repository::EmbeddingRepository for OllamaClient {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        #[derive(Serialize)]
        struct EmbeddingRequest<'a> {
            model: &'a str,
            prompt: &'a str,
        }

        #[derive(Deserialize)]
        struct EmbeddingResponse {
            embedding: Vec<f32>,
        }

        let req_body = EmbeddingRequest {
            model: &self.embed_model,
            prompt: text,
        };

        let url = format!("{}/api/embeddings", self.endpoint);
        let response = self.client.post(&url).json(&req_body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama embeddings error {}: {}",
                status,
                body
            ));
        }

        let res: EmbeddingResponse = response.json().await?;
        Ok(res.embedding)
    }
}

#[async_trait::async_trait]
impl LlmRepository for OllamaClient {
    #[tracing::instrument(name = "OllamaLLM Generation", skip(self, context_issues))]
    async fn generate_advice(&self, query: &str, context_issues: &[RankedIssue]) -> Result<String> {
        let mut context_text = String::new();
        let gen_model = &self.gen_model;

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
            model: gen_model,
            prompt: &prompt,
            stream: false,
            options: Some(GenerateOptions {
                num_ctx: self.gen_num_ctx,
                num_predict: self.gen_num_predict,
            }),
        };

        let url = format!("{}/api/generate", self.endpoint);
        let response = self.client.post(&url).json(&req_body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::warn!("Ollama returned an error status: {}", status);
            let err_body = response.text().await.unwrap_or_default();
            tracing::error!("Ollama error body: {}", err_body);
            if status.as_u16() == 400 && err_body.contains("does not support generate") {
                return Ok(format!(
                    "Ollama model '{}' does not support generate. Set OLLAMA_GEN_MODEL to a generation-capable model (e.g., llama3:latest).",
                    gen_model
                ));
            }
            if status.as_u16() == 500 && err_body.contains("requires more system memory") {
                return Ok(format!(
                    "Ollama failed to load model '{}' due to insufficient memory. Try a smaller OLLAMA_GEN_MODEL, or lower OLLAMA_GEN_NUM_CTX (e.g., 1024), or increase Docker's available memory. Error: {}",
                    gen_model, err_body
                ));
            }
            return Ok(format!(
                "Ollama returned an error ({}). Please check if model '{}' is pulled and Ollama is healthy.",
                status, gen_model
            ));
        }

        let ollama_res: GenerateResponse =
            response.json().await.unwrap_or_else(|_| GenerateResponse {
                response: format!(
                    "Failed to parse Ollama response. Ensure model '{}' is available.",
                    gen_model
                ),
            });

        Ok(ollama_res.response)
    }
}
