# Rusoto / AWS SDK operational compatibility audit

`raiden/tests/operational_compat.rs` runs the same scripted HTTP responses and
assertions in separate Rusoto and AWS SDK builds, without contacting AWS or
DynamoDB Local. It exercises generated clients, request serialization, response
decoding and Raiden error conversion. It does not prove complete SDK equivalence.

## Fixed shared pagination bug

Query and Scan previously treated only an absent `LastEvaluatedKey` as the end
of pagination. An explicit empty object caused another request without a useful
cursor; depending on responses, this could repeat data or continue indefinitely.
If a configured limit stopped the loop first, Query could expose an empty
continuation token and Scan could expose `Some(empty_map)`.

Both backends now normalize an empty key to absence, stopping after the final
page and returning no continuation key/token. This changes an existing Rusoto
bug as well as the SDK backend; it is not a migration-only difference.
DynamoDB documents the end condition as an empty key:
[Query API](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_Query.html).
The direct regression failed on both backends before the fix.

## Shared HTTP regression coverage

Seven tests cover:

- Empty intermediate pages with a nonempty key: continue, preserve returned
  items, and reduce Limit by ScannedCount rather than the number of items.
- A limit reached on an empty page: return a resumable Query token. Reusing it
  sends the original cursor as ExclusiveStartKey and returns subsequent items.
- Explicit empty terminal keys: stop without an extra request or a token.
- A model-based enumeration of 80 pagination scenarios per backend: Query/Scan
  × eight empty/populated page patterns × five limits (including no limit).
  The independent expected model checks items, terminal/resumable cursors,
  request counts, ExclusiveStartKey and each remaining Limit. Terminal keys
  alternate between absence and an explicit empty object.
- InternalServerError, ProvisionedThroughputExceededException and
  RequestLimitExceeded with an explicit two-retry Raiden policy: three identical
  requests, either eventual success or the expected final RaidenError variant.
- ConditionalCheckFailedException: no retry even with that explicit policy.
- TransactGet/TransactWrite cancellation: preserve all six recognized reason
  codes, None and an unknown future code in order; all six business-rule helper
  predicates remain true for their corresponding reason. The fixture has one
  transaction operation per cancellation reason. Uppercase Message and lowercase
  message are exercised across the transaction and ordinary error fixtures.

The server bounds request acceptance and reads; unexpected pagination/retries
cannot silently pass by consuming an unbounded sequence of canned responses.
Fixtures simulate service responses; they do not establish DynamoDB acceptance
of every generated query or error scenario.

## Differences and unverified risks

- **Default retries remain backend-specific.** Rusoto uses Raiden's retry
  policy; the SDK normally uses its own retry middleware. These tests disable
  SDK middleware retries to isolate the explicitly configured Raiden policy.
  They do not compare default attempt counts, jitter, retry delays, timeout or
  credential failures, or nested SDK/custom retry budgets. Batch retries are
  outside this new fixture suite; existing batch tests cover unprocessed keys
  and bounded write retries separately.
- **Cancellation predicates still depend on canonical message text on both
  backends.** The SDK additionally exposes raw structured reasons; normalized
  reasons currently parse the message, like Rusoto. An absent or changed message
  format can produce Unknown even with structured reason codes available. The
  canonical-message tests do not establish robustness against that format change.
- **Pagination counters retain their existing contract.** Items accumulate
  across pages; Count, ScannedCount and ConsumedCapacity reflect the final
  response rather than totals. The tests use valid nonnegative ScannedCount and
  small positive limits; missing counters, out-of-range limits, malformed or
  repeated nonempty cursors, and concurrent DynamoDB writes are not covered.
- **Transport and deployment behavior is unverified here.** Credentials/region
  resolution, IAM, HTTP/TLS, real AWS throttling, deadlines and cancellation are
  not established equivalent by scripted loopback responses.
- **Attribute conversion has separate coverage and known limitations.** See the
  [attribute audit](sdk-rusoto-attribute-compatibility.md), including the existing
  integer parse panic shared by both backends. Matching failures do not establish
  safe handling.

## Reproduction

```sh
cargo test --locked -p raiden --no-default-features --features rusoto --test operational_compat
cargo test --locked -p raiden --no-default-features --features aws-sdk --test operational_compat
```

These tests are included in the ordinary test suite, so the existing CI test
matrix runs them with both backends. Finite enumeration and HTTP fixtures are
regression evidence, not coverage-guided fuzzing or proof of equivalence.
