# Rusoto / AWS SDK attribute compatibility audit

This audit covers Raiden's attribute conversions and the write regression fixed
by this PR. It does **not** establish complete behavioral equivalence between
the SDKs. Tests use the same fixtures and assertions in separate backend builds;
they compare parsed DynamoDB JSON, not JSON object ordering or generated
placeholder IDs. DynamoDB Local verifies persistence and validation separately.

## Fixed differences

| Input / operation | Rusoto behavior | Previous AWS SDK behavior | Behavior after this PR |
| --- | --- | --- | --- |
| Explicit empty list, including `Document<Vec<T>>` | Store `L: []` | Omit on Put; REMOVE on SET | Store `L: []` on both |
| Custom explicit empty SS/NS/BS | Send the value; DynamoDB rejects it | Silently omit/remove it | Send it and receive validation errors on both |
| Explicit `NULL: false` read as String, Cow, Option, Vec, a built-in set, or Document (including nested values) | Reject the type | Treat it as NULL / an empty value | Reject it on both; only `NULL: true` is NULL |
| BatchPutOutput JSON containing L/SS/NS/BS | Correct DynamoDB JSON | Extra array nesting; typed objects inside sets | Same JSON shape and successful round-trip on both |
| BatchPutOutput JSON containing boolean NULL flags | Decode either flag | Reject both flags (decoder expected JSON null) | Preserve either flag on both |
| Nested output metadata (PutRequest, DeleteRequest, KeysAndAttributes, Capacity/ConsumedCapacity, ItemCollectionMetrics) | PascalCase fields; omit unset fields | snake_case fields; emit null for unset fields | Match Rusoto field names and omission; accept old SDK input aliases |
| Capacity metadata with optional JSON null fields | Decode None | Reject null fields emitted by its own serializer | Decode None on both |

The omission implementation also prevents an omitted operand of arithmetic SET
from becoming REMOVE. Arithmetic with an omitted value sends a NULL operand and
fails validation; it never deletes the target. This preserves the failure
outcome of invalid arithmetic, not the old invalid payload or error text.

**Output JSON migration:** AWS SDK serialized nested metadata now uses Rusoto's
`Item`, `Key`, `Keys`, `TableName`, etc., and omits unset metadata fields. This
changes the previous SDK output format. Consumers inspecting those JSON keys
must update. The SDK decoder still accepts its previous snake_case metadata
keys. Outer Raiden output fields keep their existing snake_case names.

The shared `nested_output_metadata_matches_rusoto_json` test covers BatchGet
unprocessed keys and projection settings, BatchDelete unprocessed keys, and
Update item collection metrics (populated and absent optional metadata).
`legacy_sdk_metadata_is_still_readable` separately checks old SDK metadata input.

The generated `use_default` response path also used a different NULL check.
Get, BatchGet, TransactGet and Update now default only for missing values or
NULL:true; NULL:false errors. A shared HTTP response test covers all four paths
with missing, true, false and populated values (16 cases).

## Confirmed unchanged behavior

| Rust input / expression | Result on both backends | Evidence |
| --- | --- | --- |
| Empty owned String | `S: ""` | `existing_empty_scalar_and_option_conversions_are_unchanged` |
| Empty `&str`, Cow, ordinary Vec; Option::None | `NULL: true` | Same test plus captured write requests |
| Empty built-in HashSet/BTreeSet field, including Some(empty) | Omit on Put/BatchPut/transactional Put; REMOVE on ordinary/transactional SET | `attribute_write_compat` |
| Nonempty string sets and all 10 supported integer set types, ordered/unordered | Preserve SS/NS; their empty forms omit | `nonempty_builtin_sets_are_preserved` |
| Explicit empty map/list, nonempty containers, nested documents, S/N/B/BOOL/NULL and sets | Preserve the supplied typed value | 19 shared fixtures × 5 write paths = 95 request cases |
| Custom `into_attr_option() -> None`, including Some(custom) | Omit on all three Put paths; REMOVE on both SET paths | `custom_omission_propagates_through_option_and_all_write_paths` |
| Explicit values in equality conditions, equality filters, and SET if_not_exists | Retain the supplied operand | Same 19 fixtures in `condition_filter_and_if_not_exists_preserve_explicit_values` |
| Ordinary ADD/DELETE builders | Skip built-in empty sets and NULL; retain explicitly typed empty sets for service validation | `add_delete_preserve_explicit_empty_sets_and_skip_builtin_empty_sets` |
| NULL:true / NULL:false / missing reads | True is NULL, false is rejected; missing String errors, missing Option is None, missing Vec/set is empty | `null_flag_must_be_true_to_read_as_null` |

The 19 write fixtures include empty and nonempty binary values and binary sets,
both boolean values, both NULL flags, empty/nonempty string and numeric sets,
an empty map/list, and nested maps/lists/nulls. This is a boundary matrix, not an
exhaustive enumeration of arbitrary custom converters or expression combinations.
Invalid inputs captured in HTTP tests (empty sets, NULL:false, etc.) are not
claimed to be accepted by DynamoDB.

## DynamoDB Local evidence

The same three integration tests run with each backend:

- `document_list_survives_zero_one_zero_and_all_write_paths`: read after an empty
  Document Put without `use_default`, 0 → 1 → 0 ordinary updates, BatchPut and
  transactional Put, and 1 → 0 transactional updates.
- `custom_empty_sets_are_rejected_instead_of_silently_omitted`: SS/NS/BS across
  Put, BatchPut, transactional Put, Update and transactional Update (15 rejected
  requests). No silent success.
- `omitted_arithmetic_value_is_rejected_and_does_not_delete_existing_data`:
  invalid arithmetic fails and a consistent read still returns the original value.

The original AWS SDK implementation failed the empty-list and invalid-set
regressions. The follow-up read/output fixtures also failed before their fixes:
NULL:false classification, both NULL flag decodes, collection JSON, and capacity
metadata decoding. The Rusoto run also exposed the nested metadata field-name
difference; canonical fixtures now run unchanged on both backends.

## Remaining differences and migration requirements

1. **The raw AttributeValue APIs differ.** Rusoto uses a struct that can represent
   zero or several type fields; the AWS SDK uses a tagged enum. The SDK cannot
   represent Rusoto's `AttributeValue::default()` omission sentinel. Custom
   converters must use `into_attr_option` to express omission. SDK Unknown
   variants and direct native serde calls remain backend-specific. Raiden's
   raw JSON entry points reject malformed zero/multi-type attributes on both
   backends; this intentionally tightens previous Rusoto acceptance.
2. **Omission is contextual, not recursive.** Nested Vec/map elements, condition
   and filter operands, key values, and arithmetic's added operand use
   `into_attr`, not `into_attr_option`. A nested built-in empty string set is
   encoded as `{}` by Rusoto and `{"SS":[]}` by the SDK (numeric sets similarly).
   Both are invalid DynamoDB values, but the payload/error text differs. This
   difference is pinned by
   `nested_builtin_empty_sets_retain_the_documented_backend_difference`.
   Use Document to represent empty lists/maps rather than nesting empty sets.
3. **Wrappers must delegate omission.** A custom type delegating only
   `into_attr()` to a built-in empty set still has the raw representation
   difference above. Delegate `into_attr_option()` too when omission is intended.
4. **Transactional DELETE has a separate API.**
   `Model::update(key).delete(attribute, value)` calls `into_attr()` directly,
   unlike the ordinary DELETE expression builder. It does not implement
   empty-set omission. The invalid empty built-in set representation therefore
   differs as described in (2); this PR does not change that Rusoto behavior.
5. **Old omitted data is not repaired.** Missing Document fields can still fail
   decoding. Keep `#[raiden(use_default)]` where historical records need a
   default; this PR fixes future writes, not stored data.
6. **Raw JSON validation intentionally tightens both decoders.** At Raiden's
   output/token JSON boundaries, `{}`, unknown tags, multiple tags, null
   payloads (including `{"NULL":null}`), and wrong payload types now fail on
   both backends, including nested L/M values. The SDK no longer coerces
   NULL:null to true. Previously Rusoto accepted zero/multi-type attributes
   and ignored unknown fields; those malformed inputs must be repaired before
   replay. The six entry points are BatchPut unprocessed items, BatchDelete
   unprocessed keys, BatchGet unprocessed keys, Update item collection metrics,
   Scan last evaluated keys, and NextToken decoding. Token failures are
   NextTokenDecodeError; output failures are serde errors. Shared tests
   `malformed_raw_attributes_are_rejected_at_every_json_boundary` and
   `raw_json_validation_preserves_explicit_empty_values_and_null_flags` cover
   all six. The latter also verifies that ordinary model JSON is unaffected.
   Valid metadata still accepts old SDK snake_case aliases. Duplicate JSON
   object keys, every output DTO, and every metadata combination are not
   exhaustively compared. Direct native Rusoto serde remains permissive;
   Raiden does not replace the backend's exposed AttributeValue type.
7. **SDK operational behavior is outside this attribute audit.** A separate
   [operational audit](sdk-rusoto-operational-compatibility.md) covers scripted
   pagination, explicit retry policies and transaction cancellation reasons.
   Credentials/region resolution, default retries, timeouts, HTTP/TLS behavior,
   pagination under concurrent
   writes, throttling, IAM and real AWS service failures have not been proven
   equivalent. Error classification is checked for the listed Local validation
   failures; exact SDK/service error messages are not promised to match.

## Seeded differential exploration

`scripts/check-attribute-backends.py` generates the same corpus once, then builds
and runs the Rust probe separately with Rusoto and AWS SDK. It compares parsed
output JSON, read success/error/panic, successful read values, and their optional
write conversion serialized back to typed DynamoDB JSON. It checks 19 Rust
conversions per input, including all supported scalar integer widths, f32/f64,
String/Option/Vec, ordered string/integer sets, and Document JSON.

The generator explores all 10 AttributeValue tags, empty/nonempty sets and
containers, recursion up to four levels, Unicode/control characters, arbitrary
binary bytes, numeric boundaries/fractions/invalid strings, both NULL/BOOL flags,
and absent/null/populated canonical capacity metadata. Invalid service values
are intentional; this is an SDK/converter comparison, not service acceptance.

The first 10,000-input run (seed 20261120) found **557 differing inputs** in
Document NULL:false decoding, including nested forms. The smallest observed
counterexample was `{"NULL":false}`. Fixing this uncovered path removed those
differences; dedicated root/list/map regression cases preserve the fix.
This is seeded random exploration, not coverage-guided fuzzing, and the script
reports the smallest observed examples rather than claiming automatic shrinking.

**Common existing failure:** integer scalar conversions use `parse().unwrap()`
on both backends. Fractions, out-of-range numbers and malformed numeric strings
can panic on both. The probe records panic separately from ordinary conversion
errors; matching panics do not demonstrate correct handling. This PR preserves
that existing behavior instead of changing Rusoto's numeric conversion contract.

Use `--include-malformed` to explore missing/multiple/unknown type tags,
wrong payload types, invalid base64 and NULL:null, at the root and nested in
L/M. These are not allowlisted: any difference fails the run. The initial
2,000-input root-only malformed corpus found 208 differences across four
patterns; the common boundary validator fixes them by rejecting the malformed
attributes on both backends, rather than coercing them to valid values.
This changes prior Rusoto acceptance and prior SDK NULL:null acceptance.
Replaying the exact original 2,000 inputs after the fix produces zero
mismatches; all 208 original counterexamples now fail decoding on both backends.
An independent 10,000-input run with malformed attributes (seed 20261126)
also found zero mismatches and recorded a read-conversion panic on 692 inputs
on each backend.
Every corpus, both observation files and `differences.json` are retained in
the reported artifact directory.

```sh
python3 scripts/check-attribute-backends.py --seed 20261120 --cases 10000
python3 scripts/check-attribute-backends.py --cases 10000 --include-malformed
```

CI runs both 10,000-input corpora. Successful finite exploration must not be
generalized to every Rust type/expression, arbitrary JSON or nesting, duplicate
JSON object keys, request transport behavior, or AWS operational behavior.

## Reproduce

Use Rust 1.91.1 (the CI toolchain). For each of `rusoto` and `aws-sdk`:

```sh
cargo +1.91.1 test -p raiden --no-default-features --features aws-sdk \
  --test attribute_read_compat --test attribute_write_compat
# Requires the repository's DynamoDB Local fixture setup on localhost:8000:
cargo +1.91.1 test -p raiden --no-default-features --features aws-sdk \
  --test mod attribute_compat
```

Replace `aws-sdk` with `rusoto` for the other backend. The repository CI additionally
runs both complete suites and checks all supported feature combinations.
