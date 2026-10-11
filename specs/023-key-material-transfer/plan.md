# Implementation Plan: KMIP 2.1 Key Derivation and Transfer

**Branch**: `feature/KMIPKIT-0037-key-material-transfer-implementation` (distinct branch because the prescribed name is occupied by another task) | **Base**: `release/1.0.0` at `af172974cd5c6f56f7f938574e1546d4e295a9ef` | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: Approved feature specification at `specs/023-key-material-transfer/spec.md`, including the maintainer-approved Join Split Key threshold interpretation dated 2026-10-11.

## Summary

Add typed KMIP client support for Derive Key, Export, Import, and Join Split Key. Reuse the existing object, attribute, Cryptographic Parameters, operation data, batch/result, transport, and secret-value contracts. Preserve explicit caller choices, repeated fields, opaque key material, unknown values, and KMIP result data. Perform no local KDF, wrapping, import/export cryptography, implicit remote lookup, retry, or poll. Join Split Key requires ordered caller-supplied threshold context for every transmitted identifier occurrence and validates it locally before sending.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94; C11, Java 17, and Python 3.12 through the approved binding surface.

**Primary Dependencies**: Existing KMIPKit workspace crates and pinned catalog/API generators only; no new runtime dependency is expected.

**Storage**: N/A.

**Testing**: Protocol-table unit tests; conditional/malformed request and response tests; source-derived TTLV vectors; fake-transport tests; repeated-field/property round-trips; secret redaction/zeroization tests; C, Java, and Python parity fixtures after the canonical API mapping is available; catalog and generated-output validation.

**Target Platform**: Existing Linux, Windows, and macOS CI matrix.

**Project Type**: Synchronous Rust client/protocol library with C ABI, Java JNI, and Python CFFI product facades.

**Performance Goals**: No new throughput or latency target and no local cryptographic operations. Each explicit operation call performs at most one request/response exchange.

**Constraints**: KMIP 2.1 and TTLV only; client-initiated operations; use caller-specified algorithms, lengths, formats, wrapping, and object data; preserve unknown values and order; redact/zeroize KMIPKit-owned secrets; no remote preflight or implicit cryptographic behavior.

**Scale/Scope**: Four operations and all 18 stable client requirement IDs in the approved ledger.

## Constitution Check

- Exact OASIS references, tables, stable requirement IDs, and approved interpretations are in [spec.md](spec.md): PASS.
- 1.0.0 client/TTLV/transport and no-local-crypto boundaries are unchanged: PASS.
- Caller-chosen algorithm/format/wrapping inputs remain explicit; key/data bytes are protected: PASS.
- Unknown tags, enumerations, bitmask bits, extensions, repeated identifiers, result status/reason, Pending, and correlation are preserved: PASS.
- Join Split Key's local threshold validation uses explicit caller metadata; it does not add a lookup or client inference of remote state: PASS.
- Red, Green, and Refactor are separate development commits per operation story: PASS.
- Rust operation/client work can proceed without KMIPKIT-0043; four-language manifest/binding completion uses its canonical scope and operation tasks: PASS.
- No pinned source, constitution, accepted ADR, or generated output is changed by hand: PASS.

## Design

### Dependency and sequencing gate

The active release base supplies the generic TTLV model and codec, typed messages/batches, object and attribute forms, credentials/secrets, common result handling, synchronous client execution, and TLS/HTTPS transports. Reuse shared operation-data and Cryptographic Parameters models where the current release exposes them; verify the exact public constructors at implementation start. Do not re-create those shared types.

Each operation has a separate payload module and test file. Their client variants share `crates/kmipkit-client/src/execute.rs`; dispatcher edits are serialized. No operation in this feature depends on another operation here first.

KMIPKIT-0043 owns the complete public API manifest and Rust/C/Java/Python parity matrix. Canonical Rust protocol/client operation work is independent and may start first. Binding signatures, generated declarations, and four-language parity completion follow the 0043 canonical operation mapping tasks T046, T047, T050, and T052 after its manifest work is integrated.

### Protocol models

Add `derive_key.rs`, `export.rs`, `import.rs`, and `join_split_key.rs` under `crates/kmipkit-protocol/src/`; export documented request, response, and sanitized model-error types from `lib.rs`.

Use shared generic/typed object, attribute, Cryptographic Parameters, operation-data, result, and zeroizing-byte forms. Optional field absence, ordered children, repeated Unique Identifier occurrences, unknown tags/enums/bitmasks, and extension values remain distinct and lossless. No operation model executes KDFs, wraps/unwraps keys, compresses public keys, imports into storage, or combines split-key material.

Apply local validation only where the approved specification identifies a client obligation decidable from request data. Derive Key uses explicit source IDs, Cryptographic Usage Mask, length, algorithm, result attributes, method, and parameters; it never substitutes ID Placeholder. Import validates explicit replacement intent and caller-supplied attributes/object. Export preserves caller-selected format/wrap/compression options. Join Split Key carries one non-wire Split Key Threshold context with each Unique Identifier occurrence in the same order, then validates the serialized identifier-entry count against the maximum supplied threshold. The context is mandatory and never encoded; IDs and repetitions are emitted exactly as requested. No threshold equality or identifier de-duplication rule is added.

### Client execution

Register four operation codes and typed request/response dispatch in `crates/kmipkit-client/src/execute.rs`; expose explicit synchronous convenience methods following existing client conventions. Each explicit call uses the existing request encoder, transport, result association, and delivery-state path once. Batch APIs preserve item ordering and the caller's batch/correlation values. No automatic retry, Poll, remote Get/Locate, algorithm execution, key wrapping, or follow-up request is added.

### Catalog, traceability, and bindings

Map the 18 normative requirement IDs and all mandatory/conditional request and response table rows to code/tests in the implementation PR. Update the normative catalog, then run its pinned validator and report generator; never hand-edit generated files. Coordinate shared catalog/report changes with parallel feature work.

Four-language mapping follows KMIPKIT-0043. Its tasks T046 (Derive Key), T047 (Export), T050 (Import), and T052 (Join Split Key) own manifest declarations, binding mapping, and parity evidence. They are a gate for the 1.0.0 parity requirement, not for Rust protocol/client development.

### Test strategy

- Cover Tables 205–207, 217–219, 238–240, and 244–246 and Derivation Parameters §7.12/Table 364.
- Test required, optional, and conditional fields, exact order, unknown values, open enumeration round trips, repeated IDs, missing/duplicate/malformed payload values, and result/error variants.
- For Derive Key, test source-key identifier, usage/length/algorithm, output attributes, derivation input and PRF/mode parameters, ID Placeholder prohibition, and no local derivation.
- For Export/Import, test explicit formats/wrapping/compression, required object/attributes, replacement behavior, and redacted secret values without performing cryptography.
- For Join Split Key, require a caller threshold for every identifier occurrence; assert pre-send failure when context is absent/misaligned or the count is below the maximum threshold, count repeated field/value entries as transmitted, preserve their order, and do not fetch object attributes or require thresholds to match.
- Use fake transports to verify operation code, one exchange, result/batch association, delivery state, Pending passthrough, and no retry.
- Catalog-linked Test Case IDs appear in [spec.md](spec.md). Test only local pinned fixtures; label table-derived tests and do not claim an unavailable official fixture or profile pass.
- Run formatter, Clippy, focused/workspace tests and coverage, generator/catalog checks, security/dependency checks, and cross-platform CI during implementation.

## Project Structure

### Documentation (this feature)

```text
specs/023-key-material-transfer/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── traceability.md
└── contracts/
    └── public-rust.md
```

### Source code (implementation phase)

```text
crates/kmipkit-protocol/src/derive_key.rs
crates/kmipkit-protocol/src/export.rs
crates/kmipkit-protocol/src/import.rs
crates/kmipkit-protocol/src/join_split_key.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs
crates/kmipkit-protocol/tests/unit/export_operation_tests.rs
crates/kmipkit-protocol/tests/unit/import_operation_tests.rs
crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/src/lib.rs
crates/kmipkit-client/tests/unit/derive_key_execution_tests.rs
crates/kmipkit-client/tests/unit/export_execution_tests.rs
crates/kmipkit-client/tests/unit/import_execution_tests.rs
crates/kmipkit-client/tests/unit/join_split_key_execution_tests.rs
docs/user-guide/en/key-derivation-and-transfer.md
docs/user-guide/es/derivacion-y-transferencia-de-claves.md
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md  # regenerate using the pinned tool
```

**Structure Decision**: Use one protocol model and unit-test file per operation, then integrate through the existing shared dispatcher. Keep cross-language manifest/binding work in KMIPKIT-0043.

## Risks and mitigations

- **Secret-bearing request/response fields**: use the existing secret owner, redact formatting/errors, and test sentinel bytes.
- **No local crypto**: model byte/form choices and send to server; do not derive, encrypt, wrap, unwrap, or combine locally.
- **Threshold context mismatch**: tie each threshold to its exact ordered identifier occurrence, require one per occurrence, validate the count against max, and transmit identifiers unchanged.
- **Shared client/catalog hotspots**: serialize dispatcher, catalog, and generated report integration.
- **0043 not yet integrated**: proceed with Rust; hold only binding parity closure.
- **Unavailable/incomplete fixtures**: table-derived vectors remain distinct from official Test Case evidence.

## Complexity Tracking

No constitution violation or accepted architecture change is proposed.
