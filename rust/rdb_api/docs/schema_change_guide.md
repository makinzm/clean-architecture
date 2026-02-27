# データベーススキーマ変更ガイド (SeaORM)

このドキュメントでは、`rdb_api` プロジェクトにおいてデータベースのテーブルを追加・変更する際の手順を説明します。
本プロジェクトでは、DBのスキーマ管理とMigrationを `sea-orm-migration` を用いてRustコードで完結させています。
また、**開発環境の再現性を保つため、ツール類の実行はすべて `devbox run` を経由する** 点に注意してください。

## 1. 新しいマイグレーションファイルの作成

新しいテーブルを追加したり、既存のテーブルを変更（カラム追加など）する際は、Migrationファイルを新しく作成します。

```bash
cd rust/rdb_api
# 新しいマイグレーションファイルを生成
devbox run -- sea-orm-cli migrate generate create_user_profiles_table -d server/migration
```

上記コマンドを実行すると、`server/migration/src/mYYYYMMDD_HHMMSS_create_user_profiles_table.rs` のようなファイルが生成されます。

## 2. マイグレーションコードの記述

**【重要】マイグレーションは「追記型（インクリメンタル）」の履歴です。**
過去に作成して既に DB に適用されたマイグレーションファイル（例: `0001_create_table.rs`）は **絶対に直接書き換えないでください**。
テーブルを追加・変更したい場合は、**必ず新しいマイグレーションファイル（例: `0002_alter_table.rs`）を作成** して、そこに「どう変更するか（ALTER）」を記述します。これにより、誰の環境でも順番通りにスクリプトが実行され、同じ DB スキーマが再現されます。

生成された新しいファイルを開き、データベース定義を記述します。
SeaORMの書き方に従って、`up` (適用時) と `down` (ロールバック時) のロジックを実装します。

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 例1: テーブル作成の場合
        manager
            .create_table(
                Table::create()
                    .table(UserProfiles::Table)
                    .if_not_exists()
                    .col(pk_auto(UserProfiles::Id).integer())
                    .col(string(UserProfiles::Bio))
                    .col(integer(UserProfiles::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_profiles_user_id")
                            .from(UserProfiles::Table, UserProfiles::UserId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await

        /* 例2: 既存のテーブルを変更（カラム追加）する場合
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(integer(Users::Age).null())
                    .to_owned(),
            )
            .await
        */
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 例1: テーブル削除の場合
        manager
            .drop_table(Table::drop().table(UserProfiles::Table).to_owned())
            .await

        /* 例2: 追加したカラムを削除する場合
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Age)
                    .to_owned(),
            )
            .await
        */
    }
}

// 識別子の定義
#[derive(DeriveIden)]
enum UserProfiles {
    Table,
    Id,
    Bio,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Age, // 追加するカラムもここに定義
}
```

最後に、`server/migration/src/lib.rs` に新しいモジュールが追加されていることを確認します（自動追加されます）。

## 3. マイグレーションの適用

マイグレーションファイルが作成できたら、ローカルのデータベースに適用します。

```bash
# 事前にローカルのTiDBコンテナが起動していることを確認してください (make up)
cd rust/rdb_api
DATABASE_URL="mysql://root@127.0.0.1:4000/rdb_api" devbox run -- sea-orm-cli migrate up -d server/migration
```

※ ロールバックしたい場合は以下を実行します：
```bash
DATABASE_URL="mysql://root@127.0.0.1:4000/rdb_api" devbox run -- sea-orm-cli migrate down -d server/migration
```

## 4. Entity の自動生成・更新 (インフラ層へ配置)

クリーンアーキテクチャの原則に従い、スキーマから生成された Entity (ORM用モデル) は **必ずインフラ層 (`src/infrastructure/entity/`) に配置** します。
直接ドメイン層に置いたり、Domainモデルにマクロをつけないようにしてください。

```bash
cd rust/rdb_api
DATABASE_URL="mysql://root@127.0.0.1:4000/rdb_api" devbox run -- sea-orm-cli generate entity -o server/src/infrastructure/entity --with-serde both
```

（※注意: TiDB などで unsigned の警告等が出て自動生成に失敗する場合は、新しい Entity ファイルを `src/infrastructure/entity/` 内に手動で作成・追加してください。）

## 5. ドメインモデルの更新 (必要な場合)

Entity が変更された場合は、純粋なドメインモデルである `src/domain/entity/...` も必要に応じて更新し、
対応する Repository の `map_to_domain` 処理を変更してください。

## 6. 動作確認

ここまで完了したら、一連のE2Eテストを通じてアプリケーションが正常に動作するか確認します。
```bash
cd rust/rdb_api
make e2e
```
