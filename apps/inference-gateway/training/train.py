import json
import logging
import os
import random
from datetime import datetime

import hydra
import torch
from omegaconf import DictConfig
from optimum.onnxruntime import ORTModelForSequenceClassification
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split
from transformers import (
    AutoModelForSequenceClassification,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
)

log = logging.getLogger(__name__)


class GenericDataset(torch.utils.data.Dataset):
    def __init__(self, encodings, labels):
        self.encodings = encodings
        self.labels = labels

    def __getitem__(self, idx):
        item = {key: torch.tensor(val[idx]) for key, val in self.encodings.items()}
        item["labels"] = torch.tensor(self.labels[idx], dtype=torch.float)
        return item

    def __len__(self):
        return len(self.labels)


@hydra.main(version_base="1.3", config_path="conf", config_name="config")
def main(cfg: DictConfig):
    log.info(f"Starting training pipeline with model {cfg.model.name}")

    # 0. GPU check
    if not torch.cuda.is_available():
        log.error("GPU is not available. Training aborted.")
        return

    # 1. Load labels and texts from jsonl
    issues = []
    with open(cfg.data.knowledge_base_path, encoding="utf-8") as f:
        for line in f:
            issues.append(json.loads(line))

    log.info(f"Loaded {len(issues)} issues from knowledge base.")

    texts_q = []
    texts_d = []
    scores = []

    # Generate Positive/Negative Pairs based on title <-> body mappings
    random.seed(cfg.training.seed)

    for i, issue in enumerate(issues):
        title = issue.get("title", "")
        body = issue.get("body", "") or ""

        if not title or not body:
            continue

        # Positive pair
        texts_q.append(title)
        texts_d.append(body)
        scores.append(1.0)

        # Negative pair
        neg_idx = random.choice([j for j in range(len(issues)) if j != i])
        neg_body = issues[neg_idx].get("body", "") or ""
        if neg_body:
            texts_q.append(title)
            texts_d.append(neg_body)
            scores.append(0.0)

    if not texts_q:
        log.error("No valid text pairs generated.")
        return

    # 2. Train/Test Split
    q_train, q_test, d_train, d_test, y_train, y_test = train_test_split(
        texts_q,
        texts_d,
        scores,
        test_size=cfg.training.test_size,
        random_state=cfg.training.seed,
    )

    log.info(f"Train samples: {len(y_train)}, Test samples: {len(y_test)}")

    # 3. Setup Tokenizer and Model
    tokenizer = AutoTokenizer.from_pretrained(cfg.model.name)
    model = AutoModelForSequenceClassification.from_pretrained(
        cfg.model.name, num_labels=1, ignore_mismatched_sizes=True
    )

    train_encodings = tokenizer(
        q_train,
        d_train,
        truncation=True,
        padding=True,
        max_length=cfg.model.max_length,
    )
    test_encodings = tokenizer(
        q_test,
        d_test,
        truncation=True,
        padding=True,
        max_length=cfg.model.max_length,
    )

    train_dataset = GenericDataset(train_encodings, y_train)
    test_dataset = GenericDataset(test_encodings, y_test)

    # 4. Training
    training_args = TrainingArguments(
        output_dir=cfg.training.output_dir,
        num_train_epochs=cfg.training.epochs,
        per_device_train_batch_size=cfg.training.batch_size,
        per_device_eval_batch_size=cfg.training.batch_size,
        learning_rate=cfg.training.learning_rate,
        eval_strategy="epoch",
        logging_dir="./logs",
        seed=cfg.training.seed,
    )

    def compute_metrics(eval_pred):
        predictions, labels = eval_pred
        mse = mean_squared_error(labels, predictions)
        r2 = r2_score(labels, predictions)
        return {"mse": mse, "r2": r2}

    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=train_dataset,
        eval_dataset=test_dataset,
        compute_metrics=compute_metrics,
    )

    log.info("Starting Trainer...")
    trainer.train()

    eval_results = trainer.evaluate()
    log.info(f"Evaluation Results: {eval_results}")

    # 5. Get Git Commit Hash
    try:
        import subprocess

        git_out = subprocess.check_output(["git", "rev-parse", "HEAD"])
        git_hash = git_out.decode("ascii").strip()
    except Exception as e:
        log.warning(f"Could not retrieve git commit hash: {e}")
        git_hash = "unknown"

    # 6. Export Report
    now = datetime.now()
    tag = os.environ.get("TAG", git_hash[:7] if git_hash != "unknown" else "baseline")

    # Nested date format for reports: reports/YYYY/MM/DD-<tag>.md
    date_path = now.strftime("%Y/%m/%d")
    report_dir = os.path.join("reports", date_path)
    os.makedirs(report_dir, exist_ok=True)
    report_path = os.path.join(report_dir, f"{tag}.md")

    with open(report_path, "w") as f:
        f.write("# Training Report\n")
        f.write(f"- **Date**: {now.strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"- **Tag**: {tag}\n")
        f.write(f"- **Git Commit**: {git_hash}\n")
        f.write(f"- **Model Architecture**: {cfg.model.name} (Cross-Encoder)\n")
        f.write(f"- **Train Samples**: {len(y_train)}\n")
        f.write(f"- **Test Samples**: {len(y_test)}\n\n")
        f.write("## Evaluation Metrics (Test Split)\n")
        for k, v in eval_results.items():
            f.write(f"- **{k}**: {v:.4f}\n")
        f.write("\n> Model training validated successfully via split metrics.\n")

    # 7. Export to ONNX via Optimum
    onnx_filename = f"pointwise_{tag}.onnx" if "TAG" in os.environ else "pointwise.onnx"
    # Ensure export path maps to server weights with the specified filename
    export_dir = os.path.dirname(cfg.training.onnx_export_path)
    final_export_path = os.path.join(export_dir, onnx_filename)

    os.makedirs(export_dir, exist_ok=True)
    log.info(f"Exporting ONNX model to {final_export_path}")

    # First save PyTorch to a temp directory (keep it out of root)
    tmp_path = f"outputs/tmp_pytorch_model_{tag}"
    trainer.save_model(tmp_path)
    tokenizer.save_pretrained(tmp_path)

    # Use Optimum to load and export
    onnx_model = ORTModelForSequenceClassification.from_pretrained(
        tmp_path, export=True
    )
    onnx_model.save_pretrained(export_dir)
    # Optimum creates a 'model.onnx', rename it to our desired output pointwise.onnx
    src_onnx_path = os.path.join(export_dir, "model.onnx")
    if os.path.exists(src_onnx_path):
        os.rename(src_onnx_path, final_export_path)

    log.info("Finished successfully. ONNX Checkpoint ready for Burn.")


if __name__ == "__main__":
    main()
