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
