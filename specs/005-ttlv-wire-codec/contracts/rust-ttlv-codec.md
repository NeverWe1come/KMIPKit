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

The signatures are a design proposal, not a promise to code against an unmerged API. The exact constructors may use the project’s established options pattern after PR #14 lands. Limit objects are immutable and scoped to one call.

## Behavioral contract

- `encode` emits exactly one canonical Item. It checks tree depth/count and predicted output size before growth. It preserves Structure child order.
- `decode` accepts exactly one complete Item and rejects empty input or trailing bytes. It validates lengths and available bytes before payload allocation.
- Both default entry points use 16 MiB, 64 Structure levels, and 100,000 Items.
- `decode_with_limits` may use lower or higher message/count limits. Maximum Structure depth remains 64 unless the 004 model contract is deliberately changed.
- The decoder rejects unsupported Item Type bytes, type-specific invalid lengths, invalid UTF-8, noncanonical Boolean encodings, truncated values/padding, arithmetic overflow, parent-boundary violations, and configured limit excess.
- Padding octets whose values are not constrained by OASIS are accepted at the required extent; the encoder writes zero for those padding octets. Big Integer leading sign-extension bytes are part of the represented Item Value and are not discarded.
- A Reserved Tag received from the wire remains blocked pending the reviewed resolution of `KMIPKIT-DISC-037`.
- No operation performs I/O, retries requests, validates a KMIP operation schema, or logs payloads.

## Errors

`EncodeError` and `DecodeError` expose stable error kinds and safe structural context only. They do not contain input/output bytes, Text String values, Byte String contents, Big Integer octets, or child payloads. Public error source chains preserve non-payload causes such as allocation failure when available. They implement redacted `Debug` and `Display`.

## Safety and allocation invariants

- All offset/length arithmetic uses checked operations.
- Input size is checked before decoder traversal.
- Declared Item Length is checked against allowed type lengths, parent end, total input, and caller limits before allocating/copying value bytes.
- Structure parsing stops at its declared parent boundary and increments element/depth counters before accepting each child.
- Allocation uses fallible reserve APIs where available; allocation failure returns an error, never panic.
- The codec crate continues to forbid unsafe code.
