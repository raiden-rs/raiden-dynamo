<p align="center"><img src ="https://github.com/bokuweb/raiden/blob/master/assets/logo.png?raw=true" /></p>

<p align="center">
    DynamoDB library for Rust.
</p>

---

![Continuous Integration](https://github.com/bokuweb/raiden/workflows/Continuous%20Integration/badge.svg)

## Examples

### get_item example

``` rust
#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    let client = User::client(Region::UsEast1);
    let _res = client.get("user_primary_key").run().await;
}
```

### put_item example

``` rust
#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

#[tokio::main]
async fn main() {
    let client = User::client(Region::UsEast1);
    let input = User::put_item_builder()
        .id("foo".to_owned())
        .name("bokuweb".to_owned())
        .build();
    let res = client.put(&input).run().await;
}
```

### batch_get_item example

``` rust
#[derive(Raiden, Debug, PartialEq)]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    #[raiden(sort_key)]
    year: usize,
}

#[tokio::main]
async fn main() {
    let client = User::client(Region::UsEast1);
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

- Rust
- Deno (1.13.2+)
- GNU Make
- Docker Engine

### Setup

```
AWS_ACCESS_KEY_ID=awsdummy AWS_SECRET_ACCESS_KEY=awsdummy make dynamo
```

This starts up DynamoDB on Docker container, and then arranges test fixtures.

### Test

```
AWS_ACCESS_KEY_ID=awsdummy AWS_SECRET_ACCESS_KEY=awsdummy make test
```

NOTE: Don't recommend to use `cargo test` because our test suite doesn't support running tests in parallel. Use `cargo test -- --test-threads=1` instead of it.

### Example

```
AWS_ACCESS_KEY_ID=awsdummy AWS_SECRET_ACCESS_KEY=awsdummy  cargo run --example EXAMPLE_NAME
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
