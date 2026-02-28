use super::entity::{Issue, RankedIssue};
use mockall::automock;

#[automock]
#[async_trait::async_trait]
pub trait SearchRepository: Send + Sync {
    /// Retrieve the top K conceptually matching issues from the Vector DB (e.g. Qdrant)
    async fn search_issues(&self, query: &str, limit: usize) -> anyhow::Result<Vec<Issue>>;
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
