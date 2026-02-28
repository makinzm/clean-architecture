use anyhow::Result;
use std::sync::Arc;

use crate::domain::entity::{RankedIssue, Recommendation};
use crate::domain::repository::{LlmRepository, RankingRepository, SearchRepository};

pub struct RecommendUsecase {
    search_repo: Arc<dyn SearchRepository>,
    ranking_repo: Arc<dyn RankingRepository>,
    llm_repo: Arc<dyn LlmRepository>,
}

impl RecommendUsecase {
    pub fn new(
        search_repo: Arc<dyn SearchRepository>,
        ranking_repo: Arc<dyn RankingRepository>,
        llm_repo: Arc<dyn LlmRepository>,
    ) -> Self {
        Self {
            search_repo,
            ranking_repo,
            llm_repo,
        }
    }

    pub async fn execute(&self, query: &str) -> Result<Recommendation> {
        // Stage 1: Retrieval (Fetch top 100)
        let issues = self.search_repo.search_issues(query, 100).await?;

        // Stage 2: Ranking (Rank those 100 and get top 3)
        let mut ranked = self.ranking_repo.rank_issues(query, issues).await?;
        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_3: Vec<RankedIssue> = ranked.into_iter().take(3).collect();

        // Final Stage: LLM Generation
        let advice = self.llm_repo.generate_advice(query, &top_3).await?;

        Ok(Recommendation {
            original_query: query.to_string(),
            top_issues: top_3,
            llm_advice: advice,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Issue;
    use crate::domain::repository::{
        MockLlmRepository, MockRankingRepository, MockSearchRepository,
    };

    #[tokio::test]
    async fn test_recommend_usecase() {
        let mut mock_search = MockSearchRepository::new();
        let mut mock_ranking = MockRankingRepository::new();
        let mut mock_llm = MockLlmRepository::new();

        let query = "How to handle database connections?";

        let issue1 = Issue {
            id: "1".into(),
            problem: "db".into(),
            solution: "pool".into(),
        };
        let issue2 = Issue {
            id: "2".into(),
            problem: "conn".into(),
            solution: "close".into(),
        };

        let ranked1 = RankedIssue {
            issue: issue1.clone(),
            score: 0.9,
        };
        let ranked2 = RankedIssue {
            issue: issue2.clone(),
            score: 0.5,
        };

        let retrieved_issues = vec![issue1.clone(), issue2.clone()];
        let expected_ranked = vec![ranked1.clone(), ranked2.clone()];

        // Setup expectations
        mock_search
            .expect_search_issues()
            .with(mockall::predicate::eq(query), mockall::predicate::eq(100))
            .times(1)
            .returning({
                let issues = retrieved_issues.clone();
                move |_, _| Ok(issues.clone())
            });

        mock_ranking
            .expect_rank_issues()
            .withf(move |q, issues| q == "How to handle database connections?" && issues.len() == 2)
            .times(1)
            .returning({
                let ranked = expected_ranked.clone();
                move |_, _| Ok(ranked.clone())
            });

        mock_llm
            .expect_generate_advice()
            .times(1)
            .returning(|_, _| Ok("Use a connection pool and ensure you drop the handles.".into()));

        let usecase = RecommendUsecase::new(
            Arc::new(mock_search),
            Arc::new(mock_ranking),
            Arc::new(mock_llm),
        );

        let result = usecase.execute(query).await.unwrap();

        assert_eq!(result.original_query, query);
        assert_eq!(result.top_issues.len(), 2);
        assert_eq!(
            result.llm_advice,
            "Use a connection pool and ensure you drop the handles."
        );
    }
}
