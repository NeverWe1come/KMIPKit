# Implementation Plan: KMIP 2.1 Get and Locate Operations

**Branch**: `feature/KMIPKIT-0017-managed-object-retrieval` | **Date**: 2026-10-10 | **Spec**: [spec.md](spec.md)

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
**Scale/Scope**: Exactly two operations and 15 linked catalog inventory rows (2 Get and 13 Locate): 7 client-applicable Locate, 5 server-only Locate, and 1 retired Locate extraction. Shared protocol requirements remain with their existing feature owners.

## Constitution Check

**Pre-design gate**: PASS.

- Protocol, encoding, direction, and transport remain inside the approved 1.0 boundary.
- The client does not create a local cryptographic algorithm provider or imply object-state knowledge.
- Get payloads use the existing redacted, zeroizing generic TTLV ownership model; diagnostics and failures do not expose raw object bodies.
- No unsafe code, global state, dependency, or server-side operation is introduced by the design.
- Unknown tags, enumeration values, bitmask bits, and extensions remain subject to the existing generic TTLV allocation and lossless-preservation contracts.
- Tests and traces cite exact OASIS sections/tables and stable catalog identifiers. Tests and implementation are gated on approved specification review.

**Post-design gate**: SATISFIED by the release-contract merge; the model and scope are bounded and KMIPKIT-DEC-003 through -005 resolve the catalog issues. The initial specification and inventory-disposition revisions were human-merged in PRs #63 and #79. The release-contract refresh merged in PR #81 at `fd6b7782`. T001 records the implementation branch baseline and verifies the shared client contracts before implementation tests. The design introduces two operation modules and adds variants to shared client dispatch. The API/dispatch changes may conflict with other in-flight operation branches; implementation must start from the then-current `release/1.0.0` and rerun the required checks.

## Phase 0: Research Decisions

See [research.md](research.md). The design follows OASIS Tables 220–222 and 247–249, cataloged requirements, and already-approved lossless/secret TTLV contracts. KMIPKIT-DEC-003 treats lowercase PKCS#12 “shall” as descriptive server guidance. KMIPKIT-DEC-004 maps and pins both official PKCS#12 fixtures. KMIPKIT-DEC-005 classifies five Locate rows as server-only and retires one false normative extraction. Locate's server matching rules and ID Placeholder state remain server-owned.

## Phase 1: Design & Contracts

See [data-model.md](data-model.md), [contracts/public-rust.md](contracts/public-rust.md), and [quickstart.md](quickstart.md).

### Design

1. Add operation-specific Get and Locate models to `kmipkit-protocol`, each retaining Table-defined field presence and order.
2. Model Get's Any Object as the existing generic ordered TTLV tree. Parse only the required response envelope fields; do not reconstruct object values, normalize nested structures, or format object contents.
3. Model Locate's Attributes with the existing direct-item AttributeSet. Model request selectors and results as their source-defined Integer, Enumeration, Structure, and repeated Unique Identifier values. Do not evaluate matching criteria, reorder identifiers, or infer ID Placeholder state.
4. Add the two request variants and one-exchange response handling to the existing non-exhaustive `ClientRequest`, `ClientOperation`, and `ClientBatchOutcome` surfaces. Reuse common Result Status/Reason/Message, Pending, redacted errors, codec limits, and delivery-state behavior; do not change shared batching or delivery semantics.
5. Parse responses through `ResponseBatchItemView`. Its `with_ttlv` callback lends the full ordered response tree only for the callback lifetime; use the planned fallible `Item::try_clone` to copy only the Get Any Object item into KMIPKit-owned TTLV when the typed response must outlive that view. Preserve repeated and unknown descendants and keep the copied object redacted and zeroizing. Reuse the current validated `AttributeSet` contract: the direct-item model originated in KMIPKIT-0014 and subsequent catalog validation is present in the release baseline.
6. Extend operation-specific tests, user guides, and traceability in the same implementation PR. Generated catalog reports are regenerated from the catalog tool; generated files are not edited manually.

### Alternatives Considered

- **One combined Get/Locate specification (selected)**: Both operations read managed-object state, share identifier and attribute foundations, and can be independently delivered as separate user stories within one bounded implementation PR.
- **Separate Get and Locate specifications**: Rejected for this increment because it duplicates common result, secret-payload, and client-dispatch gates without reducing the interface dependency. Tasks remain separately testable and can be split if implementation reveals independent size or risk.
- **Locally parse/retrieve cryptographic object variants**: Rejected because KMIPKit transports and manages cryptographic material and is not a local cryptographic algorithm provider; generic TTLV already preserves the complete object.
- **Run local Locate filters or automatic Recover/Get follow-ups**: Rejected because the client has no authoritative object store and this scope does not include Recover. Such behavior could disagree with server policy and object state.

## Implementation Gate

The historical release-contract snapshot reviewed for this refresh was `release/1.0.0` at `4e15a8f15c4c8529426b004737aaf16961d1c073`. The implementation branch baseline is active `release/1.0.0` at `fd6b7782b4efe81c9234999cf2faacef64e5fcf6`, where the release-contract refresh merged in PR #81. The baseline includes the existing bounded generic TTLV, message/batch, typed client dispatch, asynchronous outcomes, transport, and current validated `AttributeSet` contract, plus KMIPKIT-0021's Hash, MAC, and signature request/response variants. PR #78 added `ResponseBatchItemView::with_ttlv`, which lends the complete ordered response batch tree for callback-scoped access; `StructureView::try_clone` clones a whole Structure, so this feature adds `Item::try_clone` to retain only the Any Object item. The `AttributeSet` source did not change between the immediately prior release baseline and this one; its current contract includes validation added after its initial KMIPKIT-0014 introduction. Get and Locate remain unimplemented. Locate uses the current direct-item `AttributeSet`, not the newer Attribute Reference type.

The original specification PR #63 and catalog-disposition PR #79 are human-merged. The release-contract refresh merged in PR #81 at `fd6b7782`; T001 records the implementation branch baseline and rechecks these dependencies. If another intervening merge changes any shared client contract, update this design and rerun the specification review before implementation.

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
crates/kmipkit-ttlv/src/item.rs
crates/kmipkit-ttlv/tests/public_item_contract.rs
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
