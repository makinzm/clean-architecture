# Inference Gateway Implementation Plan

## Preparation
- [x] Add uv and volta to devbox
- [x] Initialize Python environment using `uv` in `apps/inference-gateway/crawler` and `apps/inference-gateway/training`
- [x] Initialize Rust project using `cargo` in `apps/inference-gateway/server`
- [x] Setup `docker-compose.yml` for Qdrant, Ollama, Prometheus, Grafana
- [x] Setup `devbox.json` environment definitions

## Step 1: Python Data Pipeline (Crawler & Training)
- [x] **Crawler**: Script to fetch closed GitHub Issues with linked PRs using GitHub API.
- [x] **Crawler**: Save output to `knowledge_base.jsonl` (RAG text) and `ranking_features.csv` (Stage 2 training data).
- [x] **Training**: Train a simple ranking model using data from `crawler`.
- [x] **Training**: Export the ranking model in ONNX format to `server/weights`.

## Step 2: Rust Domain & Usecase (SOLID Focus)
- [x] **Domain**: Define `SearchRepository` trait.
- [x] **Domain**: Define `RankingRepository` trait.
- [x] **Domain**: Define `LlmRepository` trait.
- [x] **Usecase**: Implement `RecommendUsecase` using the defined traits.
- [x] **Usecase**: Write unit tests to 100% verify `RecommendUsecase` without external connections, using `mockall`.

## Step 3: Infrastructure & Observability
- [x] **Infrastructure**: Implement `Qdrant` client for `SearchRepository` (Retrieval Stage 1).
- [x] **Infrastructure**: Implement `Burn` inference with ONNX for `RankingRepository` (Ranking Stage 2).
- [x] **Infrastructure**: Implement `Ollama` client for `LlmRepository` (Final Stage LLM).
- [x] **Infrastructure**: Implement `OpenTelemetry` tracing logic to record latency without polluting Usecase layer.

## Step 4: Web Interface & Integration
- [x] **Server**: Create API endpoints using `Axum`.
- [x] **Server**: Setup DI container and configuration in `main.rs`.
- [x] **Web**: Initialize Vite + TS in `apps/inference-gateway/web/`.
- [x] **Web**: Create simple UI and integrate with the Rust API.

## Step 5: Documentation & QA (Requested by User)
- [x] **QA**: Configure `ruff` for Python files (Crawler & Training) and format scripts.
- [x] **CI**: Add GitHub Actions workflow (`.github/workflows/ci.yml`) to test Python and Rust code.
- [x] **Docs**: Write a comprehensive `README.md` with instructions on how to run everything (Data pipelines, API, and UI).
