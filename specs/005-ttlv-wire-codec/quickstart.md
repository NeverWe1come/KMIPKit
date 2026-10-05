# Quickstart: TTLV Codec Review Scenarios

The KMIPKIT-0004 generic model API is available in the updated release tree. KMIPKIT-0005 defines a public bounded decoder in `kmipkit-ttlv` and implements/tests a private byte writer without a production callsite. It does not create `Client::execute` or the `OperationEncodingPermit` type/constructor. The first client feature/spec owns the caller-facing execute/request/limit API, permit, sole production writer callsite, exact-one audit, and boundary integration test. ADR-0012 and this feature/boundary are approved under `approval-record.md`; KMIPKIT-0005 still has no production writer callsite. The release branch cannot send secret-bearing TTLV until the first client candidate passes its owner-through-transport integration test in CI. The first client feature PR must include the sole production callsite and its owner-through-transport integration test together. CI must pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must have neither the callsite nor a secret-bearing send. The first client feature owns the execute permit and closed typed request boundary. If that boundary cannot be enforced, do not create a production secret-bearing request path. Reserved-tag behavior follows accepted ADR-0011; T001 records `KMIPKIT-DEC-001` against `KMIPKIT-DISC-037` with the exact OASIS KMIP Specification v2.1 §11.56 reference and validates/regenerates the catalog report before T006. T012 later adds only applicable code/test traceability for normative requirements assigned to this codec.

## Review private encoder vectors

No public encoder call is shown or proposed. The implementation's private `kmipkit-client` encoder unit tests should cover these review scenarios after every implementation gate is satisfied:

1. Construct generic Item values using the actual merged 004 APIs; typed protocol conversion belongs to the first client feature/spec.
2. Cover every supported Item Type, exact Tag/Type/Length fields, big-endian values, Structure child order, repeated Tags, Big Integer sign extension, and each padding family with reviewed expected bytes.
3. Exercise full-tree preflight, configured limits, U32 Item Length boundaries, failure atomicity, payload-free errors, and zeroization of the initialized encoded byte range on private-owner drop. Spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and its cleanup is verified.
4. Assert that no public `encode(&Item)`, public encoder module, public encoded-byte owner, or arbitrary writer API is exposed.
5. Keep the permit, client invocation, and transport-lifetime proof out of KMIPKIT-0005. The first client feature/spec owns the execute API, permit, sole production callsite, exact-one audit, and integration test; the candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send.

## Decode a response item

The decoder is public through the proposed `kmipkit-ttlv` codec module. Once the merged API is stable, an illustrative decode-only call is:

```rust,ignore
use kmipkit_ttlv::codec::decode;

let item = decode(&reviewed_response_item_bytes)?;
// Inspect tags, value bits, and ordered Structure children through the 004 model.
```

Review these cases:

1. Decode exactly one reviewed KMIP 2.1 item using default limits.
2. Inspect its generic tag, item type, value bits, and ordered Structure children through the 004 model API.
3. Confirm unknown extension Tags, Enumeration values, mask bits, repeated child Tags, and Structure order remain available to the caller.
4. Confirm nonzero padding values are accepted by extent and do not cause original raw bytes to be retained for re-emission.

## Reject malformed input safely

Run a byte-vector case for each malformed category in `spec.md`: truncated fields, unsupported Type, invalid fixed length, invalid Boolean/UTF-8, invalid parent boundary, arithmetic overflow, excess limits, and trailing bytes. Every case must return a structured error, no partial Item, and no payload bytes in formatted diagnostics.

## Implementation readiness

Turn decoder scenarios into tested documentation after the public decoder API is stable. Keep encoder vectors and owner checks as private `kmipkit-client` unit tests; do not add an encoder example to the public `kmipkit-ttlv` API. The first client feature/spec owns the request-path integration test. The first client feature PR must include the sole production callsite and its owner-through-transport integration test together. CI must pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must have neither the callsite nor a secret-bearing send. Use the actual merged 004 constructors and do not expose an untested example as complete.
