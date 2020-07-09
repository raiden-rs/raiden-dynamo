<p align="center"><img src ="https://github.com/bokuweb/raiden/blob/master/assets/logo.png?raw=true" /></p>

<p align="center">
    DynamoDB library for Rust.
</p>

---

![Continuous Integration](https://github.com/bokuweb/raiden/workflows/Continuous%20Integration/badge.svg)

## Examples

### get_item example

```Rust
use raiden::*;

#[derive(Raiden)]
#[raiden(table_name = "user")]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn hello() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let res = client.get("user_primary_key").run().await;
    }
    rt.block_on(hello());
}
```

### put_item example

```Rust
fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn hello() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let input = User::put_item_builder()
            .id("mock_id".to_owned())
            .name("bokuweb".to_owned())
            .build()
            .unwrap();
        let res = client.put(&input).run().await;
    }
    rt.block_on(example());
}
```

### batch_get_item example

```Rust
use raiden::*;

#[derive(Raiden, Debug, PartialEq)]
pub struct User {
    #[raiden(partition_key)]
    id: String,
    name: String,
    #[raiden(sort_key)]
    year: usize,
    num: usize,
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn hello() {
        let client = User::client(Region::Custom {
            endpoint: "http://localhost:8000".into(),
            name: "ap-northeast-1".into(),
        });
        let keys: Vec<(&str, usize)> = vec![("Alice", 1992), ("Bob", 1976), ("Charlie", 2002)];
        let res = client.batch_get(keys).run().await;
    }
    rt.block_on(hello());
}
```

## Development

### Requirements

- Rust
- Node.js
- yarn
- GNU Make
- Docker Engine

### Setup

```
cd setup
yarn install
cd ..
make dynamo
```

### Test

```
make test
```

NOTE: Don't recommend to use `cargo test` because our test suite doesn't support running tests in parallel. Use `cargo test -- --test-threads=1` instead of it.

## Known limitations

Here is a list of unsupported features/behaviors in the actual implementation.
We have a plan to resolve these issues in a future release.

- [ ] Automatic retrying: https://github.com/raiden-rs/raiden/issues/44
- [ ] Strict type checking of keys: https://github.com/raiden-rs/raiden/issues/26
- [ ] Exponential backoff handling
