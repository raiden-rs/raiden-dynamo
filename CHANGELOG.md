# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- Restore the safe-builder based `put_item_builder()` of 0.0.94 and earlier, and re-export safe-builder's derive as `raiden::Builder` again. 0.0.95 to 0.0.97 generated put builders with bon, which broke downstream code in three ways: setting an `Option` field conditionally (`if let Some(v) = opt { builder = builder.field(v) }`) no longer type-checked because every bon setter changes the builder type, the `Default<Struct>PutItemInputBuilder` type aliases disappeared, and `#[derive(raiden::Builder)]` with safe-builder style attributes such as `#[builder(setter(into))]` failed with `Unknown field: setter`. All three work again. bon stays available as `raiden::bon` and its derive as `raiden::BonBuilder`; see "Migrating from 0.0.95 - 0.0.97" in the README for code written against those releases.

### Added

- Add the `rusoto_native_tls` feature, an explicit name for the native-tls Rusoto backend (same as `rusoto`).
- Add attribute comparison operators, `BETWEEN`, `IN`, and negation of complete condition groups.

### Fixed

- Route the `rustls` and `rusoto_rustls` features to Rusoto's rustls backend again (`rusoto_core/rustls`, `rusoto_dynamodb/rustls`). 0.0.96 and 0.0.97 routed them to native-tls, so a crate that also enabled `rustls` on its own `rusoto_*` dependencies got both TLS backends and `rusoto_core` failed to compile (E0252 / E0599), and musl builds pulled in a non-vendored `openssl-sys`. As a consequence, the Dependabot alerts for rustls 0.20 / ring 0.16 that 0.0.96 silenced apply again to builds that select `rustls` / `rusoto_rustls`; they cannot be fixed within Rusoto 0.48. Builds with `rusoto` (the default, native-tls) or `aws-sdk` are unaffected. Use `aws-sdk` for a maintained rustls stack.
- Remove duplicate keys in `batch_get` before sending BatchGetItem requests. A duplicate within one 100-key request no longer fails with `ValidationException`, and a duplicate split across requests no longer returns the same item twice.
- Keep expression attribute names distinct when a condition combines different map keys with special characters.
- Preserve the underlying `ConversionError` as the source of `RaidenError::AttributeConvertError` on read failures. Pattern matches on this variant now need to account for its `source` field.

## @0.0.63 (12. April, 2022)

- Support `filter` expression for query and scan.
  You can pass `filter_expression` like following.
- Use tokio@1.17.0

``` rust
let filter = Scan::filter_expression(Scan::num()).eq(1000);
let res = client.scan().filter(filter).run().await.unwrap();
```
