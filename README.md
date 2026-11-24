<p align="center"><img src ="https://github.com/bokuweb/raiden/blob/master/assets/logo.png?raw=true" /></p>

<p align="center">
    DynamoDB library for Rust.
</p>

---

![Continuous Integration](https://github.com/bokuweb/raiden/workflows/Continuous%20Integration/badge.svg)

## Examples

You can see more examples [here](https://github.com/raiden-rs/raiden-dynamo/tree/master/raiden/examples)

### Generating client

`raiden` uses `aws-sdk-dynamodb` or `rusoto_dynamodb` as internal client.

#### With aws-sdk-dynamodb (`aws-sdk` is enabled)

Rust 1.91.1 or newer is required. The AWS SDK minimum versions include the
`aws-smithy-json` recursion-limit fix (GHSA-8ffr-xgwf-xj56).

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    // Simply, specify the region.
    let client = User::client(config::Region::from_static("us-east-1"));

    // You can also specify aws-sdk-dynamodb client.
    let client = {
        let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(raiden::config::Region::from_static("us-east-1"))
            .load()
            .await;
        let sdk_client = raiden::Client::new(&sdk_config);

        User::client_with(sdk_client)
    };

    // Run operations...
}
```

#### With rusoto_dynamodb (`rusoto` or `rusoto_native_tls` is enabled)

The default `rusoto` backend uses `native-tls` (OpenSSL on Linux).
`rusoto_native_tls` is an explicit alias. For a maintained rustls stack, use
`default-features = false, features = ["aws-sdk"]` instead.

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    // Simply, specify the region.
    let client = User::client(Region::UsEast1);

    // You can also specify rusoto_core client.
    let client = User::client_with(Client::shared(), Region::UsEast1);

    // Run operations...
}
```

#### Empty values and custom attribute omission

Both backends preserve explicit DynamoDB values returned by custom
`IntoAttribute` implementations. An empty list (`L: []`, including
`Document<Vec<T>>`) is stored, and an empty map is stored. Explicit empty
`SS`, `NS`, and `BS` values are sent to DynamoDB and rejected as invalid.
Built-in empty `HashSet` and `BTreeSet` fields are still omitted (or removed
by a SET update); an empty `Vec` still writes `NULL`.

For a custom type that deliberately omits a field, override
`IntoAttribute::into_attr_option` to return `None`. This also applies when
wrapped in `Some(...)`. Wrappers that delegate conversion to a built-in set
should also delegate `into_attr_option` to retain empty-set omission.
`Option::None` itself continues to write `NULL`.
The override is used by Put, BatchPut, transactional Put, and update
expressions; nested document values and condition values use `into_attr`.

When migrating custom Rusoto converters that return
`AttributeValue::default()`, use this explicit omission method on the SDK
backend. The previous SDK workaround of returning `L: []` now stores an
empty list. This change does not recover attributes already omitted from
stored items: keep `#[raiden(use_default)]` where reading older missing
attributes requires a default.

#### Backend compatibility and migration differences

The backends share the tested attribute conversion contract, but migrating
backends does not guarantee identical behavior for every input or operation.

| Area | Behavior and migration action |
| --- | --- |
| Explicit empty lists/maps | Both backends preserve them. Previous SDK versions silently omitted some empty values; this release stores them. Existing omitted data is not repaired; retain `#[raiden(use_default)]` where needed. |
| Deliberate omission | Rusoto's native `AttributeValue::default()` has no SDK enum equivalent. On the SDK backend, implement `into_attr_option()` returning `None`; wrappers must delegate this method too. Do not substitute an empty list for omission. |
| Nested empty built-in sets | Omission applies to supported field/update contexts, not recursively. Nested empty sets and direct key/condition/filter/arithmetic operands can still encode as `{}` with Rusoto versus `SS: []` / `NS: []` with SDK. Both payloads are invalid, but request bodies and error messages differ. Transactional `.delete(attribute, value)` also uses this direct conversion. |
| Persisted output JSON | SDK nested request/capacity/metrics metadata now uses Rusoto's PascalCase keys and omits unset fields. Older SDK snake_case metadata keys remain accepted as input; new output uses the canonical keys. Outer Raiden output field names stay snake_case. |
| Malformed attribute JSON | Raiden's JSON entry points now reject missing/multiple/unknown type tags and null or wrongly typed payloads on **both** backends. In particular, `{"NULL":null}` is rejected instead of becoming SDK `NULL:true`. This intentionally tightens the previous Rusoto decoder too; see the examples and affected APIs below. |
| Numeric reads | Both backends can still panic when an integer scalar is read from a fractional, out-of-range, or malformed number, due to the existing `parse().unwrap()` conversion. Matching behavior does not mean these inputs are handled safely. |
| Pagination | Query/Scan now stop on either an absent or empty `LastEvaluatedKey`; previously both backends could request another page or expose an empty continuation token. Shared scripted tests cover page/limit boundaries. |
| HTTP error compatibility | SDK errors now use Rusoto's per-operation classification, message text and strict `__type`/`message` decoding, including malformed error details. Exploration compares retry eligibility as well as variants. Raw transport/error payload types remain backend-specific. |
| Optional page counters | SDK Query/Scan preserve absent or `null` Count/ScannedCount as `None`, instead of synthesizing `Some(0)`. Explicit zero stays `Some(0)`; counters still describe the final page. |
| Transaction request tokens | WriteTx omits ClientRequestToken on both backends, including retries. SDK previously generated a token implicitly; this compatibility fix removes that implicit AWS idempotency protection. A lost response followed by a retry can execute the transaction again, as with Rusoto. |
| Runtime behavior | Explicit retry policies and canonical transaction cancellation reasons have shared HTTP regression tests; see the [operational compatibility audit](docs/sdk-rusoto-operational-compatibility.md) for coverage and remaining risks. Retry defaults differ (see below). Credentials, region resolution, timeouts, HTTP/TLS, throttling, IAM, and real AWS failure handling have not been proven equivalent by the attribute tests. |

**Raw JSON validation is a compatibility change for both backends.** Each raw
attribute must be an object with exactly one recognized DynamoDB type field.
For example, all of the following now fail decoding on both backends, including
when nested inside `L` or `M`:

```json
{}
{"UNKNOWN":"value"}
{"S":"a","N":"1"}
{"NULL":null}
```

These are four independent examples. Rusoto previously accepted all four;
the SDK previously accepted the last as a true NULL. They are now rejected,
not coerced to NULL, discarded, or resolved by choosing one of multiple tags.
The rule applies when deserializing `BatchPutOutput.unprocessed_items`,
`BatchDeleteOutput.unprocessed_items`, `BatchGetOutput.unprocessed_keys`,
`UpdateOutput.item_collection_metrics`, `ScanOutput.last_evaluated_key`, and
when decoding attributes with `NextToken::into_attr_values()`.
Malformed tokens return `RaidenError::NextTokenDecodeError`; malformed output
JSON returns a serde decode error. Regenerate malformed pagination tokens and
repair persisted malformed output from its original source before replaying it;
there is no automatic migration that can infer the intended attribute type.

Explicit `L: []`, `M: {}`, empty set payloads, and both boolean `NULL` flags are
preserved at this JSON boundary. This validates representation, not every
DynamoDB service constraint: empty sets are still invalid writes, and
`NULL:false` still fails nullable Rust reads. Ordinary user model JSON is not
interpreted as raw DynamoDB JSON by these checks.

Native backend types remain exposed. Directly calling serde on Rusoto's own
`AttributeValue`/request types, or constructing SDK requests outside Raiden,
bypasses these Raiden boundary checks and retains backend-specific behavior.

Regression tests cover these six entry points with root and nested malformed
attributes, as well as accepted empty values. CI also compares seeded random
corpora both with and without malformed attributes; it fails on differences
rather than excluding known malformed cases. This is finite differential
exploration, not proof of equivalence or coverage-guided fuzzing.
See the [attribute compatibility audit](docs/sdk-rusoto-attribute-compatibility.md)
for the complete tested matrix, reproduction commands, and limitations.

#### Set prefix/suffix to the table name

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    let client = User::client(config::Region::from_static("us-east-1"))
        .table_prefix("prefix-")
        .table_suffix("-suffix");

    // Print `prefix-user-suffix`
    println!("{}", client.table_name());
}
```

#### Configure retry strategy

NOTE: Default retry strategy differs between `aws-sdk` and `rusoto`

- `aws-sdk` ... Not retry in raiden by default. Because you can configure retry strategy using `aws_config`. Or you can configure your own strategy like next example.
- `rusoto` ... Enabled retrying in raiden by default. See detail [here](https://github.com/mythrnr/raiden-dynamo/blob/master/raiden/src/retry/mod.rs).

The `again` based error retry policies use fixed or exponential delays without
jitter. Batch write retries still apply jitter to unprocessed items.

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

// Force retry 3 times.
struct MyRetryStrategy;

impl RetryStrategy for MyRetryStrategy {
    fn should_retry(&self, _error: &RaidenError) -> bool {
        true
    }

    fn policy(&self) -> Policy {
        Policy::Limit(3)
    }
}

#[tokio::main]
async fn main() {
    let client = User::client(config::Region::from_static("us-east-1"))
        .with_retries(Box::new(MyRetryStrategy));

    // Run operations...
}
```

### Running operations

#### get_item

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;
    let _res = client.get("user_primary_key").run().await;
}
```

#### put_item

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;
    let input = User::put_item_builder()
        .id("foo".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let _res = client.put(&input).run().await;
}
```

`put_item_builder()` is generated with `raiden::Builder` (safe-builder). Setters
for required fields can be called in any order, and `build()` is available once
all of them are set. Setters for `Option` fields (`nickname(T)` and
`nickname_with_option(Option<T>)`) keep the builder type, so they can be called
conditionally. `Default<Struct>PutItemInputBuilder` names the type returned by
`put_item_builder()`.

```rust
fn start() -> DefaultUserPutItemInputBuilder {
    User::put_item_builder()
}

let mut builder = start().id("foo".to_owned()).name("bokuweb".to_owned());
if let Some(nickname) = nickname {
    builder = builder.nickname(nickname);
}
let input = builder.build();
```

#### Migrating from the removed Rusoto rustls features

**Breaking change:** `rustls` and `rusoto_rustls` have been removed.
Update existing lockfiles with `cargo update` after changing dependencies. Rusoto 0.48
pins rustls 0.20 / ring 0.16, and their security fixes cannot be selected by a
compatible dependency update. Both supported backends now avoid those versions.

For rustls, switch to the AWS SDK backend and remove direct `rusoto_*`
dependencies from the migrated code:

```toml
[dependencies]
raiden = { version = "*", default-features = false, features = ["aws-sdk"] }
aws-config = "1.8.18"
aws-sdk-dynamodb = { version = "1.114.0", default-features = false, features = ["default-https-client", "rt-tokio"] }
```

Replace Rusoto's `Region`, `Client`, and `HttpClient` construction with AWS SDK
configuration. Load credentials and region through `aws_config`; pass the SDK
client as the single argument to `User::client_with`:

```rust,ignore
let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
    .region(aws_sdk_dynamodb::config::Region::new("ap-northeast-1"))
    // For local DynamoDB, also set .endpoint_url("http://localhost:8000").
    .load()
    .await;
let client = User::client_with(aws_sdk_dynamodb::Client::new(&config));
```

The SDK's default HTTPS client uses modern rustls. For an explicit HTTP client,
add `aws-smithy-http-client = { version = "1.1.13", features = ["rustls-aws-lc"] }`
and set the HTTP client before `.load()`:

```rust,ignore
.http_client(
    aws_smithy_http_client::Builder::new()
        .tls_provider(aws_smithy_http_client::tls::Provider::Rustls(
            aws_smithy_http_client::tls::rustls_provider::CryptoMode::AwsLc,
        ))
        .build_https(),
)
```

Update backend-specific types and operations using the AWS SDK examples and the
backend differences documented above. The downstream fixture
in `raiden/tests/downstream-backends` compiles this client construction and the
public put builders together.

To retain the Rusoto API, switch Raiden to `features = ["rusoto_native_tls"]`
(with default features disabled), and change every direct `rusoto_core` /
`rusoto_dynamodb` dependency to `default-features = false, features = ["native-tls"]`.
`rusoto_core::HttpClient::new()` then uses native TLS. This option requires
OpenSSL on Linux; use the AWS SDK option above for rustls or musl builds that
must avoid OpenSSL.

#### Migrating from 0.0.95 - 0.0.97

0.0.95 to 0.0.97 generated put item builders with [bon](https://bon-rs.com) and
re-exported bon's derive as `raiden::Builder`. 0.0.98 goes back to the
safe-builder API of 0.0.94 and earlier, so code written for 0.0.94 compiles
unchanged. Code written for 0.0.95 - 0.0.97 may need these changes:

- `#[derive(raiden::Builder)]` meant bon's derive. Use `raiden::BonBuilder` with
  `#[builder(crate = ::raiden::bon)]`, or depend on `bon` directly.
  `raiden::Builder` is safe-builder's derive again and ignores bon attributes.
- bon specific methods on generated put builders (for example `maybe_nickname`)
  are gone. Use `nickname_with_option(opt)`.
- Types written as `UserPutItemInputBuilder<S>` with a bon state parameter
  become `DefaultUserPutItemInputBuilder` for the initial builder.

#### store maps and nested documents

```rust
use std::collections::{BTreeMap, HashMap};

use raiden::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, RaidenDocument)]
struct Profile {
    display_name: String,
    level: usize,
}

#[derive(Raiden)]
#[raiden(table_name = "user")]
struct User {
    #[raiden(partition_key)]
    id: String,
    metadata: HashMap<String, usize>,
    flags: BTreeMap<String, bool>,
    profile: Profile,
    profiles: HashMap<String, Profile>,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;

    let mut metadata = HashMap::new();
    metadata.insert("score".to_owned(), 42);

    let mut flags = BTreeMap::new();
    flags.insert("active".to_owned(), true);

    let profile = Profile {
        display_name: "bokuweb".to_owned(),
        level: 3,
    };

    let mut profiles = HashMap::new();
    profiles.insert("primary".to_owned(), profile.clone());

    let input = User::put_item_builder()
        .id("user#1".to_owned())
        .metadata(metadata)
        .flags(flags)
        .profile(profile)
        .profiles(profiles)
        .build();

    let _res = client.put(input).run().await;
}
```

Notes:

- map key is currently limited to `String`
- use `#[derive(RaidenDocument)]` when you want to store a nested type directly as a field
- enums can also derive `RaidenDocument`; serde enum tagging such as `#[serde(tag = "type")]` is preserved when values are encoded into DynamoDB maps and decoded back
- `Document<T>` remains available as an explicit wrapper when you prefer opt-in at the field type level
- empty maps are preserved as empty DynamoDB `M` values rather than being dropped

Tagged enums can be stored directly as nested document fields:

```rust
use std::collections::HashMap;

use raiden::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, RaidenDocument)]
#[serde(tag = "type")]
enum Message {
    Request {
        id: String,
        method: String,
        params: HashMap<String, usize>,
    },
    Response {
        id: String,
        result: String,
    },
}

#[derive(Raiden)]
#[raiden(table_name = "event")]
struct Event {
    #[raiden(partition_key)]
    id: String,
    message: Message,
}

fn input() -> EventPutItemInput {
    let mut params = HashMap::new();
    params.insert("attempt".to_owned(), 1);

    Event::put_item_builder()
        .id("event#1".to_owned())
        .message(Message::Request {
            id: "msg#1".to_owned(),
            method: "send".to_owned(),
            params,
        })
        .build()
}
```

When a tagged enum is used as the item type itself, the DynamoDB item map is decoded through serde as a whole document. This allows projection reads such as `query().project::<Message>()` when the item contains the tag and variant fields at the top level.

When decoding an item fails, `RaidenError::AttributeConvertError` includes the attribute name and the original `ConversionError` in its `source` field. This also preserves custom messages returned by `FromAttribute` implementations:

```rust
if let Err(RaidenError::AttributeConvertError { attr_name, source }) = result {
    eprintln!("could not decode {attr_name}: {source}");
}
```

#### query nested map and document values

```rust
use std::collections::HashMap;

use raiden::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, RaidenDocument)]
struct Profile {
    display_name: String,
    level: usize,
}

#[derive(Raiden)]
#[raiden(table_name = "user")]
struct User {
    #[raiden(partition_key)]
    id: String,
    admin_ids: Vec<String>,
    metadata: HashMap<String, usize>,
    profile: Profile,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;

    let key = User::partition_key_condition().eq("user#1");

    let filter = User::filter_expression(User::metadata().key("score"))
        .ge(40)
        .and(User::filter_expression(User::profile().field(Profile::level())).eq(3));

    let _res = client
        .query()
        .key_condition(key)
        .filter(filter)
        .run()
        .await;

    let condition = User::condition()
        .attr_exists(User::metadata().key("score"))
        .and(User::condition().attr(User::profile().field(Profile::level())).eq_value(3));

    // Condition expressions also support typed function operands and logical
    // composition. Values and names are bound through collision-free placeholders.
    let preserve_last_admin = User::condition()
        .not()
        .contains(User::admin_ids(), "member#1")
        .or(User::condition().size(User::admin_ids()).ge(2_usize));

    let not_in_range = User::condition()
        .between(User::metadata().key("score"), 10, 20)
        .or(User::condition().in_values(User::metadata().key("score"), [30, 40]))
        .not(); // NOT (the entire BETWEEN ... OR ... group)

    // `condition` can be passed to `put`, `update`, `delete`,
    // transaction writes, and other conditional operations.
}
```

Notes:

- use `.key("...")` for dynamic map keys such as `metadata.score`
- use `partition_key_condition().eq(...)` for a table query; when the model has a sort key, chain `.and(Model::sort_key_condition().begins_with(...))` (or another supported sort-key comparison)
- the existing `key_condition(Model::attribute())` remains available for compatibility, but it does not verify that the attribute is a table key
- use `.field(...)` with `#[derive(RaidenDocument)]` accessors for nested document fields such as `profile.level`
- document paths are supported in `filter_expression` and `condition`
- condition expressions support `contains`, attribute comparisons (`eq_attr` / `eq_value`, `ne_attr` / `ne_value`, `lt_attr` / `lt_value`, `le_attr` / `le_value`, `gt_attr` / `gt_value`, `ge_attr` / `ge_value`), `between`, `in_values` (1–100 values), numeric `size` comparisons (`eq`, `ne`, `lt`, `le`, `gt`, and `ge`), and `not` / `and` / `or` composition
- call `.not()` on a completed condition to negate the whole group
- `size` comparisons only accept values implementing `IntoNumberAttribute`; raw condition strings are not needed
- `key_condition` still follows DynamoDB key rules, so nested map/document values are not valid partition or sort keys unless you project them to top-level attributes or an index
- `.index(usize)` is also available when you need to address list elements in a document path
- path segments are emitted through expression attribute names, so reserved words remain escaped correctly

#### batch_get_item

```rust
use raiden::*;

#[derive(Raiden, Debug, PartialEq)]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    year: usize,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;
    let keys: Vec<(&str, usize)> = vec![("Alice", 1992), ("Bob", 1976), ("Charlie", 2002)];
    let res = client.batch_get(keys).run().await;
}
```

`batch_get` splits the keys into requests of at most 100 keys and retries `UnprocessedKeys`.
Duplicate keys are removed before sending, so each existing item is returned once. The order of the returned items is not specified.

Use `client.batch_get(keys).consistent().run().await` for strongly consistent reads.
The default is eventually consistent reads. The setting applies to all batches and retries.

#### batch_put

```rust
use raiden::*;

#[derive(Raiden, Debug, PartialEq)]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    year: usize,
    name: String,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;
    let items = vec![
        User::put_item_builder()
            .id("Alice".to_owned())
            .year(1992)
            .name("Alice".to_owned())
            .build(),
        User::put_item_builder()
            .id("Bob".to_owned())
            .year(1976)
            .name("Bob".to_owned())
            .build(),
    ];

    let res = client.batch_put(items).run().await;
}
```

#### transact_get_items

```rust
use raiden::*;

#[derive(Raiden, Debug, PartialEq)]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    year: usize,
    name: String,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;
    let keys: Vec<(&str, usize)> = vec![("Alice", 1992), ("Bob", 1976)];
    let res = client.transact_get(keys).run().await;

    // Responses preserve the request order.
    // Missing items are returned as `None`.
}
```

#### query with typed GSI

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
#[raiden(
    gsi(
        name = "userIndex",
        partition_key = "org_id",
        sort_key = "created_at",
        sort_key = "status"
    )
)]
struct User {
    #[raiden(partition_key)]
    id: String,
    org_id: String,
    created_at: String,
    status: String,
    #[raiden(omit_gsi = "userIndex")]
    internal_note: String,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;

    let cond = UserIndexItem::user_index_key_condition()
        .eq("org_1")
        .and(UserIndexItem::user_index_sort_key_condition_1().eq("2026-03-28T00:00:00Z"))
        .and(UserIndexItem::user_index_sort_key_condition_2().begins_with("active"));

    let _res = client
        .query()
        .user_index()
        .project::<UserIndexItem>()
        .key_condition(cond)
        .run()
        .await;

    let _res = UserIndexItem::query(&client)
        .key_condition(cond)
        .run()
        .await;
}
```

If you want to override the generated projection shape or type name, declare a
`RaidenIndex` explicitly:

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
#[raiden(gsi(name = "userIndex", partition_key = "org_id"))]
struct User {
    #[raiden(partition_key)]
    id: String,
    org_id: String,
    display_name: String,
    avatar_url: String,
    #[raiden(omit_gsi = "userIndex")]
    internal_note: String,
}

#[derive(RaidenIndex, Debug, PartialEq)]
#[raiden(source = "User", gsi = "userIndex")]
#[raiden(gsi(name = "userIndex", partition_key = "org_id"))]
struct PublicUserIndexItem {
    org_id: String,
    display_name: String,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;
    let cond = PublicUserIndexItem::user_index_key_condition().eq("org_1");

    let _res = PublicUserIndexItem::query(&client)
        .key_condition(cond)
        .run()
        .await;
}
```

Composite GSIs with multiple sort-key segments are also supported:

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
#[raiden(
    gsi(
        name = "activityIndex",
        partition_key = "org_id",
        sort_key = "created_at",
        sort_key = "status"
    )
)]
struct User {
    #[raiden(partition_key)]
    id: String,
    org_id: String,
    created_at: String,
    status: String,
    #[raiden(omit_gsi = "activityIndex")]
    internal_note: String,
}

#[tokio::main]
async fn main() {
    let client = /* generate client */;

    let cond = UserActivityIndexItem::activity_index_key_condition()
        .eq("org_1")
        .and(UserActivityIndexItem::activity_index_sort_key_condition_1().eq("2026-03-28T00:00:00Z"))
        .and(UserActivityIndexItem::activity_index_sort_key_condition_2().begins_with("active"));

    let _res = UserActivityIndexItem::query(&client)
        .key_condition(cond)
        .run()
        .await;
}
```

The composite helper methods enforce DynamoDB's ordering rules:

- start with the partition key
- then chain sort key segment 1, sort key segment 2, and so on
- use range operators such as `gt`, `between`, and `begins_with` only on the last sort key segment

Notes:

- typed GSI methods such as `user_index()` are generated from `#[raiden(gsi = "...")]` or `#[raiden(gsi(...))]`
- `#[raiden(omit_gsi = "userIndex")]` also generates a default projection type such as `UserIndexItem`, so the common case does not require writing `#[derive(RaidenIndex)]` manually
- `#[derive(RaidenIndex)]` remains available when you want to override the generated projection shape or name, or when you prefer to declare the projection item explicitly
- `#[derive(RaidenIndex)]` also generates `YourIndexType::query(&client)` and `YourIndexType::scan(&client)` helpers for projection-first access
- add `#[raiden(gsi(name = "...", partition_key = "...", sort_key = "..."))]` to the `RaidenIndex` type when you also want typed key condition helpers on the projection type itself
- typed GSI query/scan keeps the base struct projection by default; switch to an index projection explicitly with `project::<...>()`
- `client.query().user_index().project::<UserIndexItem>()` and `UserIndexItem::query(&client)` are equivalent entrypoints; choose whichever style is clearer for your call site
- `client.scan().user_index().project::<UserIndexItem>()` and `UserIndexItem::scan(&client)` are also equivalent entrypoints
- `run_with::<...>()` remains available as a backward-compatible convenience wrapper
- the legacy `.index("userIndex")` API is still available for backward compatibility, and existing typed GSI builders still default to the source-item projection unless you opt into a projection item
- composite GSI conditions must be chained in order: partition key -> sort key 1 -> sort key 2 ...
- range conditions such as `gt`, `between`, and `begins_with` are only allowed on the last sort key
- the old `.index("userIndex")` API is deprecated, but preserved for compatibility while migrating to typed GSI helpers

#### Query and scan with a typed LSI

Declare an LSI with the table's alternate sort-key field. The table must have
its own sort key, and the LSI shares the table partition key automatically:

```rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "events")]
#[raiden(lsi(name = "createdIndex", sort_key = "created_at"))]
struct Event {
    #[raiden(partition_key)]
    account_id: String,
    #[raiden(sort_key)]
    event_id: String,
    created_at: String,
    #[raiden(omit_lsi = "createdIndex")]
    private_note: String,
}

// `EventCreatedIndexItem` is generated from fields not marked `omit_lsi`.
let condition = Event::created_index_key_condition()
    .eq("account-1")
    .and(Event::created_index_sort_key_condition().begins_with("2026-"));
let result = client.query()
    .created_index()
    .project::<EventCreatedIndexItem>()
    .consistent()
    .key_condition(condition)
    .run()
    .await?;

let scan = client.scan()
    .created_index()
    .project::<EventCreatedIndexItem>()
    .consistent()
    .run()
    .await?;
```

`#[derive(RaidenIndex)]` also accepts `#[raiden(source = "Event", lsi = "createdIndex")]`
for a custom projection item. Add
`#[raiden(lsi(name = "createdIndex", partition_key = "account_id", sort_key = "created_at"))]`
on that projection type to generate its own typed key-condition methods. The
`partition_key` on a custom projection identifies the source table's key; on
the source `Raiden` type it is inferred and validated.

LSI queries and scans support `.consistent()`. Reading attributes that are not
projected into the LSI makes DynamoDB fetch them from the base table, increasing
read cost and latency. A GSI supports eventual consistency only. The library's
existing `.consistent()` method does not prevent selecting a GSI, so avoid that
combination.

## Support `tokio-rs/tracing`

`raiden` supports making span for Tracing ( span name is `dynamodb::action` with table name and api name in field ).  
To activate this feature, you need to specify `tracing` feature in your `Cargo.toml`. And your crate needs `tracing` .

```toml
# Example
[dependencies]
raiden = {
    tag = "0.0.76",
    git = "https://github.com/raiden-rs/raiden-dynamo.git",
    features = [ "tracing"]
}
tracing = "0.1"
```

## Development

### Requirements

- Rust (1.76.0+)
- Deno (1.13.2+)
- GNU Make
- Docker Engine

### Run tests

```
make test
```

NOTE: Don't recommend to use `cargo test` because our test suite doesn't support running tests in parallel. Use `cargo test -- --test-threads=1` instead of it.

### Run examples

```
make dynamo

AWS_ACCESS_KEY_ID=dummy AWS_SECRET_ACCESS_KEY=dummy cargo run --example EXAMPLE_NAME
```

### Utility

[dynamodb-admin](https://github.com/aaronshaf/dynamodb-admin) is useful to check data in DynamoDB Local.

```
npx dynamodb-admin
```

Then open `http://localhost:8001` in browser.

## Supported APIs

### Item

- [x] BatchGetItem
- [x] BatchWriteItem
- [x] DeleteItem
- [x] GetItem
- [x] PutItem
- [x] Query
- [x] Scan
- [x] TransactGetItems
- [x] TransactWriteItems
- [x] UpdateItem

## Known limitations

Here is a list of unsupported features/behaviors in the actual implementation.
We have a plan to resolve these issues in a future release.

- [x] Automatic retrying: https://github.com/raiden-rs/raiden/issues/44
- [x] Strict type checking of keys: https://github.com/raiden-rs/raiden/issues/26
- [x] Exponential backoff handling

## License

This project is available under the terms of either the [Apache 2.0 license](./LICENSE-APACHE) or the [MIT license](./LICENSE-MIT).
