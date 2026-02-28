pub mod domain;
pub mod infrastructure;
pub mod interface;
pub mod usecase;

use anyhow::Result;
use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::infrastructure::{
    burn::BurnRanker, ollama::OllamaClient, qdrant::QdrantSearch, telemetry::init_tracer,
};
use crate::interface::handler::{AppState, handle_recommend};
use crate::usecase::recommend::RecommendUsecase;
// Import Qdrant properly
use qdrant_client::Qdrant; // Note: Or qdrant_client::qdrant::qdrant_client::QdrantClient if we use the gRPC client... wait let's use Qdrant builder from qdrant_client!
// Quick fallback: Qdrant::from_url("http://localhost:6334").build().unwrap() requires Qdrant builder.

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Tracing
    init_tracer()?;
    tracing::info!("Starting Inference Gateway...");

    // 2. Initialize Infrastructure
    // (Stub Qdrant init to avoid async errors; a real app parses config here)
    let qdrant_client_result = Qdrant::from_url("http://localhost:6334").build();
    let qdrant_client = match qdrant_client_result {
        Ok(client) => client,
        Err(_) => {
            // Provide a dummy panicky implementation if we just want it to compile and not run Qdrant DB yet
            panic!("Could not initiate Qdrant connection for compilation test");
        }
    };
    // Wait, earlier I wrote `client: qdrant_client::qdrant::qdrant_client::QdrantClient`. But qdrant-client 1.17 provides Qdrant wrapper under `qdrant_client::Qdrant`.
    // I need to be careful with Qdrant initialization. I'll just skip actual DB connect if testing compilation since we want to move fast.
    // Let's instantiate a `BurnRanker` and `OllamaClient`

    let search_repo = Arc::new(QdrantSearch::new(
        qdrant_client,
        "github_issues".to_string(),
    ));

    let ranking_repo = Arc::new(BurnRanker::new("../weights/pointwise.onnx".to_string()));

    let llm_repo = Arc::new(OllamaClient::new(
        "http://localhost:11434".to_string(),
        "llama3".to_string(),
    ));

    let recommend_usecase = Arc::new(RecommendUsecase::new(search_repo, ranking_repo, llm_repo));

    let app_state = Arc::new(AppState { recommend_usecase });

    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::interface::handler::handle_recommend,
        ),
        components(
            schemas(
                crate::interface::handler::RecommendRequest,
                crate::interface::handler::RankedIssueDto,
                crate::interface::handler::RecommendResponse
            )
        ),
        tags(
            (name = "recommendation", description = "Inference Gateway Recommendation API")
        )
    )]
    struct ApiDoc;

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/recommend", get(handle_recommend))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8800").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
