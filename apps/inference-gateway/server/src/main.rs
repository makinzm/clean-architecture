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
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let ollama_endpoint =
        std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_embed_model =
        std::env::var("OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "llama3:latest".to_string());
    let ollama_gen_model =
        std::env::var("OLLAMA_GEN_MODEL").unwrap_or_else(|_| "llama3:latest".to_string());
    let ollama_gen_num_ctx: Option<u32> = std::env::var("OLLAMA_GEN_NUM_CTX")
        .ok()
        .and_then(|v| v.parse::<u32>().ok());
    let ollama_gen_num_predict: Option<i32> = std::env::var("OLLAMA_GEN_NUM_PREDICT")
        .ok()
        .and_then(|v| v.parse::<i32>().ok());
    tracing::info!("Using Ollama embed model: {}", ollama_embed_model);
    tracing::info!("Using Ollama gen model: {}", ollama_gen_model);
    if let Some(v) = ollama_gen_num_ctx {
        tracing::info!("Using Ollama gen num_ctx: {}", v);
    }
    if let Some(v) = ollama_gen_num_predict {
        tracing::info!("Using Ollama gen num_predict: {}", v);
    }

    let qdrant_client = Qdrant::from_url(&qdrant_url).build()?;

    let search_repo = Arc::new(QdrantSearch::new(
        qdrant_client,
        "github_issues".to_string(),
    ));

    let model_tag = std::env::var("MODEL_TAG").unwrap_or_default();
    let base_dir = if model_tag.is_empty() {
        "../training/outputs/latest".to_string()
    } else {
        format!("../training/outputs/{}", model_tag)
    };

    let model_path = format!("{}/pointwise.onnx", base_dir);
    let tokenizer_path = format!("{}/tokenizer.json", base_dir);

    tracing::info!("Loading ranking model from {}...", model_path);
    let ranking_repo = Arc::new(BurnRanker::new(&model_path, &tokenizer_path)?);

    let llm_client = Arc::new(OllamaClient::new(
        ollama_endpoint,
        ollama_embed_model,
        ollama_gen_model,
        ollama_gen_num_ctx,
        ollama_gen_num_predict,
    ));

    let recommend_usecase = Arc::new(RecommendUsecase::new(
        search_repo.clone(),
        ranking_repo,
        llm_client.clone(),
        llm_client,
    ));

    let app_state = Arc::new(AppState { recommend_usecase });

    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::interface::handler::handle_recommend,
            health_check,
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
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/recommend", get(handle_recommend))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8800").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy")
    )
)]
async fn health_check() -> &'static str {
    "Inference Gateway is running"
}
