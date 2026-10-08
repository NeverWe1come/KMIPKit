# Data Model: Server-Generated Object Creation

## ObjectType

An extensible KMIP Object Type value. Known assigned values receive named Rust constants or variants; an unknown raw value remains representable for decoding and inspection. Outbound validation follows the reviewed assigned-value policy.

## AttributeSet

AttributeSet is an ordered sequence of generic TTLV `Item`s for the direct §4 Object Attribute items defined by §§5.1–5.4, Tables 157–160. Each item retains its tag and typed value; repeated items, unknown tags, and wire order survive encoding and decoding without synthesizing an attribute name or tag. Outbound validation continues to follow the assigned-value and registered-extension policy; this feature does not widen it. The Vendor Attribute is a distinct §4.60 structure defined by Table 150 and includes its required Vendor Identification. AttributeSet distinguishes absent optional outer structures from present-empty structures where the operation table permits both. The typed operation wrapper, not an arbitrary Item conversion trait, owns encoding.

## CreateRequest and CreateResponse

CreateRequest contains the selected ObjectType, the required outer Attributes structure from Table 186, optional Protection Storage Masks, and no implicit algorithm or size. The Attributes structure may be empty under §5.1/Table 157 but is not absent. CreateResponse contains the required returned ObjectType and Unique Identifier on success. A non-success result has no typed success payload.

## CreateKeyPairRequest and CreateKeyPairResponse

CreateKeyPairRequest keeps Common Attributes, Private Key Attributes, Public Key Attributes, and the remaining Table 189 fields separate. Each group contains direct attribute items under Tables 158–160; repeated items preserve source order. Validation enforces Table 191 same-value request constraints without silently copying omitted values. KMIPKit preserves the grouped inputs; the server applies the multi-instance union semantics after Table 189. CreateKeyPairResponse contains the required private-key and public-key Unique Identifiers.

## CreateSplitKeyRequest and CreateSplitKeyResponse

CreateSplitKeyRequest contains all Table 193 inputs, including split method, part count, threshold, optional input Unique Identifier, the required outer Attributes structure, and protection/storage masks. The Attributes structure may be empty under §5.1/Table 157. Table 193 marks request Prime Field Size optional. KMIPKit separately requires the caller to supply it for Polynomial Sharing Prime Field as an explicit client restriction; §2.8/Table 9 concerns the Split Key object, and KMIPKit never chooses or synthesizes the value. CreateSplitKeyResponse contains one or more ordered Unique Identifiers from repeated Table 194 fields.

Because Table 194 permits a repeated response identifier list, any ClientBatch containing CreateSplitKey is potentially large. The execute path sets the peer-visible Maximum Response Size to the smaller of its configured local response-byte limit and `i32::MAX`; the local response cap remains independently enforced.

## OperationResult

OperationResult preserves the shared KMIP Result Status, optional Reason, and optional Message. The response payload is present only when the returned status and operation table define one. Pending remains a distinct, operation-agnostic asynchronous result under KMIPKIT-0009 and retains its operation identity, exact result, and correlation value.

## Ownership and validation

- Caller input moves or borrows according to the existing request-owner contract; secret-bearing buffers use existing zeroizing owners.
- Typed operation models do not clone or own the complete ResponseMessage.
- Generic decoded TTLV remains the lossless source of truth for unmodeled fields.
- Structural validation distinguishes missing, duplicate, wrong-type, and disallowed-cardinality fields without exposing raw values in errors.
