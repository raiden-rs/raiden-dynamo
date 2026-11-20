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
| Explicit `NULL: false` read as String, Cow, Option, Vec, or a built-in set | Reject the type | Treat it as NULL / an empty value | Reject it on both; only `NULL: true` is NULL |
| BatchPutOutput JSON containing L/SS/NS/BS | Correct DynamoDB JSON | Extra array nesting; typed objects inside sets | Same JSON shape and successful round-trip on both |
| BatchPutOutput JSON containing boolean NULL flags | Decode either flag | Reject both flags (decoder expected JSON null) | Preserve either flag on both |
| BatchPutOutput capacity metadata with optional JSON null fields | Decode None | Reject null fields emitted by its own serializer | Round-trip the tested capacity metadata on both |

The omission implementation also prevents an omitted operand of arithmetic SET
from becoming REMOVE. Arithmetic with an omitted value sends a NULL operand and
fails validation; it never deletes the target. This preserves the failure
outcome of invalid arithmetic, not the old invalid payload or error text.

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
metadata decoding.

## Remaining differences and migration requirements

1. **The raw AttributeValue APIs differ.** Rusoto uses a struct that can represent
   zero or several type fields; the AWS SDK uses a tagged enum. The SDK cannot
   represent Rusoto's `AttributeValue::default()` omission sentinel. Custom
   converters must use `into_attr_option` to express omission. SDK Unknown
   variants and malformed multi-type Rusoto values are outside this contract.
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
6. **JSON compatibility is tested for the listed fixtures.** The SDK decoder
   continues accepting legacy `{"NULL":null}` as NULL:true. This permissive
   legacy input is not a claim of exact Rusoto decoding equivalence. Unknown
   fields, duplicate fields, malformed attributes, every output DTO, and every
   metadata combination are not exhaustively compared here.
7. **SDK operational behavior is outside this audit.** Credentials/region
   resolution, retries, timeouts, HTTP/TLS behavior, pagination under concurrent
   writes, throttling, IAM and real AWS service failures have not been proven
   equivalent. Error classification is checked for the listed Local validation
   failures; exact SDK/service error messages are not promised to match.

## Reproduce

Use Rust 1.88.0 (the CI toolchain). For each of `rusoto` and `aws-sdk`:

```sh
cargo +1.88.0 test -p raiden --no-default-features --features aws-sdk \
  --test attribute_read_compat --test attribute_write_compat
# Requires the repository's DynamoDB Local fixture setup on localhost:8000:
cargo +1.88.0 test -p raiden --no-default-features --features aws-sdk \
  --test mod attribute_compat
```

Replace `aws-sdk` with `rusoto` for the other backend. The repository CI additionally
runs both complete suites and checks all supported feature combinations.
