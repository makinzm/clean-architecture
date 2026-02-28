use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::entity::RankedIssue;
use crate::usecase::recommend::RecommendUsecase;

// DTOs
#[derive(Deserialize, Debug, utoipa::IntoParams, utoipa::ToSchema)]
pub struct RecommendRequest {
    pub query: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RankedIssueDto {
    pub id: String,
    pub title: String,
    pub score: f32,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RecommendResponse {
    pub query: String,
    pub llm_advice: String,
    pub related_issues: Vec<RankedIssueDto>,
}

impl From<RankedIssue> for RankedIssueDto {
    fn from(ri: RankedIssue) -> Self {
        Self {
            id: ri.issue.id,
            title: ri.issue.problem.lines().next().unwrap_or("").to_string(), // extract first line
            score: ri.score,
        }
    }
}

// App State holding the Usecase
pub struct AppState {
    pub recommend_usecase: Arc<RecommendUsecase>,
}

#[tracing::instrument(name = "HTTP handle_recommend", skip(state))]
#[utoipa::path(
    get,
    path = "/api/recommend",
    params(RecommendRequest),
    responses(
        (status = 200, description = "Recommendation from LLM based on similar issues", body = RecommendResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn handle_recommend(
    State(state): State<Arc<AppState>>,
    Query(req): Query<RecommendRequest>,
) -> impl IntoResponse {
    match state.recommend_usecase.execute(&req.query).await {
        Ok(recommendation) => {
            let res = RecommendResponse {
                query: recommendation.original_query,
                llm_advice: recommendation.llm_advice,
                related_issues: recommendation
                    .top_issues
                    .into_iter()
                    .map(|ri| RankedIssueDto {
                        id: ri.issue.id,
                        title: ri.issue.problem.lines().next().unwrap_or("").to_string(),
                        score: ri.score,
                    })
                    .collect(),
            };
            Json(res).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to generate recommendation: {:?}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            )
                .into_response()
        }
    }
}
