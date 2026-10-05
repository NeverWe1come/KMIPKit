# Implementation Plan: KMIP 2.1 Message and Batch Model

**Branch**: `feature/KMIPKIT-0006-message-batch-model` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/006-message-batch-model/spec.md`

## Summary

Add ordered, raw-preserving request/response message and batch-item types to `kmipkit-protocol`. Convert these models to and from the accepted generic TTLV tree, validate common message structure, and expose asynchronous Pending response data. Do not add transport I/O, runtime response matching, automatic Poll/Cancel, operation-specific payload schemas, or version/continuation policies blocked by open discrepancies.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94
**Primary Dependencies**: Existing workspace `kmipkit-ttlv` and `kmipkit-protocol` Result types; no new runtime dependency planned. Property tests may add one test-only framework in the Red commit only after `dependency-review.md` records the capability, alternatives, maintenance and security history, MSRV, license, platform support, and transitive cost required by `docs/development/coding-standards.md`.
**Storage**: In-memory only; generic TTLV owns and zeroizes payload storage.
**Testing**: External Rust integration tests, deterministic property tests for conversion/order validation, doctests, full workspace checks, and coverage gates. Each property uses 256 cases with seed `0x4B4D49504B495430`; the chosen test framework must support a fixed seed and disabled external failure-state persistence.
**Target Platform**: Supported Rust workspace platforms (Linux, Windows, macOS).
**Project Type**: Rust protocol library.
**Performance Goals**: Message model conversion is linear in the number of items/fields and performs no wire-byte allocation or network I/O.
**Constraints**: Preserve OASIS field order; do not log/format payloads; retain raw future values; no unsafe code; no auto retries or async tasks; no policy guesses for DISC-001. ADR-0002 fixes the 1.0 protocol version to KMIP 2.1 only; document that scope exception to §9.16 and assign enforcement and positive/negative tests to KMIPKIT-0007.
**Scale/Scope**: Common client-initiated KMIP 2.1 message/header/batch shape, unknown values, and Pending outcome data; operation payloads stay generic.

## Constitution Check

- Approved-scope gate: **Pass for design**. The 1.0 client, TTLV-only, async result, Rust core boundaries remain unchanged. This plan does not authorize implementation until the required spec/design review and foundation dependency gate are satisfied.
- Normative traceability: **Pass with explicit gaps recorded**. Exact OASIS clauses/catalog IDs are named in `spec.md`; server-only duties and open discrepancies are not recast as client obligations. Implementation tasks must create the `KMIPKIT-0006.csv` mapping before completion.
- TDD: **Pass**. Separate Red, Green, and Refactor commits are required; derived tests are not represented as official fixtures.
- Security/lossless handling: **Pass**. Opaque payloads use existing zeroizing TTLV ownership; diagnostics redact them; parser byte limits remain KMIPKIT-0005 scope.
- Layering: **Pass**. Message structures/conversion live in `kmipkit-protocol`; request defaults and runtime matching remain `kmipkit-client` work.
- Human governance: **Pass**. This branch prepares design artifacts and a draft PR only. No self-approval, merge, release, or unresolved ADR acceptance is planned.

## Project Structure

### Documentation (this feature)

```text
specs/006-message-batch-model/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/rust-message-model.md
├── checklists/requirements.md
├── checklists/message-model.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/kmipkit-protocol/src/message/
├── mod.rs                 # public message model exports
├── version.rs             # raw-preserving protocol version pair
├── header.rs              # ordered request/response headers
├── batch.rs               # ordered request/response batch items
├── conversion.rs          # typed model <-> generic TTLV tree
└── validation.rs          # message invariants and payload-free errors

crates/kmipkit-ttlv/tests/
├── structure_view.rs     # scoped borrowed access to generic Structures

crates/kmipkit-protocol/tests/
├── message_contract.rs    # external public API and construction contract
├── message_conversion.rs  # ordered generic TTLV conversion
└── message_validation.rs  # batch, ID, async, and extension negatives

specification/compliance/requirements/KMIPKIT-0006.csv
```

**Structure Decision**: Add a private `message` module with focused public types re-exported from `kmipkit-protocol`; keep testable public contracts in external integration tests. Update the catalog/report traceability and `docs/architecture/public-api.md` in the same implementation PR. No generated output is hand-edited.

## Architecture and Validation

1. `RequestMessage` and `ResponseMessage` own a validated generic `Structure`; each exposes typed, read-only header and batch-item views. Owning the original tree preserves unknown fields, source order, and zeroizing payload storage without cloning secrets.
2. Each batch item carries a raw-preserving Operation value, its generic request/response payload, optional Unique Batch Item ID, optional Message Extension values (repeatable in requests and singleton in responses), and response result fields as applicable.
3. Typed view accessors read `kmipkit_ttlv::Structure` in the exact Table 395/398 and Table 396/399 order. Builders emit known fields in schema order; parsing preserves the original ordered tree. The generic model adds a scoped read-only Structure view; conversion back consumes and returns the owned Structure unchanged.
4. Structural validators check required fields, allowed cardinality, batch count equality, required IDs on multi-item requests and their pairwise distinctness as a KMIPKit project invariant, option/cardinality constraints, Pending correlation, Result Message status constraints, and Message Extension shape. Request parsing preserves raw Asynchronous Indicator and Batch Error Continuation values, including otherwise unassigned values, without outbound send validation. The model does not compare responses to a prior request, gate Pending results on the request indicator, enforce Maximum Response Size, generate or source a request Time Stamp, or interpret unresolved option semantics; `KMIPKIT-0007-client-execution` owns those cross-message checks, outbound option-value acceptance/rejection under Tables 432/435, response-delivery limit, request Time Stamp passthrough/omission policy and the OD-004 countdown-source disposition, and client extension registry actions.
5. Validation failures use safe message/model error categories with offsets/field identifiers only; no raw TTLV, result text, or credential/operation payload is retained in error context.

## Implementation Gates

- Before starting implementation, verify every KMIPKIT-0005 precondition applicable to its codec foundation and confirm the foundation is accepted and merged into the current `release/1.0.0` head. Use the implementation gate and T001 evidence record in [`specs/005-ttlv-wire-codec/tasks.md`](../005-ttlv-wire-codec/tasks.md), the KMIPKIT-0005 status and FR-013 in [`specs/005-ttlv-wire-codec/spec.md`](../005-ttlv-wire-codec/spec.md), and the authoritative ADR status/decision sources [`docs/adr/README.md`](../../docs/adr/README.md), [`ADR-0011`](../../docs/adr/0011-reserved-tag-decoding-policy.md), and [`ADR-0012`](../../docs/adr/0012-caller-requested-wire-encoding-policy.md). Record revision-bound evidence for the human approval of the KMIPKIT-0005 specification and its correction, acceptance of ADR-0011, the release-head update/rebase, inspection of the merged KMIPKIT-0004 Tag/Item APIs and accepted ADR-0010, and review/acceptance/merge of the codec foundation. Before any KMIPKIT-0005 work relying on FR-013, the evidence must separately establish all three human approvals: ADR-0012 acceptance, approval of the exact KMIPKIT-0005 specification revision, and approval of the exact enforceable-boundary design described by ADR-0012 and KMIPKIT-0005 FR-013. The KMIPKIT-0005 T001 record is the evidence index; Draft/Proposed statuses and unchecked reviewer-owned checklist items are not proof of approval. Preserve KMIPKIT-0005's boundary: it adds no `Client::execute`, permit type/constructor, or production writer callsite; the first client feature/spec owns that path and its passing integration gate before merge, enablement, or release. This gate does not replace KMIPKIT-0006's own reviewer and human-approval gates in T001, and this specification remains Draft until human approval.
- No policy resolution for `KMIPKIT-DISC-001` is embedded here. ADR-0002 selects KMIP 2.1 only for 1.0; `KMIPKIT-0007-client-execution` enforces the 2.1-only send/accept rule with positive and negative version tests, and `KMIPKIT-DISC-022` documents that scope exception to §9.16 without claiming same-major backward compatibility. The immutable per-client registry is defined in `docs/architecture/extensions.md`; ADR-0007 requires extension preservation and criticality handling, and `KMIPKIT-0007-client-execution` implements recognition/actions. This protocol parser preserves extension values without classifying them.
- Rebase this feature branch from the then-current `release/1.0.0` before implementation/PR review; its starting SHA is recorded by Git history.
- `KMIPKIT-0007-client-execution` owns paired Operation/ID validation, mixed synchronous/asynchronous response handling under `KMIPKIT-REQ-SPEC-8-003-002`, Pending permission checks, extension criticality, outbound Asynchronous Indicator and Batch Error Continuation validation under Tables 432/435, ADR-0002's KMIP 2.1-only request/response version enforcement (including positive and negative tests for `KMIPKIT-DISC-022`), `KMIPKIT-REQ-SPEC-9.12-001-002` Maximum Response Size enforcement, and `KMIPKIT-REQ-SPEC-9.12-001-003` large-response recommendations. Its acceptance tests include a deterministic fake-transport response batch containing both completed and Pending items, checking that Pending is accepted when the request indicator permits asynchronous results and rejected otherwise; assigned and extension-range option acceptance plus negative tests for unassigned values outside those ranges; exact-limit and over-limit response-size boundaries; and configured Maximum Response Size coverage for operations likely to return large responses. `KMIPKIT-0009-asynchronous-operations` owns use of exact Asynchronous Correlation Value bytes in explicit Poll/Cancel requests, with tests that verify byte-for-byte preservation for both operations and no automatic polling or retry. It also owns `KMIPKIT-DISC-039`/§6.1.41 Query Asynchronous Requests response mapping: review the exact normative source conflict, document and test a decision, and assume no resolution in this feature. `KMIPKIT-0008-credentials-attestation` owns truthful Attestation Capable Indicator behavior. Each follow-on needs its own approved specification and executable acceptance tests before implementation.

## TDD and Verification Strategy

- **Red**: Add external API tests first for message, header, and batch-item order; lossless owned-tree round trips; counts; optional single-item and required multi-item IDs with project-invariant uniqueness; optional-vs-default preservation; rejection of either Batch Order Option or Batch Error Continuation Option on a one-item batch; parse-time preservation and round trips for assigned, extension-range, and otherwise unassigned option Enumeration values; pending correlation requirements; Result Message status restrictions; request/response Message Extension cardinality and shape; scoped Structure views; and diagnostic redaction. Outbound rejection tests for option values outside Tables 432/435 belong to `KMIPKIT-0007-client-execution`. Run focused Rust 1.94 tests and record expected failures.
- **Property-test contract**: Use fixed-seed bounded generators with 256 cases per property. Generate messages with 1–8 batch items, at most 16 generated fields per item, nesting depth at most 8, and at most 128 generated fields per message; byte-string samples are at most 1,024 bytes, with test-only identifier samples of 1–32 bytes. Generate raw Enumeration values across the complete `u32` domain, Bit Mask values across the complete `u32` domain, and only Tags accepted by the existing generic TTLV allocation policy. Include repeated unknown fields and schema-authorized repeatable fields in source order, and opaque payload subtrees. Assert exact owned-tree equality after model conversion, exact ID-to-item association, and byte-for-byte asynchronous correlation preservation. These bounds constrain generated test inputs only; they do not add protocol acceptance limits. Disable external failure persistence; turn any minimized counterexample into a named fixed regression case before refactoring. Do not widen generic TTLV allocation policy for test convenience.
- **Green**: Add the smallest public model, validation, and conversion implementation to satisfy the focused tests. No generic byte encoder/decoder or network behavior is added.
- **Refactor**: Extract focused modules, document public contracts, preserve `#![forbid(unsafe_code)]`, and keep Red/Green behavior unchanged.
- **Verification**: Run `cargo fmt --all --check`, focused tests, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo doc --workspace --all-features --no-deps`, `cargo llvm-cov --workspace --all-features`, catalog validator/report checks, and source immutability checks when implementation is authorized.
- **Coverage**: Meet at least 95% line coverage for changed protocol/model code and protocol-model crate, 90% workspace overall, and all applicable repository gates; generated code may be excluded only with a documented reason.

## Complexity Tracking

No constitution violations or architectural exceptions are proposed.
