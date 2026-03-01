use super::entity::{Issue, RankedIssue};
use mockall::automock;

#[automock]
#[async_trait::async_trait]
pub trait EmbeddingRepository: Send + Sync {
    /// Generate embeddings for the given text (Ollama)
    async fn embed_text(&self, text: &str) -> anyhow::Result<Vec<f32>>;
}

#[automock]
#[async_trait::async_trait]
pub trait SearchRepository: Send + Sync {
    /// Retrieve the top K conceptually matching issues from the Vector DB (e.g. Qdrant)
    async fn search_issues(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> anyhow::Result<Vec<Issue>>;
    /// Upsert issues into the search index
    async fn upsert_issues(&self, issues: &[(Issue, Vec<f32>)]) -> anyhow::Result<()>;
}

#[automock]
#[async_trait::async_trait]
pub trait RankingRepository: Send + Sync {
    /// Rank the retrieved issues given the query using the ONNX model (Burn)
    async fn rank_issues(
        &self,
        query: &str,
        issues: Vec<Issue>,
    ) -> anyhow::Result<Vec<RankedIssue>>;
}

#[automock]
#[async_trait::async_trait]
pub trait LlmRepository: Send + Sync {
    /// Generate final advice based on the original query and the top-ranked issues (Ollama)
    async fn generate_advice(
        &self,
        query: &str,
        context_issues: &[RankedIssue],
    ) -> anyhow::Result<String>;
}
