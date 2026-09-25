# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Add attribute comparison operators, `BETWEEN`, `IN`, and negation of complete condition groups.

### Fixed

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
