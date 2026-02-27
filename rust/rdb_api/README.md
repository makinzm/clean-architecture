# rdb_api — Clean Architecture REST API (Rust)

Rust + Axum + sqlx + TiDB で実装した学習用 REST API。
**TransactionManager を use_case レイヤーの trait として定義し、UseCase がトランザクションを制御する** Clean Architecture。

## Architecture

```
src/
├── domain/          # 純粋なドメインモデル (外部依存なし) + リポジトリ trait (Tx ジェネリクス)
├── use_case/        # TransactionManager trait + ユースケース実装
├── infrastructure/  # SeaORM 具体実装 (Entities, SeaOrmTransactionManager, SeaOrmUserRepository, ...)
└── presentation/    # Axum ハンドラ + OpenAPI
```

### Domain vs Entity (SeaORM) の分離について

本プロジェクトでは、クリーンアーキテクチャの原則に従い、**ドメインモデル（`src/domain/entity`）と DBモデル（`src/entity`）を明確に分離** しています。

通常、ORM（SeaORM など）を使う際、強力なマクロ（`#[derive(DeriveEntityModel)]` など）を直接ドメインモデルに付与したくなりますが、これをやってしまうと「ドメイン層が特定の DB ライブラリ（インフラ層）に強く依存（汚染）してしまう」という問題が発生します。

そのため、SeaORM 用のエンティティは `src/entity/` という独立したインフラ向けのモジュールに配置し、Repository（`src/infrastructure/repository/`）の中で、DBから取得した SeaORM モデルを純粋なドメインモデルに詰め替える（Mapping）処理を行っています。これにより、将来的に DB 操作ライブラリを変更したとしても、ドメインロジックの変更を皆無に抑えることができます。

### Transaction control flow

```
Handler → UseCase::execute()
            ├─ tx_manager.begin()
            ├─ user_repo.find_by_id(&mut tx, ...)   ┐
            ├─ order_repo.create(&mut tx, ...)       ├─ 同一 tx
            ├─ user_repo.increment_order_count(...)  ┘
            └─ tx_manager.commit(tx)
```

## Quick start

```bash
# 1. devbox shell に入る (jq は devbox 経由; docker は不要)
cd /path/to/clean-architecture   # devbox.json があるディレクトリ
devbox shell

# 2. .env 作成
cp server/.env.example server/.env

# 3. 一気通貫で動かす
cd rust/rdb_api
make all
#   → docker compose up -d
#   → TiDB が起動するまで待機 + DB 作成
#   → cargo build
#   → サーバー起動 + ヘルスチェック待機
#   → curl で 9 シナリオを順に実行
#   → サーバー停止
```

## E2E シナリオ (`make e2e`)

| # | リクエスト | 期待レスポンス |
|---|-----------|--------------|
| 1 | GET /api/v1/users | `[]` (空) |
| 2 | POST /api/v1/users (Alice) | Alice の JSON |
| 3 | POST /api/v1/users (Bob) | Bob の JSON |
| 4 | GET /api/v1/users | `[Alice, Bob]` |
| 5 | GET /api/v1/users/1 | Alice |
| 6 | POST /api/v1/orders (user_id=1, Widget×3) | Order JSON |
| 7 | GET /api/v1/users/1 | Alice (`order_count=1`) |
| 8 | POST /api/v1/orders (user_id=999) | HTTP 404 |
| 9 | POST /api/v1/users (重複 email) | HTTP 409 |

## Make targets

| コマンド | 説明 |
|---------|------|
| `make all` | up → db → e2e |
| `make up` | TiDB コンテナ起動 |
| `make db` | DB 作成 (TiDB 起動待機込み) |
| `make build` | cargo build |
| `make e2e` | サーバー起動 → curl テスト → 停止 |
| `make stop` | サーバープロセス停止 |
| `make down` | サーバー停止 + コンテナ停止 |
| `make logs` | サーバーログを tail -f |

## Endpoints

| Method | Path | 説明 |
|--------|------|------|
| GET | /api/v1/users | ユーザー一覧 |
| POST | /api/v1/users | ユーザー作成 |
| GET | /api/v1/users/{id} | ユーザー取得 |
| POST | /api/v1/orders | 注文作成 (cross-repo transaction) |
| GET | /swagger-ui | Swagger UI |
| GET | /api-docs/openapi.json | OpenAPI spec |

## Dependencies

```toml
axum = "0.8"
sea-orm = "1.1"   # MySQL / TiDB
utoipa = "5"      # OpenAPI spec generation
```

> **Note:** `utoipa-swagger-ui` は axum 0.7 にしか対応していないため、
> Swagger UI は CDN 経由で `/swagger-ui` にて提供しています。
