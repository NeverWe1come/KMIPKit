# Rust Protocol Contract: Vendor Extension Values

## Ownership

- kmipkit-protocol owns ExtensionIdentity, ExtensionInformation, ExtensionSchema, ExtensionDefinition, and sealed ValidatedExtensionValue.
- kmipkit-client owns ClientExtensionRegistry, attachment to immutable client configuration, and ClientRequestMessageExtension request use.
- kmipkit-protocol owns the public ExtensionRegistryLimits value type; kmipkit-client enforces per-registry definition, schema-node, constraint-member, discriminator-byte, metadata, payload-index, and lookup-comparison totals when assembling or inspecting a snapshot.
- kmipkit-ffi owns all unsafe opaque-handle conversion. Other crates retain forbid(unsafe_code).
- The protocol crate depends only on TTLV and existing approved workspace dependencies. Registry code has no transport/TLS handle.

## Construction and validation

Definitions and schemas use typed, fallible constructors. Constructor errors identify a stable category and structural rule/tag path, never an actual value or payload. The protocol API may validate a generic TTLV subtree against a definition, returning a sealed `ValidatedExtensionValue` for protocol-level inspection. That type alone is not request-admissible. The client registry API resolves an exact registered identity, validates the subtree against the stored definition, and returns a sealed client-owned `RegisteredExtensionValue`; only this registry-scoped type can be wrapped for an outbound typed request.

The discriminator is an exact tag path and scalar type/value. Every path step must be unique in its containing Structure. An absent/repeated path, wrong type, or unequal value is not a match. Registry construction rejects duplicate `(vendor, discriminator path, scalar type, scalar value)` keys. Distinct keys may match the same payload when their separate paths and values are both present. Lookup collects exact discriminator matches before schema validation: zero matches or more than one match produces an unrecognized generic result; exactly one match is validated against its complete schema before a typed view is returned. Multiple matches never select by registration order. This bounds full schema validation to one candidate.

## Preservation and secrecy

- Preserve the original generic subtree and child order.
- Do not produce arbitrary TTLV bytes from this API; outbound bytes remain owned by the existing private execute encoding permit.
- Debug/Display and errors redact payload and secret values.
- Owned payload follows existing TTLV zeroization behavior. Caller and external runtime copies are outside this guarantee.
- Apply CodecLimits to every received extension subtree traversal.
- Inbound lookup builds one temporary 24-bit-tag index for the received subtree and shares it across definitions. The index preserves duplicate tags and source order; each discriminator path step matches only a unique child. Index entries and tag comparisons are charged to ExtensionRegistryLimits, and exhaustion returns a redacted resource-limit error without a partial typed projection.
- Schema construction compiles allowed Enumeration values to sorted numeric indexes and order requirements to unique acyclic tag edges. Validation uses binary enum lookup and checks ordering edges once after recording first/last child positions in one input traversal; repeated fields cannot trigger a constraint-list rescan.

## Integration

Client execution attaches an extension only through a closed typed request variant and a sealed `RegisteredExtensionValue` produced by exact-identity validation against the immutable client registry. A protocol-level `ValidatedExtensionValue` from a standalone definition is not request-admissible. Unknown critical/non-critical behavior remains in KMIPKIT-0007. No generic Item, raw body, transport implementation, or caller conversion trait is accepted as a request substitute.

For outbound use, the caller must construct ClientRequestMessageExtension with an explicit Criticality Indicator Boolean and attach it to one typed ClientBatchItem. Encoding emits the standard Message Extension fields and validated payload through the existing private OperationEncodingPermit/zeroizing writer path. No criticality default or second writer callsite is added. This outbound type name preserves KMIPKIT-0007's existing inbound ClientMessageExtension type and its preserved-content semantics.
