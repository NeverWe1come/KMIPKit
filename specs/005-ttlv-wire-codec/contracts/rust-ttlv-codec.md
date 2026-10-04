# Rust TTLV Codec Contract

**Status**: Proposed public Rust contract for review. Final names and signatures must be reconciled with the merged KMIPKIT-0004 crate API before implementation.

## Public operations

```rust
pub struct CodecLimits { /* private fields with checked constructor/builders */ }

impl CodecLimits {
    pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
    pub const DEFAULT_MAX_STRUCTURE_DEPTH: usize = 64;
    pub const DEFAULT_MAX_ELEMENTS: usize = 100_000;

    pub fn new(max_message_bytes: usize, max_structure_depth: usize, max_elements: usize)
        -> Result<Self, LimitsError>;
    pub fn defaults() -> Self;
}

pub fn encode(item: &Item) -> Result<Vec<u8>, EncodeError>;
pub fn encode_with_limits(item: &Item, limits: &CodecLimits)
    -> Result<Vec<u8>, EncodeError>;
pub fn decode(bytes: &[u8]) -> Result<Item, DecodeError>;
pub fn decode_with_limits(bytes: &[u8], limits: &CodecLimits)
    -> Result<Item, DecodeError>;
```

The signatures are a design proposal layered on the merged 004 API. The model currently exposes `RawTag::new`, `RawTag::try_checked`, `Item::new`, `Structure::new`, `Structure::try_push`, `Value` constructors for all eleven types, and closure-scoped `Item::with_value`. Codec option names remain proposed; limit objects are immutable and scoped to one call. The eventual public module path is `kmipkit_ttlv::codec` unless implementation review identifies a better existing convention.

## Behavioral contract

- `encode` emits exactly one canonical Item. It validates the complete tree, checks depth/count and predicted output size, then reserves the complete output capacity before copying payload bytes. It preserves Structure child order and has no fallible exit after payload copying begins; if that invariant cannot be maintained, partial output is zeroized on every error path.
- `decode` accepts exactly one complete Item and rejects empty input or trailing bytes. It validates lengths and available bytes before payload allocation.
- Both default entry points use 16 MiB, 64 Structure levels, and 100,000 Items.
- `decode_with_limits` may use lower or higher message/count limits. Maximum Structure depth remains 64 unless the 004 model contract is deliberately changed.
- Each Item Value length must fit `u32::MAX`, even when the configured message limit is larger. The encoder checks this before writing a header; the decoder obtains the value from the U32 header and checks cumulative arithmetic and bounds before allocation.
- The decoder rejects unsupported Item Type bytes, type-specific invalid lengths, invalid UTF-8, noncanonical Boolean encodings, truncated values/padding, arithmetic overflow, parent-boundary violations, and configured limit excess.
- The codec rejects an empty Big Integer as KMIPKit project policy. OASIS §10.1.2 requires a two's-complement byte sequence with a length multiple of eight but does not explicitly set a minimum length.
- Padding octets whose values are not constrained by OASIS are accepted at the required extent; the encoder writes zero for those padding octets. Big Integer leading sign-extension bytes are part of the represented Item Value and are not discarded.
- A Reserved Tag received from the wire is rejected before construction under proposed ADR-0011; implementation remains gated on review/acceptance of that ADR.
- No operation performs I/O, retries requests, validates a KMIP operation schema, or logs payloads.

## Errors

`EncodeError` and `DecodeError` expose stable error kinds and safe structural context only. They do not contain input/output bytes, Text String values, Byte String contents, Big Integer octets, or child payloads. Public error source chains preserve non-payload causes such as allocation failure when available. They implement redacted `Debug` and `Display`.

## Safety and allocation invariants

- All offset/length arithmetic uses checked operations.
- Input size is checked before decoder traversal.
- Declared Item Length is checked against allowed type lengths, parent end, total input, and caller limits before allocating/copying value bytes.
- Structure parsing stops at its declared parent boundary and increments element/depth counters before accepting each child.
- Allocation uses one complete fallible output reservation before any payload copy; allocation failure returns an error, never panic. Decoded payload allocations are also fallible and occur only after length/limit preflight.
- The codec crate continues to forbid unsafe code.
