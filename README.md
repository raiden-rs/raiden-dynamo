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
