# Data Model: TTLV Wire Codec

This document defines the wire facts and per-call safety state consumed by the codec. It does not add a second public value tree; input/output values use the KMIPKIT-0004 generic model after that implementation lands.

## Codec Limits

| Field | Default | Allowed configuration | Meaning |
|---|---:|---|---|
| `max_message_bytes` | 16,777,216 | Positive platform-representable byte count; caller may raise or lower | Maximum complete encoded item slice/output length for one codec call. Check before parser-owned allocation and before encoder output growth. |
| `max_structure_depth` | 64 | 0 through the generic model's hard maximum 64 | Maximum nested Structure nodes. A root Structure counts as depth 1. A non-Structure root has depth 0. The open question about raising above 64 is a feature gate. |
| `max_elements` | 100,000 | Positive platform-representable item count; caller may raise or lower | Maximum total Items including the root and all Structure descendants. |

Limit values are per operation. The codec has no global mutable limit state. A limit value that cannot be represented on the target is rejected when options are constructed. Zero `max_structure_depth` permits leaf Items but no Structure Items.

## TTLV Item

An Item consists of an 8-byte header followed by an Item Value and any type-specific padding. Header fields are:

| Header field | Wire width | Interpretation |
|---|---:|---|
| Tag | 3 bytes | Unsigned 24-bit value, big-endian |
| Type | 1 byte | Assigned Item Type code from KMIP 2.1 §11.23 |
| Length | 4 bytes | Unsigned 32-bit Item Value byte count, big-endian |

The item’s full wire span is not always `8 + Length`: for some types padding follows the Item Value and is excluded from Length; Big Integer padding is part of Item Value; Structure Length covers every child’s full wire span.

## Type-Length-Padding Table

| Item Type | Value form | Allowed Item Length | Padding accounting |
|---|---|---|---|
| Structure | Concatenated encoded child Items | Multiple of 8 | Length includes all child headers, values, and child padding |
| Integer | Signed 32-bit two's complement, big-endian | 4 | Four following bytes; excluded from Length |
| Long Integer | Signed 64-bit two's complement, big-endian | 8 | None |
| Big Integer | Big-endian two's-complement octets | Multiple of 8 | Minimal leading sign-extension bytes are part of Item Value and Length |
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

- A decoded Tag must enter the 004 allocation-checked Tag/Item model. `KMIPKIT-DISC-037` remains open for Reserved-tag receipt; neither rejection nor opaque preservation is assumed by this data model.
- Unsupported Type bytes have no 004 value variant and are rejected by this codec draft.
- Structure schema field order is not derivable from a generic tree. The encoder preserves model order; typed protocol models must construct known Structures in their OASIS-defined order.
