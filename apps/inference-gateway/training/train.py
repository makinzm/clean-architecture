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
    set_seed,
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

    # 1. Get Tag and Setup Unified Output Directory
    now = datetime.now()
    default_tag = now.strftime("%Y%m%d-%H%M")
    tag = os.environ.get("TAG", default_tag)
    # Since hydra.job.chdir is False, we use relative path from project root
    output_dir = os.path.join("outputs", tag)
    os.makedirs(output_dir, exist_ok=True)
    log.info(f"Unified Output Directory: {output_dir}")

    # Set seed for reproducibility
    set_seed(cfg.training.seed)
    log.info(f"Seed set to {cfg.training.seed}")

    # 2. Load labels and texts from jsonl
    data_path = cfg.data.knowledge_base_path
    issues = []
    with open(data_path, encoding="utf-8") as f:
        for line in f:
            issues.append(json.loads(line))

    log.info(f"Loaded {len(issues)} issues from {data_path}.")

    texts_q = []
    texts_d = []
    scores = []

    # Generate Positive/Negative Pairs
    random.seed(cfg.training.seed)

    for i, issue in enumerate(issues):
        title = issue.get("title", "")
        body = issue.get("body", "") or ""
        if not title or not body:
            continue
        texts_q.append(title)
        texts_d.append(body)
        scores.append(1.0)
        neg_idx = random.choice([j for j in range(len(issues)) if j != i])
        neg_body = issues[neg_idx].get("body", "") or ""
        if neg_body:
            texts_q.append(title)
            texts_d.append(neg_body)
            scores.append(0.0)

    if not texts_q:
        log.error("No valid text pairs generated.")
        return

    # 3. Train/Test Split
    q_train, q_test, d_train, d_test, y_train, y_test = train_test_split(
        texts_q,
        texts_d,
        scores,
        test_size=cfg.training.test_size,
        random_state=cfg.training.seed,
    )

    log.info(f"Train samples: {len(y_train)}, Test samples: {len(y_test)}")

    # 4. Setup Tokenizer and Model
    tokenizer = AutoTokenizer.from_pretrained(cfg.model.name)
    model = AutoModelForSequenceClassification.from_pretrained(
        cfg.model.name, num_labels=1, ignore_mismatched_sizes=True
    )

    train_encodings = tokenizer(q_train, d_train, truncation=True, padding=True, max_length=cfg.model.max_length)
    test_encodings = tokenizer(q_test, d_test, truncation=True, padding=True, max_length=cfg.model.max_length)

    train_dataset = GenericDataset(train_encodings, y_train)
    test_dataset = GenericDataset(test_encodings, y_test)

    # 5. Training
    training_args = TrainingArguments(
        output_dir=os.path.join(output_dir, "checkpoints"),
        num_train_epochs=cfg.training.epochs,
        per_device_train_batch_size=cfg.training.batch_size,
        per_device_eval_batch_size=cfg.training.batch_size,
        learning_rate=cfg.training.learning_rate,
        eval_strategy="epoch",
        logging_dir=os.path.join(output_dir, "logs"),
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

    # 6. Export to ONNX via Optimum
    log.info("Exporting ONNX model and assets...")
    tmp_path = os.path.join(output_dir, "pytorch_model_tmp")
    trainer.save_model(tmp_path)
    tokenizer.save_pretrained(tmp_path)
    # Also save tokenizer directly to output_dir to ensure it's there
    tokenizer.save_pretrained(output_dir)

    # Export ONNX
    onnx_model = ORTModelForSequenceClassification.from_pretrained(tmp_path, export=True)
    onnx_model.save_pretrained(output_dir)

    # Rename Optimum export
    src_onnx_path = os.path.join(output_dir, "model.onnx")
    final_onnx_path = os.path.join(output_dir, "pointwise.onnx")
    if os.path.exists(src_onnx_path):
        os.rename(src_onnx_path, final_onnx_path)

    # Clean up temp pytorch model
    import shutil
    shutil.rmtree(tmp_path)

    # 7. Get Git Commit Hash
    git_hash = "unknown"
    try:
        import subprocess
        git_hash = subprocess.check_output(["git", "rev-parse", "HEAD"]).decode("ascii").strip()
    except Exception:
        pass

    # 8. Export Report
    report_path = os.path.join(output_dir, "report.md")
    with open(report_path, "w") as f:
        f.write("# Training Report\n")
        f.write(f"- **Date**: {now.strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"- **Git Commit**: `{git_hash}`\n")
        f.write(f"- **Tag**: {tag}\n")
        f.write(f"- **Output Directory**: `{os.path.abspath(output_dir)}`\n")
        f.write(f"- **Model Architecture**: {cfg.model.name}\n")
        f.write(f"- **Seed**: {cfg.training.seed}\n")
        f.write(f"- **Train Samples**: {len(y_train)}\n")
        f.write(f"- **Test Samples**: {len(y_test)}\n\n")
        f.write("## Evaluation Metrics\n")
        for k, v in eval_results.items():
            f.write(f"- **{k}**: {v:.4f}\n")
        f.write("\n> Assets consolidated: pointwise.onnx, tokenizer.json, config.json\n")

    log.info(f"Finished successfully. All assets in {output_dir}")


if __name__ == "__main__":
    main()
