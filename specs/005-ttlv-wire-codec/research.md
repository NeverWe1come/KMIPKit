# Research: KMIP TTLV Wire Codec

## Source validation

The normative source is the repository's immutable copy at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, identified by `specification/oasis/kmip-2.1/SOURCES.md`. The relevant sections are §§10.1.1–10.1.5, §11.23, Chapter 11 introduction, and §11.56. The checked-in catalog contains five requirement IDs directly associated with §§10.1.2 and 10.1.5. It does not define clause/requirement IDs for §§10.1.1, 10.1.3, or 10.1.4; this specification uses stable `KMIPKIT-0005-NR-*` IDs for those rules and records the gap for catalog reconciliation.

An independent source audit verified the pinned copy against its SHA-256 digest `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF` and identified the exact source section and catalog clause locations. No upstream source file was changed.

## Protocol findings

- §10.1.1: Tag is three bytes, unsigned, big-endian.
- §10.1.2 and §11.23: Type is one byte. The eleven assigned types have distinct representations and fixed or variable widths. Specification-defined Structure fields have a required schema order.
- §10.1.3: Item Length is a 32-bit big-endian count of Item Value bytes, subject to type-specific allowed lengths.
- §10.1.4: Item Value is interpreted according to Item Type.
- §10.1.5: Structure Item Length includes child encodings and their padding. Integer, Enumeration, Text String, Byte String, and Interval Item Length exclude their following padding. Text/Byte padding is the minimum trailing amount to an eight-byte boundary; Integer/Enumeration/Interval have four following bytes.
- §10.1.2 Big Integer rules: minimal leading sign-extension bytes make the value length a multiple of eight; these bytes are part of Item Value and Item Length.
- §10.1.2 does not explicitly give Big Integer a minimum wire length. KMIPKit adopts the project validity rule that an empty value is rejected because an empty octet sequence represents no two's-complement integer. This policy is distinct from an OASIS MUST.
- §10.1.3 encodes each Item Length in an unsigned 32-bit field. Raising the configured total-message limit cannot make any individual Item Value larger than `u32::MAX`; this representability check is independent of platform-sized allocation limits.
- §10.1.5 gives no required value for the following padding octets. The decoder therefore checks padding extent, not content. The encoder emits zero for deterministic canonical output; this is project policy.
- Chapter 11 introduction prohibits use of Tags marked Reserved. §11.56 supplies the standard and extension prefix ranges. `KMIPKIT-DISC-037` leaves receipt/preservation behavior open; proposed ADR-0011 recommends rejection before generic model construction.

## Design decisions and alternatives

### Codec input/output boundary

**Selected**: Encode or decode exactly one generic Item from/to a byte vector/slice. The codec does no stream reads, socket I/O, TLS, HTTP, message batching, or operation-level schema validation.

**Alternative**: Expose a prefix decoder returning the unconsumed suffix or a streaming reader. Rejected for this feature because transport framing and streaming error states belong to the transport layer; one complete root item has a length in its own header.

### Model coupling

**Selected**: Operate on the approved `kmipkit-ttlv` Item/Structure/Value types from KMIPKIT-0004. Keep all eleven types in one generic model and preserve caller-provided child order.

**Alternative**: Add a second raw wire AST. Rejected for the normal decode path because it duplicates the generic tree and creates conversion/lifetime surfaces. This alternative is needed only if the project later revises proposed ADR-0011 to preserve received Reserved Tags under `KMIPKIT-DISC-037`.

### Allocation strategy

**Selected**: Validate the complete input byte bound before parsing, validate every header length and checked addition before allocating payload storage, then copy values into the generic model with fallible reserve operations. Enforce per-call item/depth limits during traversal.

**Alternative**: Trust declared lengths and allocate a complete child/value buffer up front. Rejected because network-supplied lengths are untrusted and may request excessive memory.

### Limit semantics

Defaults are 16 MiB per message, Structure depth 64, and 100,000 total Items. Count the root Item toward the item limit. Count a root Structure as depth 1; a non-Structure root has Structure depth 0. Per-call message and item limits can be raised or lowered; depth can be configured from 0 through the model's hard ceiling of 64. A depth above 64 would require changing the KMIPKIT-0004 model contract and is a gate, not an implicit extension.

The U32 Item Length ceiling is unconditional even if a caller raises `max_message_bytes`. A Structure's own Item Value (including complete encoded child spans) must also fit the U32 ceiling. Tests must exercise a synthetic U32-max header and checked cumulative arithmetic without allocating a multi-gigabyte buffer.

### Reserved Tags

Proposed ADR-0011 selects rejection of received Reserved Tags before generic model construction. This is the recommended project policy, not an OASIS clarification, and remains an implementation blocker until reviewed/accepted. Opaque preservation would require a separate wire representation incompatible with the current checked-Tag tree.

### Unknown Item Type codes and Enumeration values

Reject Item Type bytes that are not among the eleven types in §11.23 because the current generic model cannot represent them. Preserve all unsigned Enumeration payload bits and Integer mask bits without assigning operation-level meaning; the generic codec does not perform the later catalog/profile validation needed to establish semantic enumeration validity.

### Canonical padding

Accept any pad-byte values when the required padding extent is present, since OASIS does not constrain those values in §§10.1.5. Emit zero padding for Integer, Enumeration, Interval, Text String, and Byte String. A decode/re-encode cycle can normalize nonzero padding and therefore promises canonical semantic re-encoding, not byte identity for arbitrary noncanonical padding.

## Catalog traceability gap

When implementation starts, update only the applicable requirement records in `specification/catalog/kmip-2.1.json` to include this feature and code/test references. Do not manually edit generated outputs. Catalog requirement `KMIPKIT-REQ-SPEC-10.1.2-001` for schema-specific field order is not implemented by this generic codec: catalog coverage remains incomplete until the relevant typed operation/model specification(s) are named in the catalog and provide executable order checks. The inventory workflow must assign those rows before a 100% traceability claim.
