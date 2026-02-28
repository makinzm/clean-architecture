import os

import pandas as pd
import torch
import torch.nn as nn

INPUT_DIM = 128
weights_dir = "../server/weights"

class PointwiseRanker(nn.Module):
    def __init__(self, input_dim):
        super().__init__()
        # Simple 2-layer MLP
        self.fc1 = nn.Linear(input_dim, 64)
        self.relu = nn.ReLU()
        self.fc2 = nn.Linear(64, 1)

    def forward(self, x):
        x = self.fc1(x)
        x = self.relu(x)
        x = self.fc2(x)
        return x

def main():
    print("Starting training pipeline...")

    # 1. Load labels (scores)
    csv_path = "../crawler/ranking_features.csv"
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found. Run crawler first.")
        return

    df = pd.read_csv(csv_path)
    print(f"Loaded {len(df)} records from {csv_path}")

    # 2. Setup model
    model = PointwiseRanker(INPUT_DIM)
    criterion = nn.MSELoss()
    optimizer = torch.optim.Adam(model.parameters(), lr=0.01)

    # 3. Dummy training loop
    # (to ensure weights aren't just random, though it doesn't matter for architecture)
    # Generate dummy features for the dataset
    num_samples = len(df)
    features = torch.randn(num_samples, INPUT_DIM)
    targets = torch.tensor(df["score"].values, dtype=torch.float32).view(-1, 1)

    print("Training model (dummy features)...")
    for epoch in range(10):
        optimizer.zero_grad()
        outputs = model(features)
        loss = criterion(outputs, targets)
        loss.backward()
        optimizer.step()
        if (epoch+1) % 5 == 0:
            print(f"Epoch {epoch+1}/10 - Loss: {loss.item():.4f}")

    # 4. Export to ONNX
    os.makedirs(weights_dir, exist_ok=True)
    onnx_path = os.path.join(weights_dir, "pointwise.onnx")

    # Create a dummy input tensor representing a batch of size 1
    dummy_input = torch.randn(1, INPUT_DIM)

    print(f"Exporting model to {onnx_path}...")
    torch.onnx.export(
        model,
        dummy_input,
        onnx_path,
        export_params=True,
        opset_version=14,
        do_constant_folding=True,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={
            "input": {0: "batch_size"},
            "output": {0: "batch_size"}
        }
    )

    print("Success! ONNX model exported.")

if __name__ == "__main__":
    main()
