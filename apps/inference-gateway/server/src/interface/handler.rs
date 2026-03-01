use axum::{
    Json,
    extract::{Query, State},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{convert::Infallible, time::Duration};

use crate::domain::entity::RankedIssue;
use crate::infrastructure::ollama::OllamaClient;
use crate::usecase::recommend::RecommendUsecase;

// DTOs
#[derive(Deserialize, Debug, utoipa::IntoParams, utoipa::ToSchema)]
pub struct RecommendRequest {
    pub query: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RankedIssueDto {
    pub repo_name: String,
    pub number: i64,
    pub title: String,
    pub html_url: String,
    pub score: f32,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RecommendResponse {
    pub query: String,
    pub llm_advice: String,
    pub related_issues: Vec<RankedIssueDto>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RecommendStreamMeta {
    pub query: String,
    pub related_issues: Vec<RankedIssueDto>,
}

impl From<RankedIssue> for RankedIssueDto {
    fn from(ri: RankedIssue) -> Self {
        Self {
            repo_name: ri.issue.repo_name,
            number: ri.issue.number,
            title: ri.issue.title,
            html_url: ri.issue.html_url,
            score: ri.score,
        }
    }
}

// App State holding the Usecase
pub struct AppState {
    pub recommend_usecase: Arc<RecommendUsecase>,
    pub ollama_client: Arc<OllamaClient>,
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
                        repo_name: ri.issue.repo_name,
                        number: ri.issue.number,
                        title: ri.issue.title,
                        html_url: ri.issue.html_url,
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

#[tracing::instrument(name = "HTTP handle_recommend_stream", skip(state))]
pub async fn handle_recommend_stream(
    State(state): State<Arc<AppState>>,
    Query(req): Query<RecommendRequest>,
) -> impl IntoResponse {
    let query = req.query;

    let top_issues = match state.recommend_usecase.retrieve_and_rank(&query).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to retrieve/rank issues: {:?}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            )
                .into_response();
        }
    };

    let meta = RecommendStreamMeta {
        query: query.clone(),
        related_issues: top_issues
            .iter()
            .cloned()
            .map(RankedIssueDto::from)
            .collect(),
    };

    let meta_json = match serde_json::to_string(&meta) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to serialize meta JSON: {:?}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            )
                .into_response();
        }
    };

    let mut advice_stream = match state
        .ollama_client
        .generate_advice_stream(&query, &top_issues)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to start Ollama stream: {:?}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            )
                .into_response();
        }
    };

    let s = async_stream::stream! {
        yield Ok::<Event, Infallible>(Event::default().event("meta").data(meta_json));

        while let Some(next) = advice_stream.next().await {
            match next {
                Ok(chunk) => {
                    yield Ok(Event::default().event("delta").data(chunk));
                }
                Err(e) => {
                    yield Ok(Event::default().event("server_error").data(e.to_string()));
                    break;
                }
            }
        }

        yield Ok(Event::default().event("done").data(""));
    };

    Sse::new(s)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("ping"),
        )
        .into_response()
}
