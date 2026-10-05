# Quickstart: TTLV Codec Review Scenarios

The KMIPKIT-0004 generic model API is available in the updated release tree. KMIPKIT-0005 defines a public bounded decoder in `kmipkit-ttlv` and implements/tests a private byte writer without a production callsite. It does not create `Client::execute` or the `OperationEncodingPermit` type/constructor. The first client feature/spec owns the caller-facing execute/request/limit API, permit, sole production writer callsite, exact-one audit, and boundary integration test. ADR-0012 and this feature/boundary are approved under `approval-record.md`; KMIPKIT-0005 still has no production writer callsite. The release branch cannot send secret-bearing TTLV until the first client candidate passes its owner-through-transport integration test in CI. The first client feature PR must include the sole production callsite and its owner-through-transport integration test together. CI must pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must have neither the callsite nor a secret-bearing send. The first client feature owns the execute permit and closed typed request boundary. If that boundary cannot be enforced, do not create a production secret-bearing request path. Reserved-tag behavior follows accepted ADR-0011; T001 records `KMIPKIT-DEC-001` against `KMIPKIT-DISC-037` with the exact OASIS KMIP Specification v2.1 §11.56 reference and validates/regenerates the catalog report before T006. T012 later adds only applicable code/test traceability for normative requirements assigned to this codec.

## Review private encoder vectors

No public encoder call is shown or proposed. The implementation's private `kmipkit-client` encoder unit tests should cover these review scenarios after every implementation gate is satisfied:

1. Construct generic Item values using the actual merged 004 APIs; typed protocol conversion belongs to the first client feature/spec.
2. Cover every supported Item Type, exact Tag/Type/Length fields, big-endian values, Structure child order, repeated Tags, Big Integer sign extension, and each padding family with reviewed expected bytes.
3. Exercise full-tree preflight, configured limits, U32 Item Length boundaries, failure atomicity, payload-free errors, and zeroization of the initialized encoded byte range on private-owner drop. Spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and its cleanup is verified.
4. Assert that no public `encode(&Item)`, public encoder module, public encoded-byte owner, or arbitrary writer API is exposed.
5. Keep the permit, client invocation, and transport-lifetime proof out of KMIPKIT-0005. The first client feature/spec owns the execute API, permit, sole production callsite, exact-one audit, and integration test; the candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send.

## Decode a generic TTLV Structure

The public `kmipkit_ttlv::codec::decode_with_limits` documentation contains a
rustdoc example compiled by `cargo test -p kmipkit-ttlv --doc`. This quickstart
uses the same real wire bytes and public API. The root is a generic Structure;
its child order and repeated Tags are observable, but the decoder does not
validate an operation schema or field cardinality.

```rust
use kmipkit_ttlv::codec::{
    CodecLimits, DecodeErrorKind, decode, decode_with_limits,
};
use kmipkit_ttlv::{ItemType, ValueView};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.5, 11.23, and 11.56;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-005,
    // KMIPKIT-REQ-SPEC-10.1.5-001-002, and KMIPKIT-REQ-SPEC-11.56-001.
    // Generic order, repeated Tags, and unknown mask/Enumeration values are
    // KMIPKit project behavior under KMIPKIT-0005-FR-005. Checking padding
    // extents without constraining pad-byte values follows FR-006.
    let wire = [
        // Generic Structure Tag 0x420173; 3 child Items, 16 bytes each.
        0x42, 0x01, 0x73, 0x01, 0x00, 0x00, 0x00, 0x30,
        // Cryptographic Usage Mask Integer, first occurrence, then 4 pad bytes.
        0x42, 0x00, 0x2c, 0x02, 0x00, 0x00, 0x00, 0x04,
        0xf1, 0x23, 0x45, 0x67, 0xa1, 0xa2, 0xa3, 0xa4,
        // The same Tag again, with a different set of Integer bits.
        0x42, 0x00, 0x2c, 0x02, 0x00, 0x00, 0x00, 0x04,
        0x81, 0x00, 0x00, 0x03, 0xa5, 0xa6, 0xa7, 0xa8,
        // Extension Tag 0x541234 and an unrecognized Enumeration value.
        0x54, 0x12, 0x34, 0x05, 0x00, 0x00, 0x00, 0x04,
        0xde, 0xad, 0xbe, 0xef, 0xe1, 0xe2, 0xe3, 0xe4,
    ];

    // The convenience API uses the default limits.
    let default_item = decode(&wire)?;
    assert_eq!(default_item.item_type(), ItemType::Structure);
    assert_eq!(default_item.tag().raw(), 0x0042_0173);

    // Exact limits: 56 input bytes, one Structure level, and four Items
    // counting the root.
    let limits = CodecLimits::new(wire.len(), 1, 4)?;
    assert_eq!(limits.max_message_bytes(), wire.len());
    assert_eq!(limits.max_structure_depth(), 1);
    assert_eq!(limits.max_elements(), 4);
    let item = decode_with_limits(&wire, &limits)?;

    item.with_value(|root| match root {
        ValueView::Structure(structure) => {
            let children = structure.children();
            assert_eq!(children.len(), 3);
            assert_eq!(children[0].item_type(), ItemType::Integer);
            assert_eq!(children[1].item_type(), ItemType::Integer);
            assert_eq!(children[2].item_type(), ItemType::Enumeration);
            assert_eq!(
                children.iter().map(|child| child.tag().raw()).collect::<Vec<_>>(),
                [0x0042_002c, 0x0042_002c, 0x0054_1234],
            );
            let first_mask_bits = children[0].with_value(|value| match value {
                ValueView::Integer(bits) => Some(*bits as u32),
                _ => None,
            });
            assert_eq!(first_mask_bits, Some(0xf123_4567));
            let second_mask_bits = children[1].with_value(|value| match value {
                ValueView::Integer(bits) => Some(*bits as u32),
                _ => None,
            });
            assert_eq!(second_mask_bits, Some(0x8100_0003));
            let unknown_enum = children[2].with_value(|value| match value {
                ValueView::Enumeration(value) => Some(*value),
                _ => None,
            });
            assert_eq!(unknown_enum, Some(0xdead_beef));
        }
        _ => assert!(false, "expected a Structure root"),
    });

    // One fewer Item than required rejects the same input before it is accepted.
    let too_few_items = CodecLimits::new(wire.len(), 1, 3)?;
    let error = decode_with_limits(&wire, &too_few_items)
        .expect_err("the root and all three children count toward the limit");
    assert_eq!(error.kind(), DecodeErrorKind::ElementLimitExceeded);

    Ok(())
}
```

Each four-byte child has four following nonzero padding octets. The decoder
accepts them because their required extents are present; the returned model
contains the Integer and Enumeration values, not those padding bytes or the
original input. It does not promise byte-identical re-emission. The compiled
public API example is in [`codec::decode_with_limits`](../../crates/kmipkit-ttlv/src/codec/decoder.rs).

Review these cases:

1. Decode exactly one reviewed KMIP 2.1 item using default limits.
2. Inspect its generic tag, item type, value bits, and ordered Structure children through the 004 model API.
3. Confirm unknown extension Tags, Enumeration values, mask bits, repeated child Tags, and Structure order remain available to the caller.
4. Confirm nonzero padding values are accepted by extent and do not cause original raw bytes to be retained for re-emission.

## Reject malformed input safely

Run a byte-vector case for each malformed category in `spec.md`: truncated fields, unsupported Type, invalid fixed length, invalid Boolean/UTF-8, invalid parent boundary, arithmetic overflow, excess limits, and trailing bytes. Every case must return a structured error, no partial Item, and no payload bytes in formatted diagnostics.

## Implementation readiness

The decoder scenario is executable and mirrored by a rustdoc example in the
public `kmipkit-ttlv` API. Keep encoder vectors and owner checks as private
`kmipkit-client` unit tests; do not add a public encoder example. The first
client feature/spec owns the request-path integration test. The first client
feature PR must include the sole production callsite and its
owner-through-transport integration test together. CI must pass that test
against the candidate callsite before merge, enablement, or release; until
then, the release branch must have neither the callsite nor a secret-bearing
send.
