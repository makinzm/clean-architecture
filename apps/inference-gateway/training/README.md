# Model Training (Inference Gateway)

This directory handles the feature engineering, fine-tuning, evaluation, and ONNX conversion pipelines for the Machine Learning semantic ranking models.

## Pre-requisites
- Ensure you have run the Crawler to generate a new `knowledge_base.jsonl`.
- Verify your environment variables and paths in `conf/config.yaml`.
- (Optional but highly recommended) Run an Exploratory Data Analysis (EDA) script from `eda/` to understand the data drift and feature targets before kicking off full-scale training.

## Training Workflow & Versioning

To ensure full traceability of the code artifacts against the generated weights, **all training reports are locked to the current Git Commit hash**.

Therefore, the proper workflow is:

1. **Modify & Test Structure**: Make all configuration or architectural changes (`train.py`, `config.yaml`, models, max_length thresholds). Test them superficially if needed.
2. **Commit**: Before running the definitive job, commit your code to Git:
   ```bash
   git add .
   git commit -m "chore(ml): configuring BGE reranker for release 1.3.0"
   ```
3. **Train**: Kick off the training job:
   ```bash
   uv run train.py
   ```

### Output Generation
The training script utilizes the state of the Git repository to formulate its outputs:
1. **Model Weights**: A temporary PyTorch checkpoint is saved out of sight to `outputs/tmp_pytorch_model_<hash>`.
2. **ONNX Export**: The PyTorch model is converted via Optimum and securely exported to `apps/inference-gateway/server/weights/pointwise_<hash>.onnx` (or `pointwise.onnx` if no tag override).
3. **Report Matrix**: The test split evaluations (MSE, R2) alongside the Git Commit Hash are serialized dynamically into `reports/YYYY/MM/DD-<hash>.md`.
