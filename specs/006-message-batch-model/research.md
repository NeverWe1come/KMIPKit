# Research: KMIP 2.1 Message and Batch Model

## Inputs reviewed

- Accepted ADR-0002 fixes the 1.0 target at client-initiated KMIP 2.1 using TTLV and includes protocol-level asynchronous outcomes; older versions are later work.
- Accepted ADR-0003 places the typed message model in `kmipkit-protocol`, above `kmipkit-ttlv`; the client layer owns request construction defaults and runtime response correlation.
- KMIPKIT-0003 already provides raw-preserving `ResultStatus`/`ResultReason`, redacted `ResultMessage`, and `KmipOperationResult` validation. Reuse those types.
- KMIPKIT-0004 provides ordered generic `Item`/`Structure` with unknown Enumeration preservation, generic extension retention, and zeroized model-owned payloads. Use it for message conversion and opaque payloads.
- KMIPKIT-0005 remains a draft specification in this release snapshot. This feature's in-memory model does not call byte encoder/decoder APIs. Foundation implementation remains sequenced after KMIPKIT-0005 under the roadmap's shared-interface stabilization gate.
- Exact message fields/order come from the pinned OASIS Specification §§8.1–8.6, 9.1–9.2, 9.5–9.13, and 9.16–9.21, plus §10.1.2. Source copies are immutable.
- The generated inventory contains 567 requirements, 1,750 elements, and 40 open discrepancies. No official Test Cases IDs are linked to the message requirements in this feature, and all 203 referenced XML fixtures are unavailable locally.

## Decisions

### Keep the feature as an in-memory protocol model

- **Decision**: Define request/response wrappers and typed read-only views in `kmipkit-protocol`; validate and own the existing generic TTLV tree, then return it by ownership without producing bytes or cloning payloads. Leave runtime request/response matching and delivery orchestration to a later `kmipkit-client` feature.
- **Rationale**: The current layer architecture separates protocol structures from client orchestration. The generic model already preserves ordered fields and opaque values, so byte encoding is not needed to test this feature.
- **Alternatives considered**: Put KMIP message semantics in `kmipkit-ttlv` (violates layer ownership); encode and decode bytes directly in this feature (duplicates KMIPKIT-0005); implement all operation-specific payloads here (widens scope and prevents family-based operation work).

### Preserve exact source order and field presence

- **Decision**: Represent the Table 395/398 headers and Table 396/399 batch items in OASIS order. Preserve absent-versus-explicit optional fields, schema-authorized repeated fields, and all raw Enumeration values, including unassigned request-option values. TTLV-to-model parsing and model conversion do not apply outbound value rules; `KMIPKIT-0007-client-execution` owns value validation when emitting requests. Expose effective defaults for Batch Order Option (True), Batch Error Continuation Option (Stop), and omitted Asynchronous Indicator (Prohibited) without erasing field absence. Preserve Server Correlation Value generically if present in a client request tree, but do not expose it as a typed outgoing 1.0 client field.
- **Rationale**: OASIS defines schema order and default semantics; preserving presence supports lossless conversion and diagnostics while effective accessors support client defaults.
- **Alternatives considered**: Normalize all defaults into fields on parse (loses source presence); accept arbitrary field order for known Structures (contradicts §10.1.2).

### Keep per-item IDs and message correlation distinct

- **Decision**: Model Unique Batch Item ID as Byte String, Client and Server Correlation Values as Text Strings, and Asynchronous Correlation Value as Byte String. Table 396 requires a Unique Batch Item ID on every item when Batch Count exceeds one; single-item IDs remain optional under §9.21. The model checks request-local pairwise distinctness as a KMIPKit project invariant, not an OASIS requirement. `KMIPKIT-0007-client-execution` verifies response echoes and performs runtime matching.
- **Rationale**: These fields serve different purposes in §§9.1, 9.9–9.10, and 9.21. Client Correlation Value is optional and is not an item identity. Protocol models preserve IDs; client orchestration owns pairing a response with its request.
- **Alternatives considered**: Match by array index (unsafe when execution order can be arbitrary); use Client Correlation Value for per-item matching (wrong scope and no uniqueness guarantee); perform runtime matching in the protocol model (conflicts with ADR-0003 architecture).

### Represent asynchronous results but do not execute them

- **Decision**: A Pending result retains its raw status and required Asynchronous Correlation Value. The model preserves mixed per-item outcomes; `KMIPKIT-0007-client-execution` verifies that the associated request permits asynchronous responses. Poll/Cancel/Process operation models, use of the asynchronous correlation value in Poll/Cancel, and `PendingOperation` execution belong to `KMIPKIT-0009-asynchronous-operations`; there is no automatic polling, wait helper, or background work.
- **Rationale**: ADR-0002 includes protocol asynchronous outcomes in 1.0, while no-automatic-retry policy and synchronous public APIs rule out implicit polling. The model can be tested without transport.
- **Alternatives considered**: Automatically Poll or wait (violates no-retry/no-background policy and synchronous boundaries); reject mixed results (contradicts §§8 and 9.2).

### Keep response-only status fields conditional

- **Decision**: Reuse the existing Result Status/Reason contract and add Table 399 validation that Result Message is forbidden for Success and Operation Pending.
- **Rationale**: Table 399 makes Result Message optional only for statuses other than Success and Pending. The standalone result type does not own this response-item requiredness rule.

### Record, do not decide, open source conflicts

- **Decision**: Preserve raw Protocol Version major/minor values in this model without negotiation or runtime acceptance rules. ADR-0002 selects KMIP 2.1 only for 1.0; KMIPKIT-0007 enforces requests and responses at exactly 2.1 and tests acceptance of 2.1 plus rejection of all other major/minor pairs. Preserve raw Batch Error Continuation values and individual result statuses without assigning rollback/execution effects.
- **Rationale**: `KMIPKIT-DISC-022` records the conflict between §9.16 same-major compatibility and the accepted 2.1-only 1.x boundary. ADR-0002 resolves the product's 1.0 scope, so the later client-execution spec has a concrete 2.1-only behavior to enforce; the catalog discrepancy remains visible and no same-major compatibility claim is made. `KMIPKIT-DISC-001` records inconsistent Continue/Undo descriptions in §11.5. `KMIPKIT-DISC-039` records the §6.1.41 Query Asynchronous Requests source discrepancy affecting response mapping. `KMIPKIT-0009-asynchronous-operations` must review the exact normative source text, document and test its response-mapping decision, and avoid assuming a resolution in KMIPKIT-0006. This feature can represent the fields without implementing runtime policy.
- **Alternatives considered**: Implement older-version compatibility despite the accepted 1.0 boundary (would expand the release scope); omit the fields (would make the message model incomplete); or leave response acceptance without an owner or test (would leave the 1.0 contract ambiguous).

### Preserve opaque header and extension payloads

- **Decision**: Authentication, attestation, operation payloads, and vendor extension contents remain generic TTLV subtrees. Enforce only message-level structure covered by this spec; typed credentials, attestation truthfulness, extension recognition, and vendor SDK semantics remain later features. The immutable per-client registry is defined in `docs/architecture/extensions.md`; `KMIPKIT-0007-client-execution` applies its criticality actions under ADR-0007. `KMIPKIT-0008-credentials-attestation` owns whether the Attestation Capable Indicator is truthful.
- **Rationale**: This retains extensibility and secret-aware storage without preempting dedicated schemas. The `KMIPKIT-0007-client-execution` registry will reject unknown critical response extensions and may process unknown non-critical values as opaque data.
- **Alternatives considered**: Drop opaque fields (lossy); interpret vendor/credential content here (unreviewed scope expansion).

## Verification notes

- Derive table-driven model and conversion tests from the pinned Specification field tables. Do not describe them as official OASIS Test Cases: the catalog has no linked official case IDs for these message requirements and records all fixture artifacts unavailable.
- Record each normative `KMIPKIT-REQ-*` and applicable source clause in `specification/compliance/requirements/KMIPKIT-0006.csv`, including implementation/test paths and explicit deferrals.
- Test response ID retention and message-field order here. Test matching a response to an outstanding request in `KMIPKIT-0007-client-execution`; test Poll/Cancel use of Asynchronous Correlation Value in `KMIPKIT-0009-asynchronous-operations`.
- Test model diagnostics with sentinel credentials, extension payloads, operation bodies, and Result Message content; the existing TTLV model owns zeroization.

## Open implementation gates

- KMIPKIT-0005 design correction and ADR-0011 must be reviewed and accepted before codec implementation.
- The roadmap keeps one active foundation implementer until shared interfaces are stable; therefore KMIPKIT-0006 implementation does not begin before the codec foundation is implemented and reviewed, even though this in-memory model has no direct byte-codec dependency.
- Runtime version acceptance is specified by ADR-0002 as exactly KMIP 2.1 and assigned to KMIPKIT-0007 for enforcement and positive/negative tests; `KMIPKIT-DISC-022` remains the documented §9.16 scope exception. Continuation-option execution effects require a separate resolution of `KMIPKIT-DISC-001`.
- `KMIPKIT-0009-asynchronous-operations` owns the Process operation (§6.1.39) and `KMIPKIT-DISC-039`/§6.1.41 Query Asynchronous Requests response mapping. It must review the exact normative source conflict and document and test the resulting decision; KMIPKIT-0006 makes no mapping assumption.
