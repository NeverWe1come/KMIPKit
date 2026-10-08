# Data Model: KMIP 2.1 Vendor Extension Registry

## ExtensionIdentity

Local stable identity for a vendor extension.

| Field | Type | Invariant |
|---|---|---|
| vendor_identifier | Text String | Non-empty; uses only KMIP §9.13, Table 418 characters A-Z, a-z, 0-9, underscore, and period. |
| name | Text String | Non-empty, stable within this vendor, exact comparison; never inferred from Vendor Identification. |
| version | Text String | Non-empty, stable extension version; exact comparison. |

KMIP Message Extension does not carry name/version fields. They are local metadata and are not inserted into the standard outer structure unless part of the vendor's own schema.
The UTF-8 byte lengths of `vendor_identifier`, `name`, and `version` count against the per-field and aggregate registry identity/Extension Information text limits.

## Compatibility

| Field | Type | Invariant |
|---|---|---|
| KMIP range | Inclusive pair of protocol versions | Must include KMIP 2.1 to be usable by this 1.0 client. |
| KMIPKit range | Inclusive semantic-version range | Installed library version must be within the declared range. |
| Direction | Client capability set | Only client-initiated extension construction/recognition is in scope. |

The registry does not infer server support from local compatibility. Query reports server support separately.

## TtlvPath and Discriminator

A TtlvPath is an ordered, non-empty sequence of child Tags relative to the root Vendor Extension structure. Each step must resolve through exactly one Structure child. The terminal item must be a scalar with the exact declared Item Type and value.

A Discriminator consists of one TtlvPath, scalar Item Type, and exact scalar value. A registry key is (vendor_identifier, path, item_type, value). Duplicate keys for a vendor fail registry construction. Missing, repeated, or wrong-type path items do not match. No predicates or normalization are permitted.

At registry construction, discriminator scalar bytes are counted before cloning. A Text String is counted by its UTF-8 byte length and a Byte String by its exact byte length. Every allowed-enumeration value and every member of an ordering constraint is counted before storage is reserved or copied, including members reached through reused builder values.

## ExtensionSchema

A recursive, immutable, data-only description of allowed TTLV values:

- Required Item Type for every node.
- Structure members keyed by Tag, each with a child schema and required/optional/repeated cardinality.
- Optional order constraints as directed `before_tag -> after_tag` edges when the vendor definition makes child order significant.
- Text String and Byte String minimum/maximum lengths.
- Signed/unsigned numeric minimum/maximum.
- Allowed Enumeration values.
- Allowed and required Bit Mask bits.
- Policy for undeclared children: reject or preserve without a typed field.

The schema vocabulary contains no script, callback, regex, arbitrary expression, or cross-field code hook. Every rule is deterministic and evaluated over the already bounded TTLV model. The schema root is a Structure representing the Vendor Extension value.

Schema child rules are compiled into a tag-ordered index at construction, with no more than 256 rules in one Structure by default or 4,096 at the hard maximum. This keeps child lookup bounded rather than rescanning a wide schema for each input child. Allowed-enumeration values are compiled into a sorted numeric index; validation uses binary search and performs at most 13 value comparisons for a set of up to 4,096 values.

Ordering constraints are directed tag pairs. Construction rejects duplicate edges, self-edges, and cycles, then resolves each endpoint to its child-rule index. During validation, one pass over input children records the first and last position of each declared child. A second pass checks each edge once: when both endpoints occur, every occurrence of `before_tag` must precede every occurrence of `after_tag`; when either optional endpoint is absent, cardinality rules decide validity and the order edge is satisfied. The pass is linear in input children plus declared edges, so repeated fields never cause a full constraint-list rescan. Each allowed-enumeration value and each ordering edge counts as one constraint member for ExtensionRegistryLimits.

For inbound discriminator lookup, one temporary index is built over the received TTLV subtree and shared by all registered definitions. It stores one descriptor per indexed Structure and `(Tag, original child index)` pairs for its children, keeps duplicate tags, and never reorders or rewrites the generic subtree. The hard cap is 100,000 total payload Items including the root Structure, so there can be at most 100,000 Structure descriptors and 99,999 child pairs (199,999 records); the configured payload-index record maximum remains 200,000. A checked counting pass rejects excess records before reserve/allocation. Because KMIP Tags are 24-bit values, index construction uses a bounded fixed-pass radix ordering. Each path step uses lower- and upper-bound searches; it matches only when exactly one child has that tag. Missing or repeated tags are non-matches. A configured comparison-budget exhaustion is a stable redacted resource-limit error and returns no partial typed value. Payload nesting is governed by `CodecLimits`; the registry `maxDepth` applies to registered schemas and discriminator paths.

## ExtensionRegistryLimits

An immutable per-client configuration for locally selected limits. Defaults and hard maxima are KMIPKit policy, not OASIS requirements:

| Limit | Default | Hard maximum |
|---|---:|---:|
| Definitions per registry | 256 | 1,024 |
| Total schema nodes across definitions | 16,384 | 100,000 |
| Child rules in one Structure | 256 | 4,096 |
| Bytes per identity or Extension Information Text String field | 4,096 | 4,096 |
| Total identity/Extension Information text bytes across registry | 1 MiB | 16 MiB |
| Bytes per discriminator scalar | 4,096 | 4,096 |
| Total discriminator scalar bytes across registry | 1 MiB | 16 MiB |
| Constraint members in one allowed-value/order rule | 256 | 4,096 |
| Total constraint members across registry | 16,384 | 100,000 |
| Payload-index records per received subtree (Structure descriptors plus child pairs) | 200,000 | 200,000 |
| Discriminator child-tag comparisons per lookup | 1,048,576 | 4,194,304 |
| Schema nesting depth | 64 | 64 |
| Discriminator path depth | 64 | 64 |

Callers may raise or lower defaults but may never configure a value above its hard maximum; a field whose default equals its hard maximum cannot be raised. Construction counts identity and Extension Information text bytes, nodes, rules, discriminator bytes, and constraint members with checked arithmetic before cloning or reserving storage and rejects an over-limit input without producing a partial registry. Every Text String field in Extension Information counts against both its per-field cap and aggregate registry total. Inbound payload-index descriptors and child pairs are counted with checked arithmetic before allocation; lookup charges each tag comparison against its configured budget and returns a stable redacted limit error without producing a partial typed value. Enum and order indexes are compiled once per immutable schema; validation work is bounded by indexed child lookups, binary enum lookups, input traversal, and one pass over declared order edges. These caps apply to all language adapters.

## ExtensionDefinition

Immutable configuration record containing ExtensionIdentity, Compatibility, Discriminator, ExtensionSchema, and optional Extension Information metadata. Construction validates identifier syntax, non-empty metadata, identity/metadata byte limits, compatibility ranges, schema coherence, discriminator path, and unique registry key. Metadata strings never contain payload data.

## ExtensionInformation

A model of OASIS §7.13, Table 365, for local registry/query integration:

- Extension Name: required Text String.
- Extension Tag: optional Integer.
- Extension Type: optional Enumeration containing an Item Type value.
- Extension Enumeration: optional Integer.
- Extension Attribute: optional Boolean.
- Extension Parent Structure Tag: optional Integer.
- Extension Description: optional Text String.

The complete generic TTLV structure remains available for future fields. Query Function Extension List/Map are defined in §11.44, Table 476; this feature exposes corresponding local metadata views but does not implement Query.

## ClientExtensionRegistry

A client-owned immutable snapshot of validated ExtensionDefinition values.

- Registration order cannot change lookup, validation, encoding, or list/map results.
- Lookup is keyed by Vendor Identification and exact discriminator.
- A registry has no more definitions or aggregate schema rules than its ExtensionRegistryLimits permit; limits cannot exceed the hard maxima above.
- The registry does not own Client, Transport, TLS configuration, or global process state.
- Definitions and metadata views are deterministic.
- No registration can be added after client configuration is finalized.

## ValidatedExtensionValue

A sealed value created only by successful validation of an ExtensionSchema. It contains ExtensionIdentity and an owned generic TTLV subtree; typed field access does not mutate or normalize that subtree. Its default Debug/Display is redacted. KMIPKit-owned payload storage is zeroized on drop by the existing TTLV Value ownership contract. Borrowed access follows scoped-view conventions; application/runtime copies are outside the zeroization guarantee.

Protocol validation against an ExtensionSchema produces this value, but it does not prove that the schema belongs to a particular client registry and does not admit the value to outbound typed requests. Inbound recognition separately requires exactly one registered discriminator key and complete schema validation. Zero or multiple matching keys, or schema failure for the sole candidate, return an unrecognized generic value with no partial typed projection; multiple keys never select by registration order.

## RegisteredExtensionValue

A client-owned seal created only by `ClientExtensionRegistry` validation of a generic TTLV subtree against the definition found by exact registered identity. It contains the resulting ValidatedExtensionValue and preserves its identity and original subtree. This registry-scoped type is the only extension-value type accepted by outbound typed request construction.

## ClientRequestMessageExtension

Typed request wrapper containing one RegisteredExtensionValue and the caller-selected Criticality Indicator Boolean for this specific request use. Construction requires the Boolean; no KMIPKit default is applied. Multiple wrappers may be attached to one typed Request Batch Item in caller order as permitted by §8.3/Table 396. Encoding uses the standard Message Extension structure with the registered Vendor Identification and validated Vendor Extension Structure through the existing private request writer.

This name is specific to outbound request use. KMIPKIT-0007's existing `ClientMessageExtension` continues to represent preserved inbound Message Extension content.

## State transitions

Definition: unvalidated input -> validated immutable registration -> client registry snapshot.

Registry construction: generic TTLV subtree + exact identity -> complete registered-schema validation within CodecLimits and ExtensionRegistryLimits -> sealed RegisteredExtensionValue; absent identity or schema failure returns an error without a request-admissible value.

Inbound extension recognition: generic TTLV subtree -> bounded discriminator scan (zero or multiple hits -> unrecognized generic value) -> full schema validation of the sole candidate within CodecLimits and ExtensionRegistryLimits -> typed view over the unchanged subtree. Schema failure never yields a partially typed view.

Outbound use: registry-validated RegisteredExtensionValue + explicit Criticality Indicator -> ClientRequestMessageExtension -> typed Request Batch Item -> existing private encoding permit -> zeroizing byte owner retained through all partial transport writes and until success/error return, then initialized bytes are zeroized and the correct delivery state is reported without retry.
