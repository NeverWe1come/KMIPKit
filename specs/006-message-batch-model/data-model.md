# Data Model: KMIP 2.1 Message and Batch Model

## Ownership and conversion boundary

`RequestMessage` and `ResponseMessage` are validated owners of one generic `kmipkit_ttlv::Structure`. Parsing consumes a `Structure`, validates required fields, item cardinality, known-field schema order, field types, and cross-field constraints, then keeps the original tree. `into_ttlv(self)` returns that same tree by ownership. This avoids cloning opaque or sensitive payloads and preserves unknown allocated tags, repeated fields, and source order.

The TTLV crate exposes a borrowed `Structure::view()` whose lifetime is tied to `&Structure`. `StructureView` exposes child order and `Item::with_value` keeps each value view within a closure. Typed header and batch views read scalar values while opaque credential, attestation, operation, and extension subtrees remain callback-scoped generic data. Default `Debug`, `Display`, and validation errors show only model/error categories and safe counts.

Known fields MUST appear in the exact order shown in the tables below. Unknown but allocation-valid fields are retained at their original position; they do not excuse reordering known fields or make an otherwise invalid message valid. The generic TTLV allocation policy remains authoritative for whether a tag can be represented.

## Entities

| Entity | Data | Invariants |
|---|---|---|
| `ProtocolVersion` | Raw signed 32-bit major and minor values. | Preserves all accepted Integer values. The 1.0 client builder defaults to 2.1. Compatibility policy is outside this model. |
| `RequestMessage` | Request Header followed by one or more Request Batch Items, owned as an ordered generic `Structure`. | Exactly one header; at least one batch item; Batch Count equals item count; known field order follows Table 395/396. |
| `ResponseMessage` | Response Header followed by one or more Response Batch Items, owned as an ordered generic `Structure`. | Exactly one header; at least one batch item; Batch Count equals item count; known field order follows Table 398/399. |
| `RequestHeaderView` | Typed read-only access to fields in a Request Header. | Preserves omitted fields separately from effective defaults. Authentication and attestation values remain opaque. |
| `ResponseHeaderView` | Typed read-only access to fields in a Response Header. | Repeated Attestation Type values retain source order; opaque nonce/password/attestation content is callback scoped. |
| `RequestBatchItemView` | Operation, optional Ephemeral, conditional Unique Batch Item ID, required Request Payload, and repeated Message Extensions. | IDs are required when batch count is greater than one; pairwise distinctness is a KMIPKit project invariant, not an OASIS requirement. Payload and extensions stay generic. |
| `ResponseBatchItemView` | Optional-in-isolation Operation/ID, required Result Status, optional reason/message/async correlation, conditional Response Payload, and at most one optional Message Extension. | Pending requires Async Correlation Value. Operation is required when present in the paired request. `KMIPKIT-0007-client-execution` validates that condition and response ID echo. |
| `MessageExtensionView` | Vendor Identification, Criticality Indicator, and opaque Vendor Extension. | Validate the three required singleton fields, their types, Table 418 order, and the Vendor Identification character set. The immutable per-client registry in `docs/architecture/extensions.md` decides recognition; `KMIPKIT-0007-client-execution` applies criticality behavior required by ADR-0007. |

## Request field order

### Request Header (Specification Table 395)

1. Protocol Version — Structure, required.
2. Maximum Response Size — Integer, optional.
3. Client Correlation Value — Text String, optional.
4. Server Correlation Value — Text String. For client-to-server exchanges, this is response metadata; the generic model preserves a raw field if it appears in an out-of-scope request, but typed outgoing 1.0 client requests have no setter for it. Server-to-client request use is deferred to 1.1.
5. Asynchronous Indicator — raw-preserving Enumeration, optional; effective default is Prohibited. Parsing retains any present raw value.
6. Attestation Capable Indicator — Boolean, optional; effective default is False. `KMIPKIT-0008-credentials-attestation` owns truthful setting based on supported credential creation.
7. Attestation Type — Enumeration, repeatable optional.
8. Authentication — Structure, optional; opaque in this feature.
9. Batch Error Continuation Option — Enumeration, only when Batch Count > 1; effective default is Stop. Preserve raw enum values and do not infer disputed execution effects.
10. Batch Order Option — Boolean, only when Batch Count > 1; effective default is True.
11. Time Stamp — Date Time, optional.
12. Batch Count — Integer, required and derived from item count.

### Request Batch Item (Specification Table 396)

1. Batch Item — Structure, required envelope.
2. Operation — Enumeration, required; preserve unknown raw values.
3. Ephemeral — Boolean, optional.
4. Unique Batch Item ID — Byte String; required when Batch Count > 1 and optional for a single-item message.
5. Request Payload — Structure, required and opaque to this feature.
6. Message Extension — Structure, repeatable optional.

## Response field order

### Response Header (Specification Table 398)

1. Protocol Version — Structure, required.
2. Time Stamp — Date Time, required.
3. Nonce — Structure, conditional/optional as specified by its attestation context; opaque here.
4. Server Hashed Password — Byte String, conditional/optional as specified by its authentication context; opaque here.
5. Attestation Type — Enumeration, repeatable and conditional/optional as specified by its attestation context.
6. Client Correlation Value — Text String, optional.
7. Server Correlation Value — Text String, optional.
8. Batch Count — Integer, required and derived from item count.

The exact conditional-presence checks for authentication are deferred to the credential and profile specifications; attestation checks belong to `KMIPKIT-0008-credentials-attestation`. This feature preserves and exposes those fields but does not claim the corresponding credential/profile semantics.

### Response Batch Item (Specification Table 399)

1. Batch Item — Structure, required envelope.
2. Operation — Enumeration, optional; if present, preserve its raw value.
3. Unique Batch Item ID — Byte String, optional in isolation; if the request item carried one, the server is required to echo the same bytes.
4. Result Status — Enumeration, required and represented through KMIPKIT-0003 result types.
5. Result Reason — Enumeration, conditional on Result Status.
6. Result Message — Text String; absent for Success and Pending, optional for other statuses; redacted by default formatting.
7. Asynchronous Correlation Value — Byte String, required for Pending and otherwise optional as defined by the response schema.
8. Response Payload — Structure, absent for Failure and present for the other result classes as defined by OASIS; remains opaque here.
9. Message Extension — Structure, optional singleton. Table 399 does not authorize repeated response Message Extensions.

## Validation boundaries

- Structural validation rejects absent required fields, duplicate singleton fields (including duplicate response Message Extensions), invalid known-field order, wrong item types, zero batch items, count mismatches or signed-32-bit overflow, invalid option/cardinality combinations, missing multi-item request IDs, duplicate IDs within a request as a KMIPKit project invariant, and Pending without an asynchronous correlation value. Unknown or unassigned Enumeration values do not make a TTLV-to-model parse fail.
- The result validator reuses `KmipOperationResult` for Result Status/Reason consistency and additionally rejects Result Message on Success or Pending. Result Message remains available for explicit inspection but is not emitted in default diagnostics.
- A response is not compared with a particular request by this model. `KMIPKIT-0007-client-execution` checks conditional Operation presence, echoed IDs, result-item association, sends requests using only Protocol Version 2.1 and accepts only 2.1 responses under ADR-0002. Verification includes a positive 2.1 case, a property test that accepts a version pair iff it is `(2, 1)`, and deterministic non-2.1 mismatch and signed-32-bit boundary cases. It also checks whether Pending was allowed by the request's Asynchronous Indicator, extension criticality, and declared response-size enforcement.
- `RequestMessage::try_from_ttlv` and request/response conversions preserve all raw Enumeration values, including unknown standardized option values, assigned values, OASIS extension-range values, and other unassigned values. They do not apply outbound send rules. Before emitting client requests, `KMIPKIT-0007-client-execution` accepts assigned Asynchronous Indicator and Batch Error Continuation values plus the OASIS extension range `0x80000000..=0x8FFFFFFF` and rejects other unassigned values (Tables 432, 435; §§9.2, 9.6). Generic TTLV and typed raw-preserving Enumeration wrappers remain open to unknown and vendor/future values. Explicitly repeatable fields and repeated unknown fields retain their order; duplicate known singleton fields are rejected. The protocol model preserves Message Extensions without classifying them; an unregistered extension is unknown to the per-client registry, so unknown critical extensions reject the response and non-critical extensions may be processed as opaque content.
- No byte encoding, TTLV wire framing, transport resource limit, network call, retry, Poll, Cancel, or wait operation is performed here.
