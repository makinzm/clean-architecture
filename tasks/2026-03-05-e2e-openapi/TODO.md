# E2E Test & OpenAPI Integration

## 目的
1. Playwrightを導入し、クライアントからサーバーへの疎通およびチェスゲームの基本フローをE2Eで担保する。
2. HonoのAPIをOpenAPI対応（`@hono/zod-openapi`等）に移行し、クライアント側もOpenAPIから自動生成された型安全なクライアントを利用するようにリファクタリングする。

## タスクリスト

### Phase 1: Playwright Setup & RED State
- [x] `apps/e2e` パッケージ（または `apps/client` 内）に Playwright をインストール
- [x] 基本的なチェスのE2Eテスト（ゲーム作成〜初期盤面表示〜1手動かす）シナリオを作成
- [x] サーバーとクライアントを立ち上げてテストを実行する設定（`webServer`設定）を追加
- [x] **【重要】** ここで一度 `git commit` する（RED状態の保存）

### Phase 2: Playwright GREEN State
- [ ] テストを実行し、失敗内容を確認する
- [ ] サーバーやクライアントのモック/スタブ実装を本実装に少し近づけ、E2Eテストが通るように修正する
- [ ] テストが通ったことを確認（GREEN状態）
- [ ] `git commit` する

### Phase 3: Server OpenAPI Integration
- [ ] `apps/server` に `@hono/zod-openapi`, `@hono/swagger-ui`, `zod` をインストール
- [ ] 既存の Route を OpenAPI の Route 定義（`createRoute`）で書き換える
- [ ] Swagger UI エンドポイント (`/doc`, `/ui` など) を公開する
- [ ] OpenAPI Schema (`openapi.json` 等) を出力できるようにする

### Phase 4: Client OpenAPI Integration
- [ ] OpenAPIスキーマから型を生成するツール（`openapi-typescript`, `openapi-fetch`など）をクライアントに導入する
- [ ] サーバーのコードから出力した OpenAPI schema を基に TypeScript の型を生成
- [ ] クライアントの `fetch` 処理を生成した型安全なクライアントに置き換える
- [ ] E2Eテストを再度実行し、壊れていないことを確認する

### Phase 5: CI Integration
- [ ] GitHub Actions ワークフローで Playwright のテストが実行されるように追加

---


バグ修正

コマ動かせない。あと、部屋の選択もできない。
部屋の後悔形式も選べない。
白番と黒版の起動の仕方のドキュメントがない

