# Gaze Observer - Task Checklist (Completed)

## Step 1: Foundation (Domain & Infrastructure)
- [x] Go module の初期化 (`go mod init makinzm/cleanarchitecture/gaze`)
- [x] `internal/domain/entity` 定義
  - [x] `Metric` 構造体
  - [x] `ProcessInfo` 構造体
  - [x] `Event` 構造体
- [x] `internal/domain/repository` インターフェース定義
  - [x] `MetricRepository` interface (FetchAll, FetchSorted, Stream)
  - [x] `EventPublisher` interface
- [x] `internal/infrastructure/procfs` 実装
  - [x] `/proc/[pid]/stat` パーサー (comm フィールドのスペース対応)
  - [x] `/proc/meminfo` パーサー
  - [x] `ProcfsRepository` 実装 (CPU delta 計算対応)
- [x] テスト: procfs パーサーとリポジトリのユニットテスト (PASSED)

## Step 2: Concurrency Pipeline & Sort (Usecase)
- [x] `internal/infrastructure/collector/poller.go` 実装
  - [x] channel でメトリクススナップショットを送出
- [x] `internal/usecase/threshold_monitor.go` 実装
  - [x] CPU/メモリ閾値判定とイベント発行
- [x] `internal/usecase/process_tracker.go` 実装
  - [x] 特定プロセス名（qdrant, ollama等）の起動・終了検知
- [x] `internal/usecase/sorted_snapshot.go` 実装
  - [x] CPU/メモリ使用量でのソートランキング
- [x] テスト: usecase のユニットテスト & 統合テスト (PASSED)

## Step 3: Deep Dive (eBPF Integration)
- [x] `devbox.json` に clang, llvm, libbpf 等を追加
- [x] `internal/infrastructure/ebpf/bpf/gaze.c` 実装
  - [x] `sched_process_exec` / `sched_process_exit` フック
- [x] `internal/infrastructure/ebpf/repository.go` 実装
  - [x] `cilium/ebpf` + `bpf2go` 連携構造の構築
  - [x] ビルドタグ (`generatebpf`) による依存性分離
- [x] テスト: eBPF repository のインターフェース互換性テスト (PASSED)

## Step 4: Presentation (gRPC & CLI)
- [x] `proto/gaze.proto` 定義 (SortBy, TopN 対応)
- [x] protobuf / gRPC コード生成
- [x] `internal/presentation/grpc/server.go` 実装
  - [x] ストリーミング `Watch` RPC 実装
- [x] `cmd/gazer/main.go` サーバー実装
  - [x] DI とフラグベースの構成、シグナルハンドリング
- [x] `cmd/gazer-client/main.go` CLI クライアント実装
  - [x] リアルタイム表示と人間が読める単位変換
- [x] テスト: gRPC サーバーの streaming テスト (PASSED)

## Review
- [x] すべてのテスト通過確認 (`go test ./internal/...`)
- [x] ビルド確認 (`gazer`, `gazer-client`)
- [x] クリーンアーキテクチャの遵守確認
- [x] 「何ができるようになるか」の要件を満たしていることを確認

---

## Final Results
- [x] Module Path: `makinzm/cleanarchitecture/gaze`
- [x] Test Suite: unit, integration, and gRPC tests.
- [x] Features: Real-time process lifecycle events, sorting by CPU/Mem, thresholds, and gRPC streaming.
