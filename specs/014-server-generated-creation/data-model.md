# Data Model: Server-Generated Object Creation

## ObjectType

An extensible KMIP Object Type value. Known assigned values receive named Rust constants or variants; an unknown raw value remains representable for decoding and inspection. Outbound validation follows the reviewed assigned-value policy.

## AttributeEntry and AttributeSet

An AttributeEntry models one Table 150 Attribute Structure with `name: String` for the required Attribute Name Text String and `value: Value` for the required Attribute Value. The value TTLV type is determined by the named attribute; it is not the Attribute Name field. Names and values unknown to KMIPKit remain representable without enum narrowing. AttributeSet is an ordered sequence, not a map: repeated attributes and their order survive encoding and decoding. It distinguishes absent optional attribute structures from present-empty structures where the operation table permits both. The typed operation wrapper, not an arbitrary Item conversion trait, owns encoding.

## CreateRequest and CreateResponse

CreateRequest contains the selected ObjectType, table-defined optional Attributes and Protection Storage Masks, and no implicit algorithm or size. CreateResponse contains the required returned ObjectType and Unique Identifier on success. A non-success result has no typed success payload.

## CreateKeyPairRequest and CreateKeyPairResponse

CreateKeyPairRequest keeps Common Attributes, Private Key Attributes, Public Key Attributes, and the remaining Table 189 fields separate. Repeated AttributeEntry values preserve source order in their respective groups. Validation enforces Table 191 same-value request constraints without silently copying omitted values. KMIPKit preserves the grouped inputs; the server applies the multi-instance union semantics after Table 189. CreateKeyPairResponse contains the required private-key and public-key Unique Identifiers.

## CreateSplitKeyRequest and CreateSplitKeyResponse

CreateSplitKeyRequest contains all Table 193 inputs, including split method, part count, threshold, optional input Unique Identifier, attributes, and protection/storage masks. CreateSplitKeyResponse contains one or more ordered Unique Identifiers from repeated Table 194 fields.

Because Table 194 permits a repeated response identifier list, any ClientBatch containing CreateSplitKey is potentially large. The execute path sets the peer-visible Maximum Response Size to the smaller of its configured local response-byte limit and `i32::MAX`; the local response cap remains independently enforced.

## OperationResult

OperationResult preserves the shared KMIP Result Status, optional Reason, and optional Message. The response payload is present only when the returned status and operation table define one. Pending remains a distinct, operation-agnostic asynchronous result under KMIPKIT-0009 and retains its operation identity, exact result, and correlation value.

## Ownership and validation

- Caller input moves or borrows according to the existing request-owner contract; secret-bearing buffers use existing zeroizing owners.
- Typed operation models do not clone or own the complete ResponseMessage.
- Generic decoded TTLV remains the lossless source of truth for unmodeled fields.
- Structural validation distinguishes missing, duplicate, wrong-type, and disallowed-cardinality fields without exposing raw values in errors.
