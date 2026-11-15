# Data Model: TTLV Wire Codec

This document defines the wire facts and per-call safety state consumed by the codec. It does not add a second public value tree; input/output values use the KMIPKIT-0004 generic model merged into `release/1.0.0`.

## Codec Limits

| Field | Default | Allowed configuration | Meaning |
|---|---:|---|---|
| `max_message_bytes` | 16,777,216 | Positive platform-representable byte count; caller may raise or lower | Maximum complete encoded item slice or private request-output length per call. Check before parser-owned allocation and before private encoder output growth. Raising this limit does not raise the per-Item U32 Item Length ceiling. |
| `max_structure_depth` | 64 | 0 through the generic model's hard maximum 64 | Maximum nested Structure nodes. A root Structure counts as depth 1. A non-Structure root has depth 0. |
| `max_elements` | 100,000 | Positive platform-representable item count; caller may raise or lower | Maximum total Items including the root and all Structure descendants. |

Limit values are per operation. `CodecLimits` is immutable after construction: its values are read through public `max_message_bytes()`, `max_structure_depth()`, and `max_elements()` getters, with no setters/builders or global mutable limit state. A limit value that cannot be represented on the target is rejected when options are constructed. Zero `max_structure_depth` permits leaf Items but no Structure Items. The private encoder receives the same borrowed `CodecLimits` instance for each operation and reads it via those getters. Getter and encoder unit tests exercise defaults and configured limits independently of the caller-facing `Client::execute` API; the first client feature/spec defines how that public API supplies the per-call instance.

## Encoded TTLV Owner

The private `EncodedTtlv` owner in `kmipkit-client` holds one successful outbound encoder result, which may contain credentials or secret key material. The maintainer authorization in `approval-record.md` accepts ADR-0012, the feature, and the request-only boundary for implementing and unit-testing this private owner and an uncalled writer in KMIPKIT-0005: acceptance of ADR-0012, approval of this feature, and approval of the enforceable boundary design. KMIPKIT-0005 does not create `Client::execute`, an `OperationEncodingPermit` type/constructor, or a production writer callsite; codec unit tests are the only callsites. The first client feature/spec defines the execute API, puts the permit's private type/constructor in its module, and makes execute the only production permit mint and writer callsite. The candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send. That test must verify the closed typed-request input and permit boundary, owner lifetime through partial writes and transport success/error return, and zeroization of the initialized encoded byte range before owner deallocation/drop. Spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and cleanup is verified. The owner must not be exposed through a public API or be cloneable, formattable, generally serializable, or convertible into an ordinary `Vec<u8>`. Copies made by callers or external TLS/runtime libraries are outside KMIPKit's zeroization guarantee.

`AGENTS.md` §8 currently prohibits serialization of credentials, keys, secret material, and raw KMIP bodies; that rule remains in force on the release branch until the first client candidate passes the required CI integration gate. Accepted ADR-0012 permits only temporary outbound TTLV for an explicitly caller-requested typed operation. KMIPKIT-0005 has no public encoder or publicly accessible encoded owner and adds no production invocation. Its private writer and owner are exercised only by codec unit tests. The first client feature/spec owns `Client::execute`, the closed typed request input, the private permit type/constructor and sole mint/callsite, plus the integration test. The first client feature PR must include the sole production callsite and its owner-through-transport integration test together. CI must pass that test against the candidate callsite before merge, enablement, or release; the release branch must have neither the callsite nor a secret-bearing send until then. The accepted policy does not permit diagnostics, public/general-purpose serialization, serialization traits, logging, formatting, error inclusion, persistence, or arbitrary inbound raw-byte retention or re-emission. Encoded bytes must not be copied unnecessarily. If the request-only boundary cannot be enforced, do not implement a production secret-bearing request path.

## TTLV Item

An Item consists of an 8-byte header followed by an Item Value and any type-specific padding. Header fields are:

| Header field | Wire width | Interpretation |
|---|---:|---|
| Tag | 3 bytes | Unsigned 24-bit value, big-endian |
| Type | 1 byte | Assigned Item Type code from KMIP 2.1 §11.23 |
| Length | 4 bytes | Unsigned 32-bit Item Value byte count, big-endian |

The item’s full wire span is not always `8 + Length`: for some types padding follows the Item Value and is excluded from Length; Big Integer padding is part of Item Value; Structure Length covers every child’s full wire span.

The KMIPKIT-0004 in-memory model retains Big Integer octets exactly and may hold non-empty octets whose length is not a multiple of eight. Encoding such a value adds the minimum sign-extension octets required for a wire-valid multiple of eight; decoding then preserves those encoded octets exactly. Therefore `decode(encode(item))` compares with a canonicalized expected model for unaligned Big Integer inputs, not necessarily the original in-memory octets.

Every Item Value length must fit the unsigned 32-bit Length field, independently of `max_message_bytes`. A caller may configure a message limit above 4 GiB on a platform that supports it, but no root or child Item Value can exceed `u32::MAX` bytes.

## Type-Length-Padding Table

| Item Type | Value form | Allowed Item Length | Padding accounting |
|---|---|---|---|
| Structure | Concatenated encoded child Items | Multiple of 8 | Length includes all child headers, values, and child padding |
| Integer | Signed 32-bit two's complement, big-endian | 4 | Four following bytes; excluded from Length |
| Long Integer | Signed 64-bit two's complement, big-endian | 8 | None |
| Big Integer | Big-endian two's-complement octets | Non-empty multiple of 8 | Minimal leading sign-extension bytes are part of Item Value and Length. Empty values are rejected by KMIPKit project policy because they do not encode an integer; OASIS does not state an explicit minimum. |
| Enumeration | Unsigned 32-bit, big-endian | 4 | Four following bytes; excluded from Length |
| Boolean | Exactly 8 bytes: all zero for false or numeric one for true | 8 | None |
| Text String | UTF-8 octets | Variable | Minimum following padding to align value extent to 8; excluded from Length |
| Byte String | Octets | Variable | Minimum following padding to align value extent to 8; excluded from Length |
| Date Time | Signed 64-bit two's complement, big-endian | 8 | None |
| Interval | Unsigned 32-bit, big-endian | 4 | Four following bytes; excluded from Length |
| Date Time Extended | Signed 64-bit two's complement, big-endian | 8 | None |

OASIS does not specify the value of Text String, Byte String, Integer, Enumeration, or Interval padding bytes in §§10.1.5. Decoder input padding is accepted by extent; the encoder uses zero octets for deterministic canonical output. Unknown Enumeration and mask bits remain raw values for later protocol validation.

## Decode Error

A decoder error records a stable class and safe position metadata (byte offset, tag, and/or Item Type when available). It never stores or formats raw input slices, decoded payloads, credentials, or secret material. An error does not return a partially decoded Item.

## Open Model Interactions

- A decoded Tag must enter the 004 allocation-checked Tag/Item model. Accepted ADR-0011 requires rejecting a received Reserved Tag before model construction; T001 records accepted decision `KMIPKIT-DEC-001` against `KMIPKIT-DISC-037` with the exact OASIS KMIP Specification v2.1 §11.56 reference and validates/regenerates the catalog report before T006 implements the decoder behavior. T012 records only applicable implementation and test traceability for normative requirements assigned to this codec.
- FR-013 and its enforceable request-only boundary are accepted under the delegated authorization in `approval-record.md`; they do not establish a production callsite. KMIPKIT-0005 tests the private writer/owner but adds no permit or `Client::execute`; the first client feature/spec owns the closed typed API, execute-owned permit/sole writer callsite, exact-one audit, and owner-through-transport integration test. The candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send. There is no public `encode(&Item)` API. KMIPKIT-0005 owns private codec tests for default/configured encoder byte/depth/count limits and U32 boundary preflight.
- The generic catalog requirement for schema-defined Structure field ordering remains unassigned pending named follow-on typed operation/model specification(s). The codec only guarantees preservation of the supplied child order.
- Unsupported Type bytes have no 004 value variant and are rejected by this codec.
- Structure schema field order is not derivable from a generic tree. The encoder preserves model order; typed protocol models must construct known Structures in their OASIS-defined order.
