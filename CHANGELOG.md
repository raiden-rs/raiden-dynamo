# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- **Breaking:** remove the `rustls` / `rusoto_rustls` features and their unpatched Rusoto TLS dependencies. Use `aws-sdk` for modern rustls, or `rusoto` / `rusoto_native_tls` to retain the Rusoto API with native TLS. See the README migration guide for dependency and client-construction changes.
- Raise the minimum Rust version to 1.91.1 for the patched AWS SDK dependencies.

- Validate raw DynamoDB attribute JSON at Raiden's output/token decode boundaries on both backends. Missing, unknown or multiple type tags and invalid payload types (including `NULL:null`) now fail decoding, including nested L/M values. This tightens previous Rusoto acceptance and removes the SDK's implicit `NULL:null` to `NULL:true` conversion. See README for affected APIs and migration details; direct native backend serde is unchanged.

- Restore the safe-builder based `put_item_builder()` of 0.0.94 and earlier, and re-export safe-builder's derive as `raiden::Builder` again. 0.0.95 to 0.0.97 generated put builders with bon, which broke downstream code in three ways: setting an `Option` field conditionally (`if let Some(v) = opt { builder = builder.field(v) }`) no longer type-checked because every bon setter changes the builder type, the `Default<Struct>PutItemInputBuilder` type aliases disappeared, and `#[derive(raiden::Builder)]` with safe-builder style attributes such as `#[builder(setter(into))]` failed with `Unknown field: setter`. All three work again. bon stays available as `raiden::bon` and its derive as `raiden::BonBuilder`; see "Migrating from 0.0.95 - 0.0.97" in the README for code written against those releases.

### Added

- Add the `rusoto_native_tls` feature, an explicit name for the native-tls Rusoto backend (same as `rusoto`).
- Add attribute comparison operators, `BETWEEN`, `IN`, and negation of complete condition groups.

### Fixed

- Update AWS SDK dependencies to use aws-smithy-json 0.62.7 (GHSA-8ffr-xgwf-xj56) and modern rustls to 0.23.46, removing all identified vulnerable rustls / ring versions from both lockfiles.
- Update the AWS SDK's h2 dependency to 0.4.16 for GHSA-q83h-524g-xf6h. Rusoto's Hyper 0.14 dependency still requires h2 0.3, which has no patched compatible release for this advisory.

- Align AWS SDK NULL flag reads and BatchPutOutput JSON round-trips with Rusoto, including nested lists, sets, and optional capacity metadata. Document the tested scope and remaining backend differences.
- Compare seeded random attribute corpora across backends in CI; fix additional Document and generated use_default NULL:false decoding differences found during the audit.
- Match Rusoto nested output JSON field names and omit unset metadata. This changes previous SDK output keys; decoding retains legacy SDK snake_case aliases.
- Match Rusoto's write behavior on the AWS SDK backend: preserve explicit empty lists (including `Document<Vec<T>>`) and pass custom empty sets to DynamoDB for validation instead of silently omitting them. Built-in empty sets still omit/remove attributes. Add `IntoAttribute::into_attr_option` for deliberate custom omission, including through `Option`, across puts, batches, transactions, and updates.

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
