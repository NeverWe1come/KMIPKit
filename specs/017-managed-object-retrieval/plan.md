# Implementation Plan: KMIP 2.1 Get and Locate Operations

**Branch**: `feature/KMIPKIT-0017-managed-object-retrieval` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

## Summary

Implement typed TTLV request/response models and single-exchange client execution for KMIP 2.1 Get and Locate. Reuse the existing Unique Identifier, ordered AttributeSet, bounded generic TTLV, message/batch, common result, and transport contracts. Keep returned object bodies opaque, lossless, redacted, and zeroizing. Leave object search and ID Placeholder decisions on the server. Assign the applicable inventory records, OASIS tests, code, and executable verification in one reviewable specification/implementation sequence.

## Technical Context<br>

**Language/Version**: Rust Edition 2024; MSRV 1.94.<br>
**Primary Dependencies**: Existing `kmipkit-ttlv`, `kmipkit-protocol`, `kmipkit-client`, and workspace dependencies; no new runtime dependency is planned.<br>
**Storage**: None; the client does not maintain object state.<br>
**Testing**: Rust unit and integration tests, pinned OASIS test vectors where fixtures are available, property/negative TTLV tests, deterministic fake-transport tests, and the repository CI/coverage gates.<br>
**Target Platform**: Linux, Windows, and macOS supported by the workspace and CI.<br>
**Project Type**: Rust client library protocol feature.<br>
**Performance Goals**: No feature-specific throughput target. Respect existing response-size, depth, and element limits; do not allocate an unbounded object body.<br>
**Constraints**: KMIP 2.1, client initiated, TTLV only, raw TTLV over TLS or TTLV over HTTPS/HTTP 1.1, TLS 1.3 mutual TLS, rustls with aws-lc-rs; no local cryptographic provider, retry, automatic polling, Recover follow-up, or language-parity claim.<br>
**Scale/Scope**: Exactly two operations and the 15 linked catalog requirement rows (2 Get and 13 Locate); shared protocol requirements remain with their existing feature owners.

## Constitution Check

**Pre-design gate**: PASS.

- Protocol, encoding, direction, and transport remain inside the approved 1.0 boundary.
- The client does not create a local cryptographic algorithm provider or imply object-state knowledge.
- Get payloads use the existing redacted, zeroizing generic TTLV ownership model; diagnostics and failures do not expose raw object bodies.
- No unsafe code, global state, dependency, or server-side operation is introduced by the design.
- Unknown tags, enumeration values, bitmask bits, and extensions remain subject to the existing generic TTLV allocation and lossless-preservation contracts.
- Tests and traces cite exact OASIS sections/tables and stable catalog identifiers. Tests and implementation are gated on approved specification review.

**Post-design gate**: CONDITIONAL; the model and scope are bounded, but the open catalog decisions listed below prevent affected conformance claims. The design introduces two operation modules and adds variants to shared client dispatch. The release branch already contains the foundations needed for the feature. The API/dispatch changes may conflict with the in-flight KMIPKIT-0016 implementation branch; integration must update from `release/1.0.0` after that PR changes the release, then rerun the full checks. No branch may be based on the unmerged KMIPKIT-0016 feature branch.

## Phase 0: Research Decisions

See [research.md](research.md). The design follows OASIS Tables 220–222 and 247–249, cataloged requirements, and already-approved lossless/secret TTLV contracts. Open catalog issues KMIPKIT-DISC-015 and KMIPKIT-DISC-032 affect PKCS#12 guidance and test mapping. Six Locate requirement rows also have server-action summaries under client actor metadata. These are recorded in research.md and block conformance claims for the affected rules until catalog disposition. Locate's server matching rules and ID Placeholder state are preserved rather than implemented locally.

## Phase 1: Design & Contracts

See [data-model.md](data-model.md), [contracts/public-rust.md](contracts/public-rust.md), and [quickstart.md](quickstart.md).

### Design

1. Add operation-specific Get and Locate models to `kmipkit-protocol`, each retaining Table-defined field presence and order.
2. Model Get's Any Object as the existing generic ordered TTLV tree. Parse only the required response envelope fields; do not reconstruct object values, normalize nested structures, or format object contents.
3. Model Locate's Attributes with the existing direct-item AttributeSet. Model request selectors and results as their source-defined Integer, Enumeration, Structure, and repeated Unique Identifier values. Do not evaluate matching criteria, reorder identifiers, or infer ID Placeholder state.
4. Add the two request variants and one-exchange response handling to the existing client execution path. Reuse common Result Status/Reason/Message, Pending, redacted errors, codec limits, and delivery-state behavior.
5. Extend operation-specific tests, user guides, and traceability in the same implementation PR. Generated catalog reports are regenerated from the catalog tool; generated files are not edited manually.

### Alternatives Considered

- **One combined Get/Locate specification (selected)**: Both operations read managed-object state, share identifier and attribute foundations, and can be independently delivered as separate user stories within one bounded implementation PR.
- **Separate Get and Locate specifications**: Rejected for this increment because it duplicates common result, secret-payload, and client-dispatch gates without reducing the interface dependency. Tasks remain separately testable and can be split if implementation reveals independent size or risk.
- **Locally parse/retrieve cryptographic object variants**: Rejected because KMIPKit transports and manages cryptographic material and is not a local cryptographic algorithm provider; generic TTLV already preserves the complete object.
- **Run local Locate filters or automatic Recover/Get follow-ups**: Rejected because the client has no authoritative object store and this scope does not include Recover. Such behavior could disagree with server policy and object state.

## Implementation Gate

The latest release used for specification preparation is `release/1.0.0` at `227e3f9104f14494810598d078c013c389c24f8f`. That release includes KMIPKIT-0004 generic TTLV, KMIPKIT-0006 messages/batches, KMIPKIT-0007 typed client execution, KMIPKIT-0009 asynchronous outcome models, KMIPKIT-0013 transport, and KMIPKIT-0014 Create/AttributeSet implementation. Get and Locate are not implemented on this base. The pending KMIPKIT-0016 branch is not a dependency: Locate uses KMIPKIT-0014's direct-item `AttributeSet`, not the newer Attribute Reference type.

The specification/catalog assignment PR must be approved and merged before implementation. Before coding, the implementation branch must be created from the active release branch and recheck these dependencies. If an intervening merge changes any shared client contract, update this design and rerun the specification review before implementation.

## Project Structure

### Documentation (this feature)

```text<br>
specs/017-managed-object-retrieval/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── traceability.md
├── contracts/public-rust.md
├── checklists/requirements.md
├── checklists/protocol.md
└── tasks.md
```<br>

### Source Code (implementation PR only)

```text<br>
crates/kmipkit-protocol/src/get.rs
crates/kmipkit-protocol/src/locate.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/get_tests.rs
crates/kmipkit-protocol/tests/unit/locate_tests.rs
crates/kmipkit-client/src/lib.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/tests/unit/object_read_execution_tests.rs
docs/user-guide/en/object-retrieval.md
docs/user-guide/es/object-retrieval.md
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md # generated from catalog input<br>
specs/017-managed-object-retrieval/traceability.md
```<br>

**Structure Decision**: Extend the existing protocol and client crates. Each operation owns its payload schema and sanitized model error; common result, transport, decoder, batch, and secret-storage behavior stays in existing modules. The specification/catalog preparation artifacts are kept separate from the later implementation source tasks.

## Complexity Tracking

No constitution exception, additional crate, runtime dependency, unsafe code, server component, or parallel transport is proposed.
