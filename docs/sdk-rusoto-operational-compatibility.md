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

Eleven ordinary tests cover (two additional corpus probes run explicitly):

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

## Additional differences found and fixed

The HTTP differential probe compares generated request JSON, serialized read
outputs, error variants/messages, normalized cancellation reasons and the shared
retry policy's decision. Unknown raw error payload/debug types and headers are
backend-specific and are not compared. JSON object ordering is ignored; request
tokens are retained in raw evidence. The documented SDK-only ClientRequestToken
is validated and excluded only from cross-backend payload equality.

- SDK modeled-error Display text included the exception name; Rusoto exposes the
  bare message. InvalidEndpointException was incorrectly mapped to InternalServerError,
  and TransactGet ThrottlingException to RequestLimitExceeded; Rusoto returns
  Unknown for both. These are now aligned per operation, including retry decisions.
- SDK accepts code/Code, differing type precedence and malformed message fields.
  The compatibility decoder now uses the raw HTTP error body with Rusoto's strict
  optional string schema, Message alias and last-`#` type suffix. Duplicate message
  aliases and invalid message types return Unknown. Invalid SDK cancellation
  details no longer erase a Rusoto-readable TransactionCanceled classification.
  Raw SDK reasons remain available when its structured decoder succeeds.
- **Intentional safety difference:** SDK WriteTx sends ClientRequestToken;
  Rusoto sends none. Keep this AWS idempotency protection rather than matching
  Rusoto's omission. Generate one UUID per WriteTx::run and reuse it across SDK
  middleware and explicit Raiden retries; separate runs receive fresh tokens.
  Regression tests check complete payload equality after removing only this field,
  token presence/format, stability across three explicit attempts and six nested
  SDK/Raiden attempts, and distinct tokens for distinct runs. This does not make
  separate run calls idempotent; DynamoDB's token validity window also applies.
- SDK Query/Scan synthesize zero for missing/null Count and ScannedCount, whereas
  Rusoto returns None. An interceptor records their presence from the buffered
  response. An operation-local interceptor retains the original client's
  connection pool, runtime plugins, retry state and existing interceptors, with
  separate counter state for each page/request.
  Explicit zero remains Some(0). A two-page cross-product regression checks that
  earlier values do not leak into absent final counters. A separate SDK test
  verifies that caller-installed response interceptors still execute; another
  populates all 17 Query and 16 Scan SDK input fields and compares the complete
  request JSON with the SDK's direct send_with path.

Exploration reproduced 1,620 differing observations in an initial 3,300-case
error matrix (300 also differed in transaction request JSON). An additional
2,000 seeded malformed/alternative error-body cases revealed 470 remaining
differences. After the fixes, the combined 5,300 cases had zero unexpected differences
(the SDK-only idempotency token is the explicit exception described above).
The initial successful-read matrix found 216 differences in 333 cases, all due
to optional counters; after fixing them, an expanded 373-case matrix has zero
observed differences. An independent seed (20261125) with 10,000 exploratory
cases plus the 3,300-case baseline also has zero unexpected differences (13,300 total, with the same token exception).
These are finite structured explorations, not arbitrary
byte fuzzing. Some fixtures intentionally describe service-invalid combinations
so decoding behavior can be compared independently of AWS validation.

## Differences and unverified risks

- **Default retries remain backend-specific.** Rusoto uses Raiden's retry
  policy; the SDK normally uses its own retry middleware. These tests disable
  SDK middleware retries to isolate the explicitly configured Raiden policy.
  A separate SDK regression checks stable transaction tokens with nested SDK/Raiden retries.
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
  small positive limits; out-of-range limits, malformed or
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

Run and preserve the HTTP exploration evidence with:

```sh
python3 scripts/check-operational-backends.py --output-dir /tmp/raiden-errors
python3 scripts/check-operational-backends.py --responses --output-dir /tmp/raiden-responses
# Broader independent seed; the 3,300-case baseline is always included.
python3 scripts/check-operational-backends.py --seed 20261125 --exploratory-cases 10000 --output-dir /tmp/raiden-errors-seed2
```

The default toolchain is CI's 1.91.1; pass `--toolchain 1.100.0` to reproduce
these local runs. Each directory contains every input, both observations and
`differences.json`; any difference fails the command. CI runs both HTTP corpora
and uploads their evidence, including on failure. Native malformed **successful**
HTTP payloads are not validated by the separate raw-attribute JSON entrypoints;
matching their entire decoder failure behavior remains outside this audit.
