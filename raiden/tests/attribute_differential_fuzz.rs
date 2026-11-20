//! Probe run by scripts/check-attribute-backends.py in two separate feature builds.
use raiden::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, Write};

fn encode(attribute: AttributeValue) -> Value {
    let item = [("raw".to_owned(), attribute)].into_iter().collect();
    #[cfg(feature = "aws-sdk")]
    let request = aws_sdk::types::PutRequest::builder()
        .set_item(Some(item))
        .build()
        .unwrap();
    #[cfg(any(feature = "rusoto", feature = "rusoto_rustls"))]
    let request = PutRequest { item };
    let output = raiden::ops::batch_put::BatchPutOutput {
        consumed_capacity: None,
        unprocessed_items: vec![request],
    };
    serde_json::to_value(output).unwrap()["unprocessed_items"][0]["Item"]["raw"].clone()
}

fn read<T: FromAttribute + IntoAttribute + serde::Serialize>(attribute: &AttributeValue) -> Value {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        T::from_attr(Some(attribute.clone()))
    })) {
        Ok(Ok(value)) => {
            let serialized = serde_json::to_value(&value).unwrap();
            let write = value.into_attr_option().map(encode);
            json!({"ok": serialized, "write": write})
        }
        Ok(Err(_)) => json!({"error": true}),
        Err(_) => json!({"panic": true}),
    }
}

#[test]
#[ignore = "requires corpus/output paths; use scripts/check-attribute-backends.py"]
fn emit_differential_observations() {
    // Record panics as observations so a single backend panic cannot hide later differences.
    std::panic::set_hook(Box::new(|_| {}));
    let corpus = std::fs::File::open(std::env::var("RAIDEN_FUZZ_CORPUS").unwrap()).unwrap();
    let mut output = std::io::BufWriter::new(
        std::fs::File::create(std::env::var("RAIDEN_FUZZ_OUTPUT").unwrap()).unwrap(),
    );
    for line in std::io::BufReader::new(corpus).lines() {
        let input: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let result = match serde_json::from_value::<raiden::ops::batch_put::BatchPutOutput>(input) {
            Err(_) => json!({"decode_error": true}),
            Ok(decoded) => {
                let attribute = &decoded.unprocessed_items[0].item["raw"];
                json!({
                    "output": serde_json::to_value(&decoded).unwrap(),
                    "reads": {
                        "string": read::<String>(attribute),
                        "option": read::<Option<String>>(attribute),
                        "vector": read::<Vec<String>>(attribute),
                        "strings": read::<BTreeSet<String>>(attribute),
                        "numbers": read::<BTreeSet<i64>>(attribute),
                        "bool": read::<bool>(attribute),
                        "u8": read::<u8>(attribute), "i8": read::<i8>(attribute),
                        "u16": read::<u16>(attribute), "i16": read::<i16>(attribute),
                        "u32": read::<u32>(attribute), "i32": read::<i32>(attribute),
                        "u64": read::<u64>(attribute), "i64": read::<i64>(attribute),
                        "usize": read::<usize>(attribute), "isize": read::<isize>(attribute),
                        "f32": read::<f32>(attribute), "f64": read::<f64>(attribute),
                        "document": read::<Document<Value>>(attribute),
                    }
                })
            }
        };
        serde_json::to_writer(&mut output, &result).unwrap();
        writeln!(output).unwrap();
    }
    output.flush().unwrap();
}
