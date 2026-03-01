# Inference Gateway Tasks

## Phase 3: Training & Modeling (Hydra & SOTA Model)
- [x] Integrate `Hydra` for configuration management (`conf/config.yaml`).
- [x] Create an EDA/Feature Engineering step (`eda.py` or within `train.py`) to analyze raw data and generate targets/scores.
- [x] Implement a SOTA ranking model (e.g., Cross-Encoder based on MiniLM or BGE-Reranker).
- [x] Implement Cross-Validation (K-Fold) or Train/Test Split in the training loop.
- [x] Generate training reports/metrics and save them properly to be tracked by Git.
- [x] Ensure ONNX export works correctly for the chosen architecture.
- [x] Search SOTA ranking models and choose the best one for the task like BERT, Sentence-Transformer, etc.

## Phase 4: Server & Infrastructure Setup
- [x] Update `README.md` with Ollama setup instructions (`ollama pull llama3`).
- [x] Implement Rust backend (`BurnRanker`) to actually load the ONNX model and use `tokenizers` to perform Cross-Encoder scoring.
