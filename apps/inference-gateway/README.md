## Inference Gateway Application

This repository contains the `apps/inference-gateway` project, a Hybrid Two-Stage Recommendation Engine built with Python and Rust.

### Setup & Running

**1. Start Dependencies (Qdrant, Ollama, Prometheus, Grafana)**
```bash
devbox run -- docker compose up -d
```

**2. Data Pipeline & Training (Python)**
```bash
# 1. Fetch GitHub Issues (Extract data)
devbox run -- bash -c "cd apps/inference-gateway/crawler && uv run python crawler.py"
# Outputs: knowledge_base.jsonl, ranking_features.csv

# 2. Train and Export Model (Export ONNX)
devbox run -- bash -c "cd apps/inference-gateway/training && uv run python train.py"
# Outputs: ../server/weights/pointwise.onnx
```

**3. API Server (Rust)**
Starts the Axum inference API orchestrating retrieval, Burn ONNX ranking, and Ollama LLM interactions on `http://127.0.0.1:8800`.
```bash
devbox run -- bash -c "cd apps/inference-gateway/server && cargo run"
```

**4. Web Interface (Vite TS)**
Starts the local development frontend.
```bash
devbox run -- bash -c "cd apps/inference-gateway/web && npm run dev"
```

### OpenAPI (Swagger UI)
Interactive API documentation is hosted at:
`http://127.0.0.1:8800/swagger-ui`

### CI/QA
- **Ruff Linters**: Validated on Python code via CI.
- **Rust Tests**: 100% Mocked Usecase tier tests run via GitHub Actions `.github/workflows/ci.yml`.
