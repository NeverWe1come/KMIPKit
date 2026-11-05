# Common Private Wire Codec Contract

## Boundary

The common semantic value is the existing ordered
`kmipkit_ttlv::Item` (`Tag` plus typed `Value`). `TtlvItem` is not a second
type. Codecs do not construct operation-specific requests, validate KMIP
message schemas, perform transport I/O, or expose public serialization.
`kmipkit-client` owns the private wire conversion. The protocol model remains
the source of typed request/response semantics.

## Interface shared by JSON and XML

Each codec implements the same private per-format operations:

```rust,ignore
fn encode(
    item: &kmipkit_ttlv::Item,
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<zeroize::Zeroizing<Vec<u8>>, WireCodecError>;

fn decode(
    bytes: &[u8],
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<kmipkit_ttlv::Item, WireCodecError>;
```

The signatures are private crate implementation contracts, not public Rust
API. The JSON and XML modules live in separate source files and own separate
codec-specific tests. They may each register their module in the crate root;
merge that small module declaration independently. Neither branch edits
`public-api.json`, generated API output, or client encoding dispatch. The
client selector and common dispatch are introduced only by KMIPKIT-0057.

## Names and unknown values

- Look up known tag source names, assigned Enumeration names/values, and
  tag-to-Enumeration-family mappings from the private tables generated offline
  by `tools/normative_catalog/generate_wire_names.py` from the reviewed
  catalog; builds never scrape or download OASIS sources. Mapping inputs are
  catalog `structure_member`, `message_field`, and `operation_structure` rows
  whose source encoding is Enumeration; each maps by its assigned tag name
  and the longest matching Enumeration family suffix.
- The common model retains an Enumeration as its raw 32-bit value. Resolve its
  family through the generated tag-to-family table; when an unambiguous family
  mapping is absent or the value is unknown, preserve it in an OASIS-permitted
  numeric or hexadecimal representation. Decoders preserve the raw value.
- Preserve tag identity and raw Enumeration/Bit Mask values in the `Item`
  model. JSON uses an assigned Enumeration name when resolved, otherwise a
  profile-permitted unsigned number or 32-bit hexadecimal string. XML uses the
  assigned CamelCase Enumeration name when resolved, otherwise a hexadecimal
  string. Unknown tags use profile-permitted hexadecimal forms.
- Preserve Structure child order, repeated children, exact represented
  payloads, and allocation-valid unknown tags accepted by the model.
- Each codec specification defines its exact symbolic-name normalization and
  its XML/JSON-specific handling of unknown names and values against the
  correct Profiles section.

## Limits and errors

- Enforce the same immutable `CodecLimits` values used for the exchange:
  maximum message bytes, Structure depth, and total Item count including root.
- Check input byte length before parsing and check depth/count before
  size-driven allocation or descent. Encoders must enforce message/depth/count
  limits as they traverse and before growing the output buffer beyond its cap.
- Return `WireCodecError` categories only: `InvalidDocument`, `InvalidValue`,
  `LimitExceeded`, and `AllocationFailed`. Do not retain parser/source errors,
  input slices, textual values, or raw message bytes. Display and Debug expose
  only the category and safe numeric offset when the parser can provide one.
- XML parsing disables DTD processing, external entities, external schema
  retrieval, and network access. JSON parsing rejects duplicate object keys
  and trailing non-whitespace data unless the 0055 specification documents an
  OASIS-permitted behavior requiring another rule.

## Ownership and call lifecycle

- The encoder returns a KMIPKit-owned zeroizing buffer. Its initialized bytes
  remain owned through the single transport exchange and are zeroized before
  release under ADR-0012's exact allocation-growth limitations.
- The decoder borrows the transport response bytes and must not retain that
  slice or make an unbounded/unzeroized copy. Decoded byte/text payloads use
  the existing model's zeroizing owners.
- The first client integration passes one borrowed `CodecLimits` instance to
  encode, transport cap enforcement, and decode; it does not reconstruct or
  silently widen limits.
- No codec performs retries, format detection for fallback, or network I/O.
