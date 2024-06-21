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
