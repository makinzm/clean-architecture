# プロジェクト仕様書：Gaze (Go-based eBPF/Procfs Resource Observer)

## 1. プロジェクト概要

Linuxシステムの低レイヤーリソース（プロセス、CPU、メモリ、ネットワーク）を監視し、特定のドメインイベントを検知・配信するローカル監視エージェント。  
Goの並行処理（Goroutines/Channels）と eBPF（cilium/ebpf）を組み合わせ、非同期かつ低遅延なメトリクスパイプラインを構築する。

---

## 2. 何ができるようになるか

| 機能 | 詳細 |
|------|------|
| **プロセス監視** | 特定プロセス名（例: `qdrant`, `ollama`）の起動・終了をリアルタイムに検知してイベントを発行 |
| **CPU/メモリ閾値アラート** | 任意プロセスの CPU 使用率・メモリ使用量が閾値を超えた際にイベントを発行 |
| **システム全体メトリクス** | `/proc/meminfo`・`/proc/stat` からシステム全体の CPU・メモリ使用状況を定期収集 |
| **プロセス一覧のソート** | 全プロセスを CPU 使用率またはメモリ使用量でソートして取得（`top` コマンドのようなランキング表示）|
| **eBPF即時検知** | `sched_process_exec` / `sched_process_exit` カーネルフックで、プロセスの生成・終了をポーリングなしで即座に検知 |
| **リアルタイムストリーミング** | gRPC ストリーミング (`Watch`) でクライアントがメトリクスをリアルタイム購読 |
| **CLI クライアント** | `gazer-client` コマンドでターミナルからリアルタイム監視データを表示（CPU順・メモリ順で表示切り替え可能）|
| **実装の透過的切り替え** | procfs（ポーリング）と eBPF（イベント駆動）を同一インターフェースで DI により切り替え |

---

## 3. アーキテクチャ方針 (The Clean Architecture + internal/)

* **依存性の逆転 (DIP):** `usecase` は `MetricRepository` インターフェースに依存し、`procfs` や `eBPF` の具体的な実装には依存しない。
* **カプセル化:** `internal/` ディレクトリを活用し、ドメインロジックやインフラ実装がパッケージ外から不用意に参照されるのを防ぐ。
* **イベント駆動:** メトリクスの変化を `channel` を通じて `usecase` へストリーミングし、リアクティブに処理する。
* **ソート可能なメトリクスビュー:** `MetricRepository` はソート済みスナップショットを返すメソッドを提供し、`usecase` 層はソートキーを指定できる。

---

## 4. ディレクトリ構造

```text
go/observer/
├── cmd/
│   ├── gazer/              # サーバーエントリポイント。DIとシグナルハンドリング。
│   └── gazer-client/       # gRPC 購読クライアント（CLIツール）
├── internal/
│   ├── domain/             # 純粋なドメイン定義
│   │   ├── entity/         # Metric, ProcessInfo, Event (構造体)
│   │   └── repository/     # MetricRepository, EventPublisher (interface)
│   ├── usecase/            # 監視ルール ("プロセス消滅の検知", "負荷上昇の判定", "ソート")
│   ├── infrastructure/     # 具体的な実装
│   │   ├── procfs/         # /proc のパースロジック
│   │   ├── ebpf/           # cilium/ebpf を用いたカーネルイベントフック
│   │   │   └── bpf/        # BPF C プログラム (gaze.c, gaze.bpf.go)
│   │   └── collector/      # 定期実行やストリーミングの制御ロジック
│   └── presentation/       # 外部インターフェース
│       └── grpc/           # ストリーミング gRPC サーバー
├── proto/                  # gRPC 定義ファイル (.proto)
├── devbox.json             # clang, llvm, libbpf 等の開発環境定義
├── tasks/                  # 開発タスク管理
└── spec.md                 # 本ドキュメント
```

---

## 5. 各コンポーネント詳細

### A. Metric Collector (Infrastructure)

* **Procfs実装:** `/proc/[pid]/stat`・`/proc/meminfo`・`/proc/stat` を定期的にポーリングし、ドメインモデルに変換する。標準ライブラリ（`os`, `bufio`, `strconv`）のみ使用。
* **eBPF実装:** `cilium/ebpf` + BPF Cプログラム（`gaze.c`）で `sched_process_exec` / `sched_process_exit` をフック。プロセスの生成・終了をポーリングなしで即座に検知する。

### B. Observer Usecase (Logic)

* **Threshold Monitor:** 特定のメトリクスが閾値を超えた場合に `Domain Event` を発行する。
* **Process Tracker:** 特定のプロセス名（例: `qdrant`, `ollama`）を監視対象とし、そのライフサイクルを追跡する。
* **Sorted Snapshot:** 全プロセスのメトリクスを CPU 使用率またはメモリ使用量でソートして返す。`top` 的なランキングビューを提供する。

### C. Gaze Streaming (Presentation)

* **gRPC Server:** `rpc Watch(WatchRequest) returns (stream MetricEvent)` を提供。ローカルの別クライアントから、リアルタイムにシステム状態を購読可能にする。
* **Sort フィールド:** `WatchRequest` に `sort_by` フィールドを持ち、CPU・メモリ・デフォルト（PID順）を切り替えられる。

---

## 6. 実装ロードマップ

### Step 1: Foundation (Domain & Infrastructure)

> 「`internal/domain/entity` で `Metric`・`ProcessInfo`・`Event` 構造体を定義し、`internal/infrastructure/procfs` で `/proc` をパースしてそれらを取得するコードを実装する。標準ライブラリのみを使用する。」

### Step 2: Concurrency Pipeline & Sort (Usecase)

> 「取得したメトリクスを `channel` で受け取り、非同期に閾値判定を行う `usecase` を作成する。合わせて、全プロセスを CPU またはメモリ使用量でソートして返す `SortedSnapshot` usecase を実装する。`context` でゴルーチンのシャットダウンを管理する。」

### Step 3: Deep Dive (eBPF Integration)

> 「`cilium/ebpf` + `gaze.c`（BPF Cプログラム）を導入し、プロセスの開始/終了を検知する `EbpfRepository` を実装する。`procfs` 実装と同一インターフェースで DI により透過的に切り替えられるようにする。」

### Step 4: Presentation (gRPC & CLI)

> 「`presentation/grpc` でストリーミングサーバーを実装し、`WatchRequest.sort_by` でソート順を指定できるようにする。`cmd/gazer` にサーバー起動バイナリ、`cmd/gazer-client` に受信して表示する CLI クライアントを作成する。クライアントはキー操作でソート順をインタラクティブに切り替えられるようにする。」
