# プロジェクト仕様書：Hybrid Two-Stage Recommendation Gateway

## 1. プロジェクト概要

GitHubのIssue（課題）とPR（解決策）を対（Pair）として収集し、検索（Stage 1）と再ランク（Stage 2）を経て、LLMが最適な解決策を提案するエンドツーエンドのシステム。

## 2. アーキテクチャ方針 (The Clean Architecture)

* **依存性の逆転 (DIP):** `server/`内のビジネスロジックは抽象（Trait）にのみ依存し、具体的なライブラリ（Burn, Qdrant, OTel）には依存しない。
* **SOLID原則の徹底:** 各コンポーネントを単一責任にし、インターフェースを分離することで、ユニットテストを容易にする。
* **モノレポ構成:** `uv` (Python) と `cargo` (Rust) を `devbox` で管理。

## 3. ディレクトリ構造

```text
apps/inference-gateway/
├── crawler/              # [Python/uv] GitHub APIからIssue/PRペアを収集
├── training/             # [Python/uv] Stage 2 (Ranking) モデルの学習とExport
├── server/               # [Rust/Axum] 推論ゲートウェイ本体（OpenAPI対応）
│   ├── src/
│   │   ├── domain/       # Entity, Repository Interface (Traits)
│   │   ├── usecase/      # Logic (Search -> Rank -> LLM), Unit Tests
│   │   ├── infrastructure/ # Implementation (Burn, Qdrant, Ollama, OTel)
│   │   ├── interface/    # Axum Handlers, DTO
│   │   └── main.rs       # DI Container / Setup
│   └── weights/          # 学習済みモデルファイル
├── web/                  # [Vite/TS] ユーザーインターフェース
├── docker-compose.yml    # Qdrant, Ollama, Prometheus, Grafana
└── devbox.json           # 開発環境一括定義

```

## 4. 各コンポーネント詳細

### A. Crawler (GitHub Knowledge Harvester)

* **対象:** `closed` かつ解決策（Linked PR）が明確なIssue。
* **出力:** - `knowledge_base.jsonl`: RAG用のテキストデータ。
* `ranking_features.csv`: Stage 2学習用の構造化データ。



### B. Training (Stage 2 Model)

* **手法:** Cross-EncoderまたはPointwise Rankingモデル。
* **出力:** ONNX形式（`.onnx`）で `server/weights/` へ書き出し（Rust側からは `burn-import` 等を利用して読み込み）。

### C. Inference Server (Rust)

* **Stage 1 (Retrieval):** `Qdrant` によるベクトル検索（上位100件）。
* **Stage 2 (Ranking):** `Burn` によるONNXモデルを用いた再ランク付け（上位3件）。
* **Final Stage (LLM):** `Ollama` による解決手順の生成。
* **Observability:** `OpenTelemetry` を用い、各ステージのレイテンシを計測。※Usecase層を汚さずInfrastructure層で計測すること。

---

## 5. AIへの実装実行指示 (Implementation Roadmap)

AIに対し、以下の順序でタスクを実行させてください。

### Step 1: Python Data Pipeline (Crawler & Training)

> 「`crawler/` で `uv` を使い、GitHub APIから『課題と解決策』のペアを抽出するスクリプトを作成してください。次に `training/` でそれらを元に単純なランキングモデルを構築し、**ONNX形式**でモデルをexportしてください。」

### Step 2: Rust Domain & Usecase (SOLID Focus)

> 「`server/src/domain/` で `SearchRepository`, `RankingRepository`, `LlmRepository` の **Trait** を定義してください。次に `server/src/usecase/` でこれらを組み合わせた `RecommendUsecase` を実装し、`mockall` を用いて、外部接続なしでロジックを100%検証する単体テストを記述してください。」

### Step 3: Infrastructure & Observability

> 「`server/src/infrastructure/` で各Traitの具体実装（Qdrant, Burn+ONNX, Ollama）を行ってください。その際、**OpenTelemetry** を用いて各ステージの処理時間を記録する処理を、ビジネスロジックから分離した形で実装してください。」

### Step 4: Web Interface & Integration

> 「AxumでAPIエンドポイントを作成し、`main.rs` で依存関係を注入してください。最後に `web/` でViteを用いたシンプルなUIを作成し、全体を疎通させてください。」
