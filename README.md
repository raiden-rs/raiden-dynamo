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

#### With rusoto_dynamodb ( `rusoto` or `rusoto_rustls` or `rustls` is enabled)

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

NOTE: Default retry strategy differs between `aws-sdk` and `rusoto` ( or `rusoto_rustls` )

- `aws-sdk` ... Not retry in raiden by default. Because you can configure retry strategy using `aws_config`. Or you can configure your own strategy like next example.
- `rusoto` or `rusoto_rustls` ... Enabled retrying in raiden by default. See detail [here](https://github.com/mythrnr/raiden-dynamo/blob/master/raiden/src/retry/mod.rs).

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
- `Document<T>` remains available as an explicit wrapper when you prefer opt-in at the field type level
- empty maps are preserved as empty DynamoDB `M` values rather than being dropped

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
- [ ] BatchWriteItem
- [x] DeleteItem
- [x] GetItem
- [x] PutItem
- [x] Query
- [x] Scan
- [ ] TransactGetItems
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
