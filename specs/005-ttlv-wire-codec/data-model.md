# Data Model: TTLV Wire Codec

This document defines the wire facts and per-call safety state consumed by the codec. It does not add a second public value tree; input/output values use the KMIPKIT-0004 generic model merged into `release/1.0.0`.

## Codec Limits

| Field | Default | Allowed configuration | Meaning |
|---|---:|---|---|
| `max_message_bytes` | 16,777,216 | Positive platform-representable byte count; caller may raise or lower | Maximum complete encoded item slice/output length for one codec call. Check before parser-owned allocation and before encoder output growth. Raising this limit does not raise the per-Item U32 Item Length ceiling. |
| `max_structure_depth` | 64 | 0 through the generic model's hard maximum 64 | Maximum nested Structure nodes. A root Structure counts as depth 1. A non-Structure root has depth 0. |
| `max_elements` | 100,000 | Positive platform-representable item count; caller may raise or lower | Maximum total Items including the root and all Structure descendants. |

Limit values are per operation. The codec has no global mutable limit state. A limit value that cannot be represented on the target is rejected when options are constructed. Zero `max_structure_depth` permits leaf Items but no Structure Items.

## Encoded TTLV Owner

`EncodedTtlv` owns one successful encoder result, which may contain credentials or secret key material. It exposes the bytes through an immutable borrow for protocol transport and zeroizes its initialized bytes and backing capacity when dropped. It is not cloneable, formattable, or generally serializable and cannot be converted into an ordinary `Vec<u8>` through its public API. Copies made by callers or external TLS/runtime libraries are outside KMIPKit's zeroization guarantee.

The repository rule against serializing secrets is applied to diagnostics, general-purpose serialization, and persistence. TTLV wire encoding remains permitted only as the representation needed to carry a caller-requested KMIP exchange; the owner must live through the protocol write and be dropped afterward. The encoded bytes must not be logged, formatted, persisted, or copied unnecessarily.

## TTLV Item

An Item consists of an 8-byte header followed by an Item Value and any type-specific padding. Header fields are:

| Header field | Wire width | Interpretation |
|---|---:|---|
| Tag | 3 bytes | Unsigned 24-bit value, big-endian |
| Type | 1 byte | Assigned Item Type code from KMIP 2.1 §11.23 |
| Length | 4 bytes | Unsigned 32-bit Item Value byte count, big-endian |

The item’s full wire span is not always `8 + Length`: for some types padding follows the Item Value and is excluded from Length; Big Integer padding is part of Item Value; Structure Length covers every child’s full wire span.

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

- A decoded Tag must enter the 004 allocation-checked Tag/Item model. Proposed ADR-0011 recommends rejecting a received Reserved Tag before model construction; that policy remains gated on review.
- The generic catalog requirement for schema-defined Structure field ordering remains unassigned pending named follow-on typed operation/model specification(s). The codec only guarantees preservation of the supplied child order.
- Unsupported Type bytes have no 004 value variant and are rejected by this codec draft.
- Structure schema field order is not derivable from a generic tree. The encoder preserves model order; typed protocol models must construct known Structures in their OASIS-defined order.
