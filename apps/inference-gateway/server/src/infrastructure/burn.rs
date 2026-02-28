use anyhow::Result;

use crate::domain::entity::{Issue, RankedIssue};
use crate::domain::repository::RankingRepository;

pub struct BurnRanker {}

impl BurnRanker {
    pub fn new(_model_path: String) -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RankingRepository for BurnRanker {
    #[tracing::instrument(name = "Burn ONNX Ranking", skip(self, issues))]
    async fn rank_issues(&self, query: &str, issues: Vec<Issue>) -> Result<Vec<RankedIssue>> {
        // In a real application, we would:
        // 1. Tokenize query + issue problem
        // 2. Pass it through the Burn ONNX model loaded from self.model_path
        // 3. Extract the score

        // For demonstration, we simply assign mock scores based on ID
        let mut ranked = issues
            .into_iter()
            .enumerate()
            .map(|(i, issue)| RankedIssue {
                issue,
                score: 1.0 / (i as f32 + 1.0), // Mock score
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(ranked)
    }
}
