# KMIPKIT-0053 Traceability

Exact source tables are inherited from specs 048–052 and reference the pinned OASIS KMIP 2.1 Standard. Project API choices do not create new OASIS requirements.

| ID | OASIS source | Requirement | Implementation | Verification |
|---|---|---|---|---|
| KMIPKIT-0053-R001 | Attribute tables in specs/048-typed-attributes/traceability.md; §10.1.2 | Preserve direct attribute tag, TTLV value alternative, order, cardinality, and local constraints. | C ABI, Java JNI, Python CFFI over Rust definitions. | 62-definition matrix, invalid tag/type and generic fallback tests. |
| KMIPKIT-0053-R002 | Object/key/wrapping tables in specs/049-managed-objects/traceability.md; §10.1.2 | Preserve object tags, field order/types and unknown children. | Opaque C handles and Java/Python owners over Rust. | Nine-object/key roundtrips and order checks. |
| KMIPKIT-0053-R003 | §4.16 Table 59 and affected §6.1 payload tables in specs/050-parameters-auth-client/traceability.md | Preserve Cryptographic Parameters types/order and future raw enum values. | C ABI/JNI/CFFI over shared Rust model. | All 18 fields, future values and extensions. |
| KMIPKIT-0053-R004 | §8.2 Table 395; §§8.1, 8.3 Tables 394, 396 | Authentication appears once in the Request Header for the message and its batch items. | Foreign batch setters call Rust request builder. | Encoded absent/present one-/multi-item tests. |
| KMIPKIT-0053-R005 | §6.1 operation payloads; §§8.1–8.3; §10.1.2; specs/051-pre-send-validation/traceability.md | Reject locally decidable normative violations before I/O; never guess generic schemas; omit values in reports. | All adapters use Rust validation. | Issue parity, NotSent, zero writes, valid-wire stability, redaction. |
| KMIPKIT-0053-R006 | §6.1 tables for 56 supported operations and async rules in specs/050-parameters-auth-client/gap-matrix.md | Preserve operation identity, request/options, response association and async correlation. | C ABI/JNI/CFFI wrappers. | 56-operation matrix and no-retry tests. |
| KMIPKIT-0053-R007 | §8.5 Table 398; §8.6 Table 399; §§9.13, 10.1.2; §11.56; specs/052-response-navigation/traceability.md | Preserve response presence, order, repetitions, unknown/vendor values at arbitrary depth. | KMIPKIT-0052 handles and wrappers. | Absent/empty/Pending and nested 0x54xxxx tests in four languages. |

## Project Decisions
- P001: Rust remains the sole protocol/model/validation implementation.
- P002: Idiomatic language naming is allowed only with exact mappings and runnable tests.
- P003: Generic TTLV remains the lossless route for unknown/future/vendor data.
- P004: Authentication is caller-owned, one-message scoped, never client-global.
- P005: Foreign owners have explicit close/release; only KMIPKit-owned copies are promised zeroization.
- P006: Generated outputs are written only by the pinned generator.

## Verification Record
Add exact commands/platforms before the draft PR update. Coverage and hosted CI remain release gates, temporarily deferred by direct instruction.
