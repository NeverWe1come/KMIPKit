# Quickstart: TTLV Codec Review Scenarios

The KMIPKIT-0004 generic model API is available in the updated release tree. The codec calls below are proposed by `contracts/rust-ttlv-codec.md`; these review scenarios are not an approved public API or authorization to encode secret-bearing values. The current `AGENTS.md` §8 prohibition remains in force until a human accepts ADR-0012 and approves this feature specification. The final API must restrict any approved secret-bearing output to an explicitly caller-requested KMIP operation and must not expose general-purpose serialization. Reserved-tag behavior follows proposed ADR-0011 and remains implementation-gated until review.

## Review an encoder vector

The merged 004 constructors are stable. The codec function below is the proposed 0005 contract and becomes executable when implemented. This non-secret vector demonstrates wire bytes only; it does not resolve or authorize a standalone secret-bearing serialization API.

```rust
fn encode_example() -> Result<(), Box<dyn std::error::Error>> {
    use kmipkit_ttlv::codec::encode; // proposed public path
    use kmipkit_ttlv::{Item, RawTag, Value};

    let tag = RawTag::new(0x0042_0173)?.try_checked()?;
    let item = Item::new(tag, Value::integer(42))?;
    let encoded = encode(&item)?;
    assert_eq!(encoded.as_bytes(), [
        0x42, 0x01, 0x73, 0x02, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x00,
    ]);
    Ok(())
}
```

1. Construct a generic Structure in the exact field order required by its KMIP 2.1 structure definition.
2. Add at least one leaf value from each supported Item Type in separate reviewed vectors.
3. Encode one root Item with default limits.
4. Compare Tag, Type, Length, Item Value, and padding against the exact expected bytes.
5. Confirm a deliberately oversized model fails before output grows beyond the configured byte limit.

## Decode a response item

1. Load a reviewed KMIP 2.1 TTLV vector as bytes.
2. Decode exactly one item using default limits.
3. Inspect the generic tag, item type, value bits, and ordered Structure children through the 004 model API.
4. Re-encode and compare with the vector's canonical form. For nonzero padding values, expect canonical zero padding on output while retaining the same represented Item Value.
5. Confirm unknown extension Tags, Enumeration values, mask bits, repeated child Tags, and Structure order remain available to the caller.

## Reject malformed input safely

Run a byte-vector case for each malformed category in `spec.md`: truncated fields, unsupported Type, invalid fixed length, invalid Boolean/UTF-8, invalid parent boundary, arithmetic overflow, excess limits, and trailing bytes. Every case must return a structured error, no partial Item, and no payload bytes in formatted diagnostics.

## Implementation readiness

Turn the scenarios above into executable integration tests and an example under the final public contract during implementation. Use the actual merged 004 constructors and the codec contract; do not expose an untested example as complete.
