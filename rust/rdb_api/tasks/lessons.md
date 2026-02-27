# Lessons Learned

## [2026-02-27] TDD を守れなかった

### 何が起きたか
実装計画が与えられたとき、テストを書かずに実装を先に書いた。
ユーザーから指摘されて初めて気づいた。

### ルール
- **実装ファイルを1行も書く前に、失敗するテストを書く**
- 「計画が与えられた」はテストをスキップする理由にならない
- use_case レイヤーは外部依存がないため、常にユニットテストから始められる

---

## [2026-02-27] Makefile を「動作確認済み」と言ったが実際には試していなかった

### 何が起きたか
`make all` を実行せずに「動くはず」として commit しようとした。
ユーザーから「試してもいないのに」と指摘された。
さらに `jq` / `mysql` が devbox 経由であることも確認していなかった。
一時ファイルを `/tmp` に置いたことも指摘された (他の環境・他プロジェクトに迷惑)。

### ルール
- **「動作確認」とは実際にコマンドを実行して出力を目で確認すること**
- スクリプト/Makefile を書いたら必ず1回通しで実行してから完了と言う
- **devbox 環境では `devbox add <pkg>` でツールを追加する**
  - `devbox.json` に記録されるのでチームで共有できる
  - 例: `devbox add jq mysql` → `jq`・`mysql` コマンドが使えるようになる
  - `docker`, `kubectl` は devbox 外からでも使える (devbox add 不要)
- Makefile を書く前に `which <tool>` か `devbox.json` を確認して、
  必要なツールが入っていなければ `devbox add` してから書く
- **コマンド実行は `devbox run -- <cmd>` で行う**
- **`/tmp` など他プロジェクトと共有される場所に一時ファイルを置かない**
  - ログ・PID ファイルはプロジェクト内 (例: `.e2e/`) に置く
  - `.gitignore` に追加してコミットしない

---

## [2026-02-27] devbox の `mysql@latest` が MariaDB サーバーをインストールする

### 何が起きたか
`devbox add mysql` すると `mysql@latest` → MariaDB フルパッケージがインストールされ、
`devbox shell` 入室時に MariaDB のデータディレクトリを初期化する処理が走った。
ユーザーが `make all` を実行しても TiDB ヘルスチェックが通らず "retrying in 2s..." で止まった。
(mysql クライアントが PATH にない状態でヘルスチェックが常に失敗していた)

### ルール
- **devbox の `mysql@latest` は MariaDB サーバーごとインストールする — 使わない**
- ホスト側に MySQL クライアントが必要な場合は、依存を排除する設計を検討する:
  - TCP ポート確認: `(echo > /dev/tcp/HOST/PORT) 2>/dev/null`
  - DB 作成/リセット: `sqlx::migrate::MigrateDatabase` を使ってサーバーバイナリ自身に `--reset` モードを持たせる
  - TiDB の HTTP ステータス: `curl http://localhost:10080/status`（ただし docker-compose でポート公開が必要）
- `kill` でプロセスを停止した直後はポートがまだ解放されていない場合がある
  → `kill` 後に `lsof -ti:PORT` が空になるまでループで待つ

---

## [2026-02-27] ORMとドメインモデルの分離徹底と devbox の利用

### 何が起きたか
SeaORM移行時、`src/entity` フォルダを `src/` 配下（ドメインに並ぶ位置）に作成してしまった。
ユーザーから「インフラの変更話なのにドメインやエンティティをいじっているのはおかしい」と当然の指摘を受けた。また、`devbox` 環境を活かして `devbox run` を使う配慮も足りなかった。

### ルール
- **クリーンアーキテクチャでは、ORM仕様の DB モデル（Entity）はインフラ層 (`src/infrastructure/entity`) に配置する**
  - ORM固有のマクロ（`#[derive(DeriveEntityModel)]` など）がついた構造体はインフラ固有のものであるため、ドメイン層と同格の位置（`src/entity`）に置いてはいけない。
  - `src/domain/entity` は外部DBライブラリに一切依存しない純粋な構造体として保ち、リポジトリ層実装でモデルの詰め替え（Mapping）を行うこと。
- **コマンドラインツール（`sea-orm-cli`など）は常に `devbox run` を経由して実行する**
  - グローバルにコマンドをインストールする（`cargo install`など）行為は禁止。
  - プロジェクトに閉じた再現性を担保するため、かならず `devbox.json` で管理し `devbox run -- <cmd>` の形式で呼び出すこと。
