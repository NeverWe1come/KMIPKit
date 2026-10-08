# Feature Specification: KMIP 2.1 Vendor Extension Registry

**Feature Branch**: `feature/KMIPKIT-0012-vendor-extension-registry`
**Created**: 2026-10-06
**Status**: Approved for implementation under the maintainer's standing direct authorization to execute the complete KMIPKit plan without further approval requests. This authorizes the defined feature scope for implementation; it does not claim line-by-line human review of this revision or waive PR approval and merge rules.
**Input**: Roadmap item `KMIPKIT-0012-vendor-extension-registry`, ADR-0007, ADR-0013, and `docs/architecture/extensions.md`.

## Scope and normative sources

KMIPKit 1.0 needs an immutable, client-owned registry so applications can create, send, receive, inspect, and preserve vendor extensions through equivalent Rust, C, Java, and Python capabilities. An extension is recognized only when its registered identity and declared discriminator match and the received content passes the complete registered data-only schema. Unknown non-critical extension content remains losslessly available; KMIPKIT-0007 rejects unknown critical extensions. Registry adapters are only this feature's API slice and do not satisfy the full 1.0 public API parity gate.

This feature defines registry metadata, deterministic registration and lookup, validated typed values, and registry-specific language-adapter behavior. It does not add server behavior, arbitrary generic-Item request input, raw-message execution, dynamic executable plugins, or a general-purpose vendor code SDK. It does not implement Query itself; it exposes local metadata for later typed Query integration.

The normative source is the immutable pinned OASIS KMIP Specification v2.1 at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. The normative catalog is `specification/catalog/kmip-2.1.json`. These APIs are KMIPKit product commitments, not new OASIS requirements; OASIS citations define the wire structures and behavior they must preserve.

| Stable requirement / source | Treatment |
|---|---|
| KMIPKIT-REQ-SPEC-8.3-004; §8.3, Table 396 | A client may append repeated Message Extension structures to a Batch Item; this feature preserves the caller's extension order. |
| `KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION`, `KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-IDENTIFICATION`, and `KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-EXTENSION`; §9.13, Table 418 | Preserve the standard Message Extension's fields, order, types, and generic subtree. Vendor Identification uses only `[A-Za-z0-9_.]`. |
| §9.13; `KMIPKIT-POLICY-EXTENSION-PRESERVATION` | Recognition uses a registered definition and validated content, not Vendor Identification alone. KMIPKIT-0007 rejects unknown critical extensions and preserves unknown non-critical extensions; this feature must not weaken or duplicate that execution behavior. |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-13-EXTENSION-INFORMATION`; §7.13, Table 365 | Expose Extension Information fields losslessly for later typed Query integration. This feature does not send Query. |
| `KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-LIST-00000005` and `KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-MAP-00000006`; §11.44, Table 476 | Provide deterministic local list and map metadata views corresponding to these Query functions; this is not implementation of Query. |
| `KMIPKIT-POLICY-VENDOR-VALUE-PRESERVATION` | Unknown vendor tags, enum values, bits, fields, multiplicity, and order remain representable and are not normalized by lookup. |

ADR-0007 and ADR-0013 define the project boundary: immutable per-client ownership; extension name/version, compatibility, schema, and conformance metadata; no recognition by Vendor Identification alone; no arbitrary conversion trait, raw Item, or raw body at Client::execute; no dynamic executable plugins. Existing decoder limits, redaction, zeroization, and transport isolation remain mandatory.

## Clarification record

Approved ADRs and architecture resolve the design choices:

- Registry definitions in this feature are data-only. They cannot execute scripts, callbacks, or dynamically loaded code. A future compiled vendor adapter SDK needs a separate reviewed specification.
- The registry is immutable after attachment to client configuration; no global mutable state exists.
- Public identity is Vendor Identification + extension name + version. KMIP's Message Extension does not standardize name/version fields. Each data-only definition therefore declares an exact discriminator path and value within the vendor payload, followed by complete schema validation; matching never uses Vendor Identification alone. The registry rejects duplicate discriminator keys for one vendor. Distinct keys may name different paths that are both present in one payload. Lookup is conservative: zero matching discriminator keys is unrecognized; more than one matching key is ambiguous and unrecognized without schema selection; exactly one key proceeds to complete schema validation and is recognized only if validation succeeds. No result depends on registration order.
- Duplicate identities, duplicate discriminator keys for one vendor, and conflicting compatibility declarations fail registry construction. Discriminator keys are finite path/value pairs, not executable predicates.
- A data-only schema may validate a generic TTLV subtree and produce a sealed validated value. Raw generic Items, raw KMIP bodies, and caller-implemented conversion traits cannot bypass validation or enter the typed execution boundary.
- Extension Information and Query list/map metadata are exposed locally; Query request execution and full response interpretation belong to a separate Query operation specification.

## User Scenarios & Testing

### User Story 1 - Register and construct a known extension (Priority: P1)

A client integrator declares vendor extensions for one client, receives an immutable validated registry with deterministic metadata views, and constructs and sends an outgoing typed extension value without manually building wire-format TTLV. The caller explicitly selects the Message Extension Criticality Indicator for each request use.

**Why this priority**: Registered construction is the central 1.0 commitment in ADR-0007 and enables language parity.

**Independent Test**: Build isolated clients with different definitions, create values through each registry, verify fake-transport request bytes for an explicitly selected Criticality Indicator, verify only matching typed values enter the typed request path, and compare normalized outcomes across Rust, C, Java, and Python.

**Acceptance Scenarios**:

1. **Given** a valid definition with vendor identifier, extension name/version, compatibility, and data-only schema, **when** client configuration is built, **then** an immutable registry with deterministic list/map views is attached.
2. **Given** two registered definitions and schema-valid values, **when** the caller attaches both to one typed batch item with explicit Criticality Indicators, **then** the existing private writer emits both standard Message Extension structures and payloads in caller order without a generic-Item or raw-body path.
3. **Given** invalid Vendor Identification characters, duplicate identity or discriminator key, incompatible versions, or an invalid schema, **when** the registry is built, **then** construction fails without exposing partial state or payload data.
4. **Given** an attempt to create a typed value from arbitrary Item, raw KMIP body, or executable plugin, **when** it reaches the public typed path, **then** no such route is available.

### User Story 2 - Recognize and inspect a registered extension (Priority: P1)

An application receives a vendor extension and obtains its validated typed view or original generic TTLV without losing unknown fields or order.

**Why this priority**: Safe interpretation and lossless preservation are both required for useful forward-compatible extensions.

**Independent Test**: Feed deterministic matching, mismatching, multiply-matching, malformed, unknown critical, and unknown non-critical extensions; assert recognition, preservation, and the KMIPKIT-0007 disposition.

**Acceptance Scenarios**:

1. **Given** exactly one definition matches vendor, discriminator, and complete schema, **when** the extension is inspected, **then** the typed value and unmodified generic subtree are both available.
2. **Given** a known vendor but different or absent discriminator, **when** lookup runs, **then** Vendor Identification alone does not establish recognition.
3. **Given** a payload containing two distinct registered discriminator paths whose exact values both match, **when** lookup runs, **then** it is unrecognized with no partial typed value and no registration-order preference, even if only one definition might pass later schema validation. A payload with a single discriminator candidate that fails its schema is likewise unrecognized.
4. **Given** an unrecognized critical or non-critical extension, **when** the response is processed, **then** KMIPKIT-0007 rejects the critical case and preserves the non-critical generic value.
5. **Given** unknown tags, enum values, bitmask bits, repeated fields, or ordering, **when** the generic subtree is inspected or round-tripped, **then** those values and order remain available.

### User Story 3 - Share metadata and validation across languages (Priority: P1)

A consumer receives equivalent registration, validation, lookup, and metadata behavior through Rust, C, Java, and Python.

**Why this priority**: Extension support is part of 1.0 language parity, not a Rust-only protocol feature.

**Independent Test**: Run one shared valid/invalid, outbound, and inbound fixture corpus through all four adapters and compare normalized results, error categories, typed recognition, and the preserved generic subtree.

**Acceptance Scenarios**:

1. **Given** one definition, **when** registered through each language, **then** identity, compatibility, validation, lookup, list/map views, and Extension Information are equivalent.
2. **Given** the same invalid value, **when** each adapter validates it, **then** each returns the same error category without exposing payload contents.
3. **Given** two clients with different registries, **when** either is used, **then** registrations and validation results do not leak between clients.
4. **Given** matching and non-matching inbound Message Extensions with unknown fields, **when** each adapter inspects them, **then** typed recognition and the unchanged generic subtree are equivalent across languages.

### Edge Cases

- Syntactically valid but empty, oversized, or duplicate vendor identifiers; apply only explicit project limits and Table 418.
- A discriminator path is absent, repeated, or has a value with the wrong TTLV type.
- Multiple versions under one vendor and overlapping payload shapes.
- A discriminator matches but another schema rule fails.
- Unknown non-critical values contain future tags, enums, or bits.
- Definitions conflict with core allocations or tags in the same parent structure.
- Malformed compatibility ranges, duplicate metadata entries, non-deterministic views, or schema traversal beyond configured decoder limits.
- Extension payloads may contain secrets; errors and default debug/display output must not reveal them, and KMIPKit-owned secret memory follows existing zeroization rules.
- Registry input is absent, empty, or contains multiple definitions for one vendor.

## Requirements

### Functional Requirements

- **KMIPKIT-0012-FR-001**: The client MUST attach an immutable extension registry to each client configuration. One registry MUST NOT affect another client or global state. *(ADR-0013; Constitution IV)*
- **KMIPKIT-0012-FR-002**: Every registration MUST declare a valid Vendor Identification, extension name/version, KMIP and KMIPKit compatibility, a data-only schema, and a finite exact discriminator path/value pair. Vendor Identification MUST follow §9.13, Table 418, including `[A-Za-z0-9_.]`.
- **KMIPKIT-0012-FR-003**: Registry construction MUST reject duplicate identities, duplicate discriminator keys for one vendor, invalid compatibility ranges, and conflicting definitions, without exposing partial state.
- **KMIPKIT-0012-FR-004**: This feature MUST accept only data-only schemas. It MUST NOT evaluate scripts, load dynamic executable plugins, invoke untrusted callbacks, or accept caller-implemented conversion traits.
- **KMIPKIT-0012-FR-005**: A typed extension value MUST be produced by validation through the immutable client registry: the registry resolves an exact registered identity and validates a schema-constructed or generic TTLV subtree against that registered definition. This MUST return a sealed `RegisteredExtensionValue`; a protocol-level `ValidatedExtensionValue` created from a standalone definition MUST NOT be accepted by the typed request API. A request-use wrapper MUST attach the registered value to a typed Request Batch Item (`ClientBatchItem` in Rust); repeated extensions MUST preserve caller order as permitted by §8.3, Table 396. The caller MUST explicitly provide the Criticality Indicator for each request use; KMIPKit MUST NOT choose a criticality default. Client execution MUST encode the registered Vendor Identification, explicit Criticality Indicator, and validated Vendor Extension Structure as the standard Message Extension defined by §9.13, Table 418, through the existing private request writer. The typed request boundary MUST accept only that sealed registry-validated value and MUST NOT accept an unvalidated Item, raw body, or caller conversion implementation as a substitute. *(KMIPKIT-REQ-SPEC-8.3-004; §8.3, Table 396; §9.13, Table 418; ADR-0012; ADR-0013)*
- **KMIPKIT-0012-FR-006**: Inbound recognition MUST use registered identity, declared discriminator, and complete schema validation; Vendor Identification alone MUST NOT establish recognition. Registry construction MUST reject duplicate discriminator keys for one vendor. Inbound processing MUST collect matching keys for the received vendor: zero matching keys MUST produce an unrecognized value; more than one matching key MUST produce an ambiguous unrecognized value without schema selection; exactly one matching key MUST proceed to complete schema validation and produce a typed value only if validation succeeds. No result may depend on registration order.
- **KMIPKIT-0012-FR-007**: Lookup and validation MUST preserve the generic subtree, unknown tags, enum values, bitmask bits, multiplicity, and order. A typed view MUST NOT normalize the preserved value.
- **KMIPKIT-0012-FR-008**: Unknown critical and non-critical actions MUST remain those implemented by KMIPKIT-0007 under §9.13. Registry integration MUST NOT weaken critical rejection or non-critical preservation.
- **KMIPKIT-0012-FR-009**: The registry MUST expose deterministic registry-local list/map metadata and an Extension Information model corresponding to §7.13, Table 365, for later integration with Query Extension List/Map (§11.44, Table 476). Local metadata MUST NOT claim that a remote server supports those extensions. This feature MUST NOT represent Query as implemented.
- **KMIPKIT-0012-FR-010**: Rust, C, Java, and Python adapters MUST provide equivalent registration, ExtensionRegistryLimits, validation, outbound typed Message Extension attachment with an explicit Criticality Indicator, inbound recognition and typed inspection, lossless generic subtree access, metadata, and error capabilities demonstrated with shared fixtures.
- **KMIPKIT-0012-FR-011**: Errors, logs, and default debug/display representations MUST NOT contain vendor payloads or secrets. KMIPKit-owned secret extension memory MUST follow existing zeroization policy; caller/runtime-copy limits must be documented. For secret-bearing outbound extension values, the zeroizing encoded-buffer owner MUST remain alive through every partial transport write until the transport returns; initialized encoded bytes MUST be zeroized on owner drop after success or error. Failed-request delivery state MUST be accurate and automatic retries MUST NOT occur. *(AGENTS.md §8; ADR-0012; KMIPKIT-0007-OD-006)*
- **KMIPKIT-0012-FR-012**: Schema processing MUST obey configured TTLV size, depth, and element limits and MUST NOT disable core validation, TLS, redaction, or transport isolation. Registry construction and validation MUST apply configurable per-client limits no greater than these hard maxima: 1,024 definitions, 100,000 total schema nodes, 4,096 child rules per Structure, 4,096 bytes per identity/Extension Information text field, 16 MiB total registry identity/Extension Information text, 4,096 bytes per discriminator scalar, 16 MiB total discriminator scalar bytes, 4,096 constraint members per rule, 100,000 total constraint members, 200,000 payload-index records, bounded by 100,000 total payload Items including the root Structure (at most 100,000 Structure descriptors and 99,999 child-tag/index pairs), 4,194,304 discriminator child-tag comparisons, and depth 64 for schemas and discriminator paths. Callers MAY raise or lower defaults, but MUST NOT set a value above its hard maximum; a value above a hard maximum MUST fail construction. Defaults are 256 definitions, 16,384 schema nodes, 256 child rules per Structure, 4,096 bytes per identity/Extension Information text field, 1 MiB total registry identity/Extension Information text, 4,096 bytes per discriminator scalar, 1 MiB total discriminator scalar bytes, 256 constraint members per rule, 16,384 total constraint members, 200,000 payload-index records, bounded by 100,000 total payload Items including the root Structure (at most 100,000 Structure descriptors and 99,999 child-tag/index pairs), 1,048,576 discriminator child-tag comparisons, and depth 64. Definition/schema limits MUST be checked with overflow-safe counters before cloning, reserving, or allocating; a construction limit failure MUST return no registry. A runtime payload-index or comparison-budget failure MUST return the stable redacted resource-limit error and no partial typed result. All variable-length C byte/string inputs MUST use typed pointer parameters and uint64_t byte lengths and reject over-limit lengths before dereferencing, reading, copying, or scanning input; implementations MUST NOT perform unbounded NUL scans. Registry construction MUST count UTF-8 bytes from each identity field and Extension Information Text String against per-field/aggregate text limits, discriminator scalar bytes, and every allowed-enumeration or ordering-constraint member, including values shared through reused builders. For inbound lookup, the implementation MUST build one bounded temporary index over the received TTLV subtree, containing at most 200,000 total records and bounded by the 100,000 total payload Items including the root Structure: at most 100,000 Structure descriptors plus at most 99,999 `(tag, original child index)` pairs, without changing child order or payload values. Since Tags are 24-bit, this index MUST be produced with bounded linear passes; duplicate tags MUST remain represented. All discriminator paths MUST share this index. Each path step MUST resolve a tag by lower/upper-bound search and match only when exactly one child has that tag; absent or repeated tags are non-matches. A search MUST consume no more than 35 child-tag comparisons per path step at the 100,000-payload-item hard limit. Across all definitions, lookup MUST stop within 1,024 definitions × 64 path steps and 4,194,304 child-tag comparisons; lower caller-configured comparison limits MUST fail with a stable redacted resource-limit error and no partial typed result. Allowed Enumeration values MUST be compiled into sorted numeric indexes and checked by binary search. Ordering constraints MUST be represented as acyclic, unique directed tag edges; construction MUST resolve endpoints to child-rule indexes, and validation MUST record first/last child positions in one input pass then check each edge once. For a present edge, every `before_tag` occurrence MUST precede every `after_tag` occurrence; absent endpoints are handled by cardinality and satisfy the ordering edge. Full schema validation MUST use a bounded indexed lookup and perform at most 13 child-tag comparisons per input Item, at most 13 enum-value comparisons per enum-valued Item, and one pass over the selected schema's constraints, plus linear schema/input traversal. It MUST NOT rescan enum or order-constraint collections for each repeated input item. These values are KMIPKit resource policy, not OASIS requirements.
- **KMIPKIT-0012-FR-013**: Public registry and typed-extension capabilities MUST be documented in English and Spanish with runnable, tested examples for each supported adapter.
- **KMIPKIT-0012-FR-014**: Repetitive registry declarations and adapter fixtures MUST be generated from the reviewed, checked-in `specification/api/public-api.json` manifest and committed. Later approved API specifications extend that manifest and own complete 1.0 API parity. CI regeneration MUST be deterministic and fail on a diff. This entry covers only the registry capability and MUST NOT claim 1.0 API completeness.

### Key Entities

- **Extension Identity**: Vendor Identification, extension name, and extension version. This local identity does not add name/version fields to the standard Message Extension.
- **Extension Definition**: Data-only compatibility, schema, discriminator, validation, and Extension Information metadata.
- **Client Extension Registry**: Immutable per-client definitions with deterministic lookup/list/map behavior.
- **Validated Extension Value**: Value created or recognized only after complete validation, retaining access to its unmodified generic subtree.
- **Registered Extension Value**: Client-owned seal produced only by validating a generic TTLV subtree against one exact definition in the immutable client registry. A standalone protocol-validated value cannot enter a typed request.
- **Client Message Extension**: A validated value plus the caller's explicit per-request Criticality Indicator, attached to a typed Request Batch Item.
- **Extension Information**: The §7.13/Table 365 metadata view for later typed Query integration.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0012-SC-001**: 100% of registration, outbound Message Extension, and inbound recognition/inspection/preservation fixtures produce the specified outcome across Rust, C, Java, and Python.
- **KMIPKIT-0012-SC-002**: Equivalent definitions produce identical normalized TTLV trees and deterministic list/map metadata in all four adapters.
- **KMIPKIT-0012-SC-003**: All unknown values in the preservation corpus remain inspectable and ordered after recognition and generic round-trip.
- **KMIPKIT-0012-SC-004**: Boundary audits find zero public path for raw bodies, arbitrary Item request input, dynamic executable plugins, or caller-defined conversion traits.
- **KMIPKIT-0012-SC-005**: Every feature requirement maps to an implementation artifact and passing verification; applicable normative traceability is 100%.
- **KMIPKIT-0012-SC-006**: Error/log/debug tests disclose zero extension payload contents on exercised failure paths.
- **KMIPKIT-0012-SC-007**: In all four adapters, definition, schema, metadata, discriminator-byte, constraint-member, payload-index, and lookup-comparison inputs at each configured limit and one unit above return deterministic outcomes without panic or partial state. A wide payload with a matching discriminator at the final child and repeated tags stays within the declared comparison bound and does not alter the generic subtree. Worst-case repeated-enum and ordered-field payloads assert the per-Item comparison bounds without quadratic constraint rescans.
- **KMIPKIT-0012-SC-008**: Secret-bearing outbound fixtures prove owner lifetime through partial writes and transport return, zeroization after success/error, NotSent/PossiblySent/ResponseStarted delivery classification, and zero retries.

## Assumptions

- KMIP 2.1 and TTLV remain the 1.0 protocol and encoding. This adds no transport, TLS, algorithm, or server behavior.
- OASIS defines the wire extension structure and criticality. The registry, typed values, data-only schema, and language parity are KMIPKit commitments.
- KMIPKIT-0006 owns generic lossless message models; KMIPKIT-0007 owns execution and common criticality behavior; KMIPKIT-0012 adds registry recognition, construction, typed values, and adapters.
- Query execution, profiles, and server-initiated operations belong to separate specifications. Metadata access does not claim server support or profile conformance.
- Decoder defaults remain 16 MiB, depth 64, and 100,000 elements unless separately authorized configuration changes them.
- Registry defaults and hard maxima are the explicit resource-policy values in FR-012; callers may raise or lower defaults up to hard maxima, but may not exceed any hard maximum. Lookup comparison exhaustion returns a stable redacted resource-limit error with no partial typed result.
