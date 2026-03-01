## Inference Gateway Application

This repository contains the `apps/inference-gateway` project, a Hybrid Two-Stage Recommendation Engine built with Python and Rust.

### Setup & Running

**1. Start Dependencies (Qdrant, Ollama, Prometheus, Grafana)**
```bash
devbox run -- docker compose up -d
# Required to pull the models inside the container:
docker exec -it inference-gateway-ollama-1 ollama pull llama3:latest
docker exec -it inference-gateway-ollama-1 ollama pull phi3:latest
# Highly recommended for efficient embeddings (less memory consumption):
docker exec -it inference-gateway-ollama-1 ollama pull all-minilm
```

**2. Data Pipeline & Training (Python)**
```bash
# 1. Fetch GitHub Issues (Extract data)
devbox run -- bash -c "cd apps/inference-gateway/crawler && uv run python src/main.py"
# Outputs: knowledge_base.jsonl in a dated subdirectory of crawler/data/

# 2. Train and Export Model (Export ONNX)
# You can set TAG environment variable to version the model
devbox run -- bash -c "cd apps/inference-gateway/training && uv run python train.py"
```

**3. Data Ingestion (Rust)**
Populates the Qdrant vector database with embedded issues using Ollama.
```bash
# Optional: export OLLAMA_EMBED_MODEL=all-minilm
# Optional: export DATA_PATH=../crawler/data/YYYY-MM-DD-HH-MM/knowledge_base.jsonl
devbox run -- bash -c "cd apps/inference-gateway/ingest && cargo build --release"
# Run the compiled binary for maximum throughput:
devbox run -- bash -c "cd apps/inference-gateway/ingest && ./target/release/ingest" &
```
- **Monitoring**: Each run creates `apps/inference-gateway/ingest/status/job_{job_id}.json`. Check this file for `throughput_rps` and `processed_count`.
- **Stopping**: To stop a running job, create an empty file: `apps/inference-gateway/ingest/status/stop_{job_id}`.

**4. API Server (Rust)**
Starts the Axum inference API. The server automatically looks for its weights and tokenizer in `../training/outputs/{MODEL_TAG}/`.
```bash
export MODEL_TAG=<When Training TAG, Directory Name>
# Embedding model (must match the model used during ingestion)
export OLLAMA_EMBED_MODEL=all-minilm
# Generation model for LLM advice (must support `/api/generate`)
# NOTE: embedding-only models like `all-minilm` will fail with: "does not support generate"
export OLLAMA_GEN_MODEL=phi3:mini
# Optional: lower context length / output tokens to reduce memory usage
# export OLLAMA_GEN_NUM_CTX=1024
# export OLLAMA_GEN_NUM_PREDICT=128
devbox run -- bash -c "cd apps/inference-gateway/server && cargo run"
```

**5. Web Interface (Vite TS)**
Starts the local development frontend.
```bash
# Ensure the server is running before starting the UI
devbox run -- bash -c "cd apps/inference-gateway/web && npm run dev"
```

### OpenAPI (Swagger UI)
Interactive API documentation is hosted at:
`http://127.0.0.1:8800/swagger-ui`

### CI/QA
- **Ruff Linters**: Validated on Python code via CI.
- **Rust Tests**: 100% Mocked Usecase tier tests run via GitHub Actions `.github/workflows/ci.yml`.

---

## Qdrant (Vector DB): List collection contents & reset

This project uses `Qdrant` as its vector database.

- **REST API**: `http://localhost:6333` (use this for `curl`)
- **gRPC**: `http://localhost:6334` (used by the Rust clients in `ingest` / `server`)
- **Collection name**: `github_issues`

### List collections
```bash
curl -s http://localhost:6333/collections
```

### Collection info (config, stats)
```bash
curl -s http://localhost:6333/collections/github_issues
```

### List the actual contents (points + payload)
Qdrant stores **points** (`id`, `vector`, `payload`). To iterate over points, use `scroll`.

```bash
# Fetch up to 10 points from the beginning (payload included, vectors omitted)
curl -s \
	-H 'Content-Type: application/json' \
	http://localhost:6333/collections/github_issues/points/scroll \
	-d '{"limit": 10, "with_payload": true, "with_vector": false}'
```

If there are more points, take `next_page_offset` from the response and pass it as `offset`.

```bash
# Next page using offset (replace <OFFSET> with next_page_offset)
curl -s \
	-H 'Content-Type: application/json' \
	http://localhost:6333/collections/github_issues/points/scroll \
	-d '{"limit": 10, "offset": <OFFSET>, "with_payload": true, "with_vector": false}'
```

### Count points
```bash
curl -s \
	-H 'Content-Type: application/json' \
	http://localhost:6333/collections/github_issues/points/count \
	-d '{"exact": true}'
```

### Reset

#### 1) Delete only the collection (recommended)
Use this when you want to clear vector data and re-run ingestion.

```bash
curl -s -X DELETE http://localhost:6333/collections/github_issues
```

Re-ingest:
```bash
devbox run -- bash -c "cd apps/inference-gateway/ingest && ./target/release/ingest"
```

#### 2) Full reset by removing Docker volumes (wipes Qdrant data)
This removes the persistent volume (`qdrant_data`). Note that `docker compose down -v` also removes volumes for the other services in this compose file.

```bash
devbox run -- bash -c "cd apps/inference-gateway && docker compose down -v"
devbox run -- bash -c "cd apps/inference-gateway && docker compose up -d"
```
