# Research: KMIP 2.1 Key Derivation and Transfer

## Decisions

### Reuse canonical operation data, object, and result models

**Decision**: Build Derive Key, Export, Import, and Join Split Key as operation-specific models over the existing protocol, object, attribute, generic TTLV, result, client, and transport APIs. Reuse existing Cryptographic Parameters and operation-data contracts.

**Rationale**: The approved scope adds four operations, not a second wire/model stack. Explicit values and lossless unknowns are existing product guarantees.

**Alternatives considered**: Adding a separate key material/crypto pipeline would duplicate shared types and violate the no-local-crypto boundary.

### Keep server error alternatives open

**Decision**: Carry Result Status, Result Reason, allowed Result Message, Pending/correlation, extensions, and delivery evidence through the existing common result path. Operation model errors describe local construction and malformed success-response shape only.

**Rationale**: Operation error tables describe results from the KMIP server; clients must preserve future/vendor values.

### Do not infer remote key state or execute cryptography

**Decision**: Never query a source key's Cryptographic Usage Mask, derive algorithm values, compress/wrap/unwrap a key, combine split material, import into local storage, or resolve ID Placeholder locally. Caller choices and values are passed to the server and server policy remains authoritative.

**Rationale**: The library transports/manages cryptographic material and is not a local algorithm provider. Explicit choices are part of the approved product boundary.

### Require caller-supplied Join Split Key threshold context

**Decision**: Every transmitted Unique Identifier occurrence in Join Split Key is paired in the typed request with the caller-known Split Key Threshold of that referenced split key. The context is required, ordered with identifier entries, client-side only, and never encoded. Before transmission, reject absent/misaligned context and reject when the number of identifier entries (including repeated field occurrences and repeated identifier values) is lower than the maximum supplied threshold.

**Rationale**: OASIS §6.1.27 requires the number of identifiers to be at least the threshold defined in the split keys. The ID-only request cannot reveal that threshold, and FR-002 prohibits inferring remote state or adding a hidden Get/Locate. The caller supplies already-known metadata. The maximum satisfies each per-key minimum without requiring threshold equality; emitted IDs remain exactly in caller order with repetitions preserved.

**Alternatives considered**: An implicit Get Attributes/Locate adds another request and violates one-call/no-hidden-lookup behavior. Delegating the check solely to the server leaves a client SHALL unenforced. De-duplicating identifiers or requiring identical thresholds is not stated in the source and is not added. This interpretation was approved on 2026-10-11 and is recorded in [spec.md](spec.md).

### Gate language bindings on the canonical API manifest

**Decision**: Implement canonical Rust models/client calls independently. Do not define ad hoc binding signatures or hand-edit generated declarations; use KMIPKIT-0043 tasks T046, T047, T050, and T052 for operation mappings and parity after its v2 manifest is available.

**Rationale**: KMIPKIT-0043 FR-003/005/006/021 makes the approved Rust operation contract the source for the four-language map and gives the manifest/binding work a single owner.

## Normative source and fixture evidence

Exact sections/tables and stable requirement IDs are in [spec.md](spec.md). Its approved test inventory maps Derive Key to KMIPKIT-TEST-CN01-2-14 through -19, Export and Import to CN01-2-43 through -47, and Join Split Key to no direct catalog case ID. The implementation checks pinned fixture presence before using a case; table-derived tests do not count as official fixture passes.

KMIPKIT-DISC-025, KMIPKIT-DISC-021, and KMIPKIT-DISC-049 retain their existing disposition and IDs. The 2026-10-11 Join Split Key threshold interpretation adds no discrepancy ID and does not change any ledger ID or pinned OASIS source.
