# T031 Red evidence — Get/Locate negative vectors

## Scope and commit

Added only `crates/kmipkit-protocol/tests/object_read_negative_vectors.rs`.
No production code, dependencies, generated files, specifications, or pinned
OASIS upstream files were changed.

Red test commit: `b039361d77eff9e99ebe1ebc91eb7f679a5d9860`
(`test(protocol): add object-read negative vectors`, signed off).

## Normative and feature traceability

- OASIS KMIP Specification v2.1 §10.1.2 requires defined Structure fields to
  appear in the order in which they appear in the Structure description.
  Section 6.1.19, Table 221 orders Get success fields as Object Type, Unique
  Identifier, then Any Object. The out-of-order vector also covers
  KMIPKIT-0017-FR-002.
- OASIS KMIP Specification v2.1 §6.1.28, Table 248 defines optional Located
  Items followed by zero or more repeatable Unique Identifier fields. The
  Locate vectors cover duplicate and mistyped Located Items, invalid field
  order, invalid identifier type, and an unrecognized byte-valued field;
  repeated Unique Identifiers remain permitted.
- KMIPKIT-0017-FR-004 and FR-012 apply to Get's opaque secret-bearing Any
  Object and sanitized conversion diagnostics. FR-012 also applies to
  configured generic TTLV bounds. Shared decoder checks use
  KMIPKIT-0005-FR-006–FR-009; decoder bounds are project policy, not an
  operation-specific OASIS requirement.
- FR-011 result reason/message and Pending delivery behavior is not a target
  of these malformed-success-payload vectors. They use validated Success
  metadata and do not alter the separate status/delivery contract.

## Vectors and sanitizer assertions

- Get conversion vectors cover missing required Object Type, duplicate Object
  Type and Unique Identifier, mistyped Object Type, mistyped Unique Identifier,
  mistyped Any Object, and Table 221 field order. Existing Get tests already
  rejected the missing/duplicate/mistyped forms; these new cases strengthen
  the error assertions with sentinel checks.
- Locate conversion vectors cover duplicate and mistyped Located Items,
  Located Items after Unique Identifier, an invalid Unique Identifier type,
  and an unexpected byte-valued payload field.
- Response-shaped Get and Locate wire inputs cover truncated TTLV, unsupported
  Item Type, and per-call byte, Structure-depth, and Item-count limit failures.
  The tests call the public `decode_with_limits` decoder directly.
- Error diagnostics collect Display, Debug, and every available `Error::source`
  in the chain. Assertions check that the unique text sentinel and the unique
  byte sentinel are absent as UTF-8, contiguous hexadecimal, and decimal-byte
  forms. These assertions passed for the malformed conversion and decoder
  cases in the focused run.

## Exact verification

Focused Red command, run after formatting:

```text
cargo test -p kmipkit-protocol --test object_read_negative_vectors
```

Result: exit code 1; 7 tests ran, 6 passed, and 1 failed. The sole expected
failure is `get_success_payload_rejects_table_221_fields_out_of_order`.
The current public conversion accepts Object Type, Any Object, then Unique
Identifier and returns a successful `GetResponse`; this contradicts §10.1.2 and
Table 221 ordering as required by FR-002. No other failure remained in the
final focused run.

Passing cases include the malformed Get and Locate conversion sanitization,
truncated and malformed generic TTLV sanitization, and configured byte, depth,
and element limits for both response shapes.

Formatting and staged-diff checks:

```text
cargo fmt --all --check     exit code 0
git diff --cached --check   exit code 0
```

During test setup, the initial success fixture incorrectly included a Result
Message, which the validated response contract disallows for Success; that
field was removed. The first truncation vector appended after the root and
correctly produced TrailingBytes; it was changed to truncate a child inside the
response-shaped root. A successful ResponseMessage without a Response Payload
is rejected by message validation before the operation converter, so that
unreachable case was removed. The required missing Get field remains covered;
Locate Table 248 response fields are optional, and its empty payload is valid.

## Limitations

This is Red-only evidence: no production fix was made. The ordering assertion
is intentionally failing for the Green task. Decoder limit vectors use small
per-call limits against response-shaped TTLV and do not exercise transport
integration. A missing whole Response Payload cannot be passed through the
validated public response-conversion path; the protocol message validator
rejects it first.
