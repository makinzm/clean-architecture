# Inference Gateway Tasks

## Phase 1: Planning and Research
- [x] Create this `todo.md` file.
- [x] Research SOTA ranking models (Cross-Encoder, BGE, etc.) and export-to-ONNX compatibility.
- [x] Write `implementation_plan.md` addressing all 5 user points.
- [/] Get user approval on the plan (Adding Rust ONNX inference).

## Phase 2: Crawler & Data Pipeline
- [x] Update `README.md` with explicit instructions on how to run the Crawler (e.g., GitHub Token generation).
- [ ] Refactor Python crawler to ONLY extract raw data (problems and solutions) without calculating scores.
- [ ] Save raw data to `knowledge_base.jsonl` (and potentially raw metadata).

## Phase 3: Training & Modeling (Hydra & SOTA Model)
- [x] Integrate `Hydra` for configuration management (`conf/config.yaml`).
- [ ] Create an EDA/Feature Engineering step (`eda.py` or within `train.py`) to analyze raw data and generate targets/scores.
- [x] Implement a SOTA ranking model (e.g., Cross-Encoder based on MiniLM or BGE-Reranker).
- [x] Implement Cross-Validation (K-Fold) or Train/Test Split in the training loop.
- [x] Generate training reports/metrics and save them properly to be tracked by Git.
- [x] Ensure ONNX export works correctly for the chosen architecture.

## Phase 4: Server & Infrastructure Setup
- [ ] Update `README.md` with Ollama setup instructions (`ollama pull llama3`).
- [ ] Implement Rust backend (`BurnRanker`) to actually load the ONNX model and use `tokenizers` to perform Cross-Encoder scoring.
