use crate::domain::entity::Issue;
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait EmbeddingRepository: Send + Sync {
    async fn embed_text(&self, text: &str) -> anyhow::Result<Vec<f32>>;
}

#[automock]
#[async_trait]
pub trait SearchRepository: Send + Sync {
    async fn upsert_issues(&self, issues: &[(Issue, Vec<f32>)]) -> anyhow::Result<()>;
}

#[automock]
pub trait FileRepository: Send + Sync {
    fn read_issues(&self, path: &str) -> anyhow::Result<Vec<Issue>>;
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct IngestionStatus {
    pub job_id: String,
    pub total_count: usize,
    pub processed_count: usize,
    pub elapsed_seconds: f64,
    pub throughput_rps: f64,
    pub percentage: f64,
    pub timestamp: String,
    pub is_completed: bool,
    pub embedding_model: String,
    pub commit_hash: String,
}

#[automock]
#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn record_status(&self, status: IngestionStatus) -> anyhow::Result<()>;
    async fn should_stop(&self, job_id: &str) -> anyhow::Result<bool>;
}
