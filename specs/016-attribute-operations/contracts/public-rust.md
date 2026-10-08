# Public Rust Contract: KMIP 2.1 Attribute Operations

This document describes the Rust protocol/client capability expected from KMIPKIT-0016. Exact Rust names may follow repository naming conventions, but they must preserve these observable operation distinctions and fields.

## Operations

| Operation | Request data | Successful payload |
| --- | --- | --- |
| `AddAttribute` | Optional Unique Identifier; one `NewAttribute` | Unique Identifier |
| `AdjustAttribute` | Optional Unique Identifier; `AttributeReference`; `AdjustmentType`; optional `AdjustmentValue` | Unique Identifier |
| `DeleteAttribute` | Optional Unique Identifier; optional `CurrentAttribute`; optional `AttributeReference` | Unique Identifier |
| `GetAttributes` | Optional Unique Identifier; list of distinct optional `AttributeReference` values | Unique Identifier and all returned `AttributeEntry` values |
| `GetAttributeList` | Optional Unique Identifier only | Unique Identifier and one or more returned Attribute References |
| `ModifyAttribute` | Optional Unique Identifier; optional `CurrentAttribute`; one `NewAttribute` | Unique Identifier |
| `SetAttribute` | Optional Unique Identifier; one `NewAttribute` | Unique Identifier |

The operation name is explicit in the request variant and encoded Operation Enumeration. The shared client returns the corresponding typed response plus the common operation result.

## Observable guarantees

- Request and response field presence, TTLV tags/types, multiplicity, and order follow the cited OASIS operation tables.
- Caller values are not changed, defaulted, narrowed, or selected by the client. Exact unknown Attribute Names and values with assigned/accepted extension tags remain available through the generic model; raw tags failing the KMIPKIT-0004 allocation gate cannot be encoded. Generic Enumeration values remain lossless, while outbound Adjustment Type accepts only assigned values or the Table 429 `0x80000000–0x8FFFFFFF` extension range.
- Unconditional standard-attribute prohibitions and operation-specific client prohibitions are checked from exact source-backed metadata before transmission. This includes rejecting Add/Modify when New Attribute is Usage Limits, because §7.40 Table 392 requires Count and §4.59 forbids the client from setting or modifying it through those operations. A supplied Vendor Attribute value with Vendor Identification `y` is rejected under §4.60/Table 150 when present in a value-bearing mutation field; Adjust and reference-only Delete cannot expose the identifier, so the server result is preserved. A local rejection is payload-free and reports delivery state `NotSent`. Rules that depend on remote object state remain server-authoritative; client request construction does not guess them or issue preflight exchanges, and preserves the returned KMIP result. Other unknown names/vendor values have no inferred policy.
- Operation-specific Result Reasons are preserved. Transport errors keep their source and delivery state, and KMIPKit does not automatically retry.
- No raw TTLV body or attribute value is included in diagnostics. Debug output for shared Attribute values is redacted.
- This contract does not add C, JNI, CFFI, high-level builders, server operation handling, JSON/XML, or persistent local state.

## Traceability

Requirement mappings are maintained in [traceability.md](../traceability.md). The implementation PR must add executable tests for every listed guarantee and exact OASIS clause.
