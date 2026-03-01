# Training Update Plan

- [ ] Analyze the new data format in `apps/inference-gateway/crawler/data/2026-03-01-02-45/knowledge_base.jsonl`.
- [ ] Review current training scripts in `apps/inference-gateway/training/` to understand what needs to be changed.
- [ ] Consider and choose a SOTA model for ranking (like Cross-Encoder based on MiniLM or BGE-Reranker) as requested in `tasks/2026-03-01-inference.md`.
- [ ] Update data loading paths and parsing logic to fit the new format.
- [ ] Implement/Update model training to use the improved model.
- [ ] Ensure Cross-Validation or Train/Test Split is implemented.
- [ ] Generate and track training metrics.
- [ ] Ensure ONNX export works properly.
