pub mod domain;
pub mod infrastructure;
pub mod usecase;

use anyhow::Result;
use qdrant_client::Qdrant;
use std::sync::Arc;

use crate::domain::repository::EmbeddingRepository;
use crate::infrastructure::{
    file_storage::JsonlFileRepo, git::current_commit_hash, job::FileJobRepository,
    ollama::OllamaEmbedding, qdrant::QdrantSearch,
};
use crate::usecase::ingest::IngestUsecase;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Ingestion Process...");

    // Configuration (In a real app, use env vars or a config file)
    let ollama_endpoint =
        std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_model =
        std::env::var("OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "mxbai-embed-large".to_string());
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let collection_name = "github_issues".to_string();
    let data_path = std::env::var("DATA_PATH")
        .unwrap_or_else(|_| "../crawler/data/2026-03-01-02-45/knowledge_base.jsonl".to_string());

    // 1. Initialize Infrastructure
    let file_repo = Arc::new(JsonlFileRepo);
    let embed_repo = Arc::new(OllamaEmbedding::new(ollama_endpoint, ollama_model));

    // Determine the embedding vector dimension dynamically from the selected embed model.
    let probe = embed_repo
        .embed_text("dimension probe")
        .await
        .map_err(|e| anyhow::anyhow!("Failed to probe embedding dimension: {}", e))?;
    let vector_size: u64 = probe
        .len()
        .try_into()
        .map_err(|e| anyhow::anyhow!("Embedding dimension is too large to fit u64: {}", e))?;
    if vector_size == 0 {
        return Err(anyhow::anyhow!(
            "Embedding dimension probe returned an empty vector. Check OLLAMA endpoint/model."
        ));
    }
    tracing::info!(
        vector_size,
        "Detected embedding dimension (used for Qdrant collection vectors)"
    );

    let qdrant_client = Qdrant::from_url(&qdrant_url).build()?;
    let qdrant_repo =
        Arc::new(QdrantSearch::new(qdrant_client, collection_name, vector_size).await?);
    let job_repo = Arc::new(FileJobRepository::new(std::path::PathBuf::from("status")));

    // 2. Initialize Usecase
    // Retrieve embedding model and commit hash to pass into the usecase
    let embed_model =
        std::env::var("OLLAMA_EMBED_MODEL").unwrap_or_else(|_| "unknown_model".to_string());
    let commit_hash = current_commit_hash()?;
    let usecase = IngestUsecase::new(
        file_repo,
        embed_repo,
        qdrant_repo,
        job_repo,
        embed_model,
        commit_hash,
    );

    // 3. Execute
    let job_id = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let abs_path = std::path::Path::new(&data_path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&data_path));
    tracing::info!("Starting ingestion job {} for path: {:?}", job_id, abs_path);
    usecase.execute(job_id, &data_path).await?;

    tracing::info!("Ingestion completed successfully!");
    Ok(())
}
