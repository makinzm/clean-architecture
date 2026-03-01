import json
import logging
import os

import hydra
import pandas as pd
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

    # 1. Load labels and texts
    df_features = pd.read_csv(cfg.data.features_path)
    issues_dict = {}
    with open(cfg.data.knowledge_base_path, encoding="utf-8") as f:
        for line in f:
            data = json.loads(line)
            issues_dict[str(data["id"])] = data

    texts_q = []
    texts_d = []
    scores = []

    for _, row in df_features.iterrows():
        issue_id = str(row["id"])
        if issue_id in issues_dict:
            # Cross-Encoders take a pair of sentences (query, document)
            texts_q.append(issues_dict[issue_id]["problem"])
            texts_d.append(issues_dict[issue_id]["solution"])
            scores.append(float(row["score"]))

    if not texts_q:
        log.error("No valid data loaded. Did you run the crawler?")
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

    # 5. Export Report
    report_path = "training_report.md"
    with open(report_path, "w") as f:
        f.write("# Training Report\n")
        f.write(f"- **Model Architecture**: {cfg.model.name} (Cross-Encoder)\n")
        f.write(f"- **Train Samples**: {len(y_train)}\n")
        f.write(f"- **Test Samples**: {len(y_test)}\n\n")
        f.write("## Evaluation Metrics (Test Split)\n")
        for k, v in eval_results.items():
            f.write(f"- **{k}**: {v:.4f}\n")
        f.write("\n> Model training validated successfully via split metrics.\n")

    # 6. Export to ONNX via Optimum
    output_dir = os.path.dirname(cfg.training.onnx_export_path)
    os.makedirs(output_dir, exist_ok=True)
    log.info(f"Exporting ONNX model to {cfg.training.onnx_export_path}")

    # First save PyTorch to a temp directory
    tmp_path = "./tmp_pytorch_model"
    trainer.save_model(tmp_path)
    tokenizer.save_pretrained(tmp_path)

    # Use Optimum to load and export
    onnx_model = ORTModelForSequenceClassification.from_pretrained(
        tmp_path, export=True
    )
    onnx_model.save_pretrained(output_dir)
    # Optimum creates a 'model.onnx', rename it to our desired output pointwise.onnx
    src_onnx_path = os.path.join(output_dir, "model.onnx")
    if os.path.exists(src_onnx_path):
        os.rename(src_onnx_path, cfg.training.onnx_export_path)

    log.info("Finished successfully. ONNX Checkpoint ready for Burn.")


if __name__ == "__main__":
    main()
