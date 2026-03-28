# GSI 型安全化・複合キー対応 計画

## 背景

現在の `raiden` / `raiden-derive` では GSI 指定が文字列ベースです。

```rust
let res = client
    .query()
    .index("testGSI")
    .key_condition(cond)
    .run()
    .await;
```

この方式には次の課題があります。

- `index("...")` が typo に弱い
- `key_condition()` に対して「その GSI のキーではない属性」も書けてしまう
- クエリ結果のデコードが常に元 struct 全体前提で、投影属性が欠ける GSI を安全に扱えない
- 複合キー GSI の制約を API と型で表現できていない

## 目標

- `#[raiden(gsi = "userIndex")]` のような宣言から、`.query().user_index()` または `.query().userIndex()` 相当の型付き API を生成する
- GSI ごとに「使えるキー属性」を型で制限する
- GSI ごとに「返る属性集合」を専用型で表現し、元 struct 全体の存在を前提にしない
- 複合キー GSI を表現できるメタデータを導入する
- 既存の `.index("...")` は移行期間中の後方互換 API として残す

## 非目標

- DynamoDB のインデックス作成そのものを `raiden` が担うこと
- 初手から全ての query/scan/filter API を GSI 専用型に全面移行すること
- 既存ユーザーのコードを一度に破壊的変更すること

## まず押さえるべき制約

複合キー GSI は通常の単一キー GSI より条件指定のルールが厳しいです。
参考: [Amazon DynamoDB で GSI の複合キーがサポートされました](https://dev.classmethod.jp/articles/amazon-dynamodb-multi-attribute-composite-keys-global-secondary-indexes/)

- GSI partition key に含めた属性はすべて `=` 条件が必要
- sort key は先頭から順にしか条件を追加できない
- 範囲条件は最後に指定した sort key にしか適用できない
- 途中の sort key を飛ばして条件を書くことはできない

この制約は、derive 時に保持する index メタデータと、生成する key condition builder の両方に反映する必要があります。

## 現状整理

### 1. index 名は文字列で注入されるだけ

- `raiden-derive/src/aws_sdk/ops/query.rs`
- `raiden-derive/src/aws_sdk/ops/scan.rs`
- `raiden-derive/src/rusoto/ops/query.rs`
- `raiden-derive/src/rusoto/ops/scan.rs`

ここでは `pub fn index(mut self, index: impl Into<String>) -> Self` を生成しているだけです。

### 2. 結果デコードは常に元 struct 全体前提

- `raiden-derive/src/aws_sdk/ops/shared.rs`
- `raiden-derive/src/rusoto/ops/shared.rs`

`expand_attr_to_item()` が元 struct の全フィールドを `FromAttribute` で復元するため、GSI が `KEYS_ONLY` や `INCLUDE` の場合は安全ではありません。

### 3. derive 属性の解析は table/key/rename まで

- `raiden-derive/src/finder/mod.rs`
- `raiden-derive/src/key/mod.rs`
- `raiden-derive/src/lib.rs`

現状は table primary key と rename 系のみを扱っており、GSI のキー構造や投影属性を保持する仕組みがありません。

## 提案方針

### 方針 A: 宣言は struct 単位、結果型は index 専用型を明示

最終形としては、GSI 定義と GSI 用投影型を分けて持つのが一番安全です。

```rust
#[derive(Raiden)]
#[raiden(table_name = "Hoge")]
#[raiden(gsi(name = "userIndex", partition_key = user_id))]
struct Hoge {
    #[raiden(partition_key)]
    id: String,
    user_id: String,
    name: String,
    age: u32,
}

#[derive(RaidenIndex)]
#[raiden(source = "Hoge", gsi = "userIndex")]
struct HogeUserIndex {
    user_id: String,
    name: String,
}
```

生成される利用イメージ:

```rust
let cond = HogeUserIndex::key_condition(HogeUserIndex::user_id()).eq("u1");
let res = client.query().user_index().key_condition(cond).run().await?;
```

利点:

- クエリ結果の型が GSI の投影と一致する
- `key_condition` の対象属性を GSI キーに限定しやすい
- 将来的に `KEYS_ONLY` / `INCLUDE` / `ALL` の扱いを整理しやすい

欠点:

- 新しい derive (`RaidenIndex`) か、それに準ずる投影型生成機構が必要
- 既存 API との差分が大きい

### 方針 B: 初期導入は元 struct 返却を残し、型安全 API を先に入れる

段階導入として、最初のリリースでは次を先行実装します。

- `#[raiden(gsi = "userIndex")]` から型付き index メソッドを生成
- GSI ごとの key condition token を別生成し、キー属性を制限
- `run()` の戻り値は従来どおり元 struct

ただしこの段階では、投影属性が不足する GSI に対しては完全な型安全性がありません。
そのため、最終的には方針 A へ進む前提にします。

## 推奨する導入順

### Phase 1: 型付き index 名 API

最低限、以下を実現します。

```rust
#[derive(Raiden)]
#[raiden(table_name = "Hoge")]
#[raiden(gsi = "userIndex")]
struct Hoge {
    #[raiden(partition_key)]
    id: String,
    user_id: String,
}

client.query().user_index()
client.scan().user_index()
```

メソッド名について:

- Rust の慣習に合わせるなら `user_index()` を正とする
- どうしても既存の命名イメージに寄せたいなら `#[allow(non_snake_case)] pub fn userIndex(...)` の別名も生成できる
- ただし長期的な保守性を考えると、標準は `snake_case` に寄せる方がよい

実装内容:

- struct attribute から GSI 名一覧を取得
- `query builder` / `scan builder` に index 固有メソッドを生成
- そのメソッド内部で `IndexName` を定数セット
- 既存の `.index("...")` は残す

このフェーズでは「文字列 typo を消す」ことに集中します。

### Phase 2: GSI キー定義の導入

`gsi = "..."` だけではキー属性が分からず、型安全な `key_condition()` を生成できません。
そのため、最終的には次のような拡張構文が必要です。

```rust
#[raiden(
    gsi(
        name = "userIndex",
        partition_key = user_id
    )
)]
```

複合キー対応版:

```rust
#[raiden(
    gsi(
        name = "userIndex",
        partition_key(user_id),
        sort_key(created_at, status)
    )
)]
```

保持したいメタデータ:

- Rust 用 index 識別子
- DynamoDB `IndexName`
- partition key の対象 field
- sort key 群
- rename / rename_all 解決後の DynamoDB 属性名

補足:

- `partition_key(user_id)` と `sort_key(created_at, status)` は複合キー GSI を見据えた表現
- 単一 sort key の場合も同じ構文で扱える

### Phase 3: GSI 専用 key condition builder

GSI ごとに以下を生成します。

- `HogeUserIndexKeyConditionToken`
- `HogeUserIndex::key_condition(...)`
- その GSI に使える属性だけを列挙した attr enum

これにより、次の誤りをコンパイル時に落とせるようにします。

- GSI に含まれない属性を `key_condition` に使う
- 複合 sort key の途中を飛ばして条件を書く
- 範囲条件を途中の sort key に書く

実現方法の候補:

- シンプル案: GSI ごとに専用 token / builder を別生成する
- 強い型案: `PartitionBound`, `Sort1Bound`, `Sort2Bound` のような stateful builder を生成する

まずは「専用 token を生成し、許可属性を絞る」実装から入るのが妥当です。

### Phase 4: GSI 専用投影型

ユーザー案の方向性を採用し、元 struct と別の index 専用型を導入します。

例:

```rust
struct Hoge {
    #[raiden(partition_key)]
    id: String,
    user_id: String,
    #[raiden(omit_gsi = "userIndex")]
    internal_field: String,
    name: String,
}

struct HogeUserIndex {
    user_id: String,
    name: String,
}
```

ただし、`omit_gsi` だけでは「どの型を返すか」が曖昧です。
安全に進めるには、次のどちらかが必要です。

- derive が `HogeUserIndex` のような projection struct を自動生成する
- ユーザーが projection struct を明示定義し、derive で関連付ける

推奨は後者です。
理由:

- 生成される public type が増えすぎない
- 命名衝突を避けやすい
- 将来的に projection ごとの trait 実装を足しやすい

`omit_gsi` は補助情報として有用ですが、単独で戻り型まで決める設計にはしない方がよいです。

### Phase 5: query/scan の戻り型を index 型へ切替

最終形では次のように分岐します。

- `client.query().run()` -> `QueryOutput<Hoge>`
- `client.query().user_index().run()` -> `QueryOutput<HogeUserIndex>`
- `client.scan().user_index().run()` -> `ScanOutput<HogeUserIndex>`

必要な改修:

- `expand_query()` / `expand_scan()` で戻り型を index ごとに切り替える
- `expand_attr_to_item()` を「対象フィールド集合を引数で受ける形」に一般化する
- projection expression / expression attribute names を index 型ベースで構築できるようにする

## `omit_gsi` の位置づけ

`omit_gsi` は次の用途なら有効です。

- projection struct 自動生成の元データ
- `ALL` ではなく `INCLUDE` / `KEYS_ONLY` 相当の投影制御
- index ごとの projection expression 組み立て

ただし、それだけでは十分ではありません。
少なくとも以下のどちらかを併用する必要があります。

- GSI ごとの専用 projection struct
- GSI ごとの `project = [field_a, field_b]` 宣言

結論として、`omit_gsi` は「補助属性」として採用し、API の中心は `gsi(...)` 定義と projection 型で組むのがよいです。

## 具体的な実装タスク

### 1. derive 属性パーサ追加

対象:

- `raiden-derive/src/finder/mod.rs`
- 新規 `raiden-derive/src/gsi/mod.rs` など

やること:

- `#[raiden(gsi = "...")]` の最小パース
- 将来の `#[raiden(gsi(...))]` も扱える AST を先に定義
- `partition_key` / `sort_key` / projection 情報を表現する内部構造体を追加

### 2. index メソッド生成

対象:

- `raiden-derive/src/aws_sdk/ops/query.rs`
- `raiden-derive/src/aws_sdk/ops/scan.rs`
- `raiden-derive/src/rusoto/ops/query.rs`
- `raiden-derive/src/rusoto/ops/scan.rs`

やること:

- `.user_index()` などのメソッドを追加生成
- 既存 `.index(String)` を非推奨候補として維持

### 3. index ごとの attr/token 生成

対象:

- `raiden-derive/src/attribute/names.rs`
- `raiden-derive/src/key_condition/*`
- `raiden-derive/src/filter_expression/*`

やること:

- GSI 専用 attr enum
- GSI 専用 key condition token / builder
- 必要なら filter 側も projection 型に追従

### 4. デコード共通化

対象:

- `raiden-derive/src/aws_sdk/ops/shared.rs`
- `raiden-derive/src/rusoto/ops/shared.rs`

やること:

- デコード対象フィールド集合を引数化
- table struct / index struct の両方で使えるようにする

### 5. 例とテストの更新

対象:

- `raiden/tests/all/query.rs`
- `raiden/examples/query_rename.rs`
- `raiden/examples/last_key.rs`
- `README.md`

やること:

- `.index("testGSI")` を型付き API の例へ差し替え
- rename 付き GSI のケースを維持
- 複合キー GSI のテストを追加
- projection 不足時の振る舞いを明文化

## 互換性ポリシー

- v1: `.index("...")` と型付き API を併存
- v2: README / examples は型付き API を優先
- v3: 破壊的変更を許容できるタイミングで `.index("...")` を非推奨化または削除

## 未解決事項

- `RaidenIndex` のような新 derive を導入するか、既存 `Raiden` のみで完結させるか
- `omit_gsi` から projection struct を自動生成するか、明示型定義を必須にするか
- 複合キー GSI の builder をどこまで state machine 化するか

## 推奨結論

実装順としては以下が最も安全です。

1. `gsi = "..."` から型付き index 名 API を生成する
2. `gsi(...)` の拡張構文でキー情報を持てるようにする
3. GSI ごとの key condition を型で制限する
4. projection struct を導入し、query/scan の戻り型を index 専用型にする
5. 複合キー制約を builder の型で表現する

特に「型付き index API」と「projection 専用型」は別フェーズに分けるべきです。
前者は移行コストが低く、後者は設計自由度が高いぶん検討項目が多いためです。

なお、index 固有メソッド名は `userIndex()` ではなく `user_index()` を第一候補とします。
要望が強ければ `userIndex()` を補助 alias として追加します。
