# Implementation Plan: KMIP TTLV Wire Codec

**Branch**: `feature/KMIPKIT-0005-ttlv-wire-codec` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification for a bounded TTLV encoder and decoder, following the generic model.

## Summary

Add canonical TTLV encoding and bounded decoding for the eleven KMIP 2.1 Item Types represented by the KMIPKIT-0004 generic tree. Apply OASIS big-endian headers, exact type-specific lengths, value widths, and padding. Preserve child order, unknown enum/mask bits, and accepted extension Tags. Check message, Structure depth, and Item-count limits before allocation. Keep transport and operation/schema handling outside the codec.

The 004 model implementation landed in `release/1.0.0` at `cf6c4c0d87c4de7dc159aba046a8fe5638ccc6bf`; implementation remains gated on approval of this feature and review/acceptance of proposed ADR-0011 for inbound Reserved Tags.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.
**Primary Dependencies**: `kmipkit-ttlv` generic model and its existing pinned `zeroize` 1.9.0 dependency; no new dependency is planned.
**Storage**: In-memory only; encoded output uses a dedicated zeroizing owner and decoded values own their payloads through the approved model.
**Testing**: Focused codec unit tests, exact OASIS-derived vectors, malformed-input negatives, property-based model round trips, coverage, workspace checks, and fuzz targets after the parser surface stabilizes.
**Target Platform**: Rust workspace supported platforms.
**Project Type**: Public Rust library crate/module in `crates/kmipkit-ttlv`.
**Performance Goals**: No throughput target is specified. Length and limit checks must be bounded and avoid body-sized allocation before validation.
**Constraints**: No unsafe code; no raw payload diagnostics; no automatic retries; all integer length arithmetic checked; no OASIS download/scraping; no hand-edited generated output. One item per call, no I/O or schema validation.
**Scale/Scope**: 11 Item Types; 16 MiB default message cap; 64 Structure-level cap; 100,000 Items by default. See `research.md` for exact limit counting and configuration semantics.

## Constitution Check

### Pre-design gate

| Principle | Result | Evidence / condition |
|---|---|---|
| I. Specification and traceability | Pass for design; completion gated | Stable FR/NR IDs and exact clauses are recorded. Implementation and executable verification links remain required. |
| II. Test first and evidence based conformance | Pass with mandatory TDD | Tasks require separate Red, Green, and Refactor commits, OASIS vectors, malformed inputs, and coverage evidence. |
| III. One core, explicit language boundaries | Pass | Rust-only codec over the common model; FFI and language adapters are out of scope. |
| IV. Secure defaults and lossless handling | Pass with policy gate | Input lengths and limits are checked before allocation; model values and order are preserved. ADR-0011 proposes rejection of Reserved Tags; `max_structure_depth` is caller-configurable from 0 to the model cap of 64. |
| V. Human governed, reviewable changes | Pass with hard gates | Dedicated worktree and branch. No code until this spec is approved and dependencies/policies are resolved. Only a human approves or merges the PR. |

### Post-design gate

No architecture boundary changes are proposed. Codec remains below protocol/message and transport layers. The parser operates on bounded slices and the encoder on model data. The design does not claim OASIS profile conformance or schema validity. Proposed ADR-0011 recommends rejection of received Reserved Tags and must be accepted before implementing that decoder branch.

## Project Structure

### Documentation

```text
specs/005-ttlv-wire-codec/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/rust-ttlv-codec.md
├── checklists/requirements.md
├── checklists/design.md
└── tasks.md
```

Related repository decision record: `docs/adr/0011-reserved-tag-decoding-policy.md`.

### Source and verification

```text
crates/kmipkit-ttlv/src/
├── lib.rs                  # declares the public codec module
└── codec/
    ├── mod.rs              # public facade, limits, errors, EncodedTtlv
    ├── encoder.rs          # bounded canonical writer
    └── decoder.rs          # checked slice parser and Structure decode
crates/kmipkit-ttlv/tests/
├── codec_vectors.rs        # per-type, exact OASIS wire vectors
├── codec_negative.rs      # malformed, unsupported, and limit cases
├── codec_roundtrip.rs     # generic model properties and canonicalization
└── codec_limits.rs        # message, depth, count, and U32 boundary cases
specification/catalog/kmip-2.1.json # requirement-to-spec/code/test references
```

**Structure Decision**: Keep codec implementation in the existing `kmipkit-ttlv` crate and keep generated catalog files generated from reviewed inputs. Final module boundaries may follow the actual merged 004 layout but must retain these responsibilities.

## Design Phases

### Phase 0 — Prerequisite confirmation and normative review

- Confirm the source digest and clauses in the pinned local OASIS copy.
- PR #14 is merged; verify the actual public model API and accepted ADR-0010 on the updated release base.
- Record `KMIPKIT-REQ-SPEC-10.1.2-001` as follow-on typed-protocol scope: the generic codec preserves supplied child order but cannot validate operation schemas. Keep the 1.0 traceability gate open until every applicable client 1.0 Structure has approved typed-spec ownership, implementation, and executable order-verification references.
- Obtain review/acceptance of proposed ADR-0011 resolving `KMIPKIT-DISC-037`; update the catalog decision reference before coding.

### Phase 1 — Contracts and data invariants

- Define per-call limits, error classes, exact one-item API, and canonical output in `contracts/rust-ttlv-codec.md`.
- Define Item Length/padding accounting for each Item Type in `data-model.md`.
- Keep operation field-order/schema checking and transport framing outside this crate.

### Phase 2 — Encoder (strict TDD)

- Write failing exact byte vectors for all eleven types and nested Structures, plus U32 maximum/one-over output-size planner boundaries without multi-gigabyte allocation.
- Validate the complete tree, compute bounded lengths, and reserve the complete zeroizing output before copying payload bytes; then emit canonical encoding with no fallible exits after payload copying begins. Return the result in `EncodedTtlv`, which exposes an immutable byte borrow and zeroizes its owned bytes and capacity on drop.
- Refactor the writer for one focused responsibility, document invariants, and verify checked arithmetic and error redaction.

### Phase 3 — Decoder (strict TDD)

- Write failing tests for valid OASIS vectors and all malformed boundaries before implementation.
- Parse one complete item with checked offsets, parent bounds, supported type lengths, UTF-8/Boolean checks, and resolved Tag policy.
- Refactor bounded Structure traversal and fallible allocation; reject trailing bytes and never expose raw input. Property tests compare against a canonicalized expected model: empty Big Integer values are excluded, unaligned Big Integer octets are minimally sign-extended to an eight-byte multiple, and already aligned Big Integer octets remain exact.

### Phase 4 — Limits and cross-cutting validation

- Add exact-boundary and one-over tests for byte, depth, and element limits.
- Add bounded property tests, normative catalog traceability, documentation, security review, platform CI, and coverage records.
- Run repository-required fmt, Clippy, workspace test, and coverage automation after the tests exist and the implementation is complete.

## Dependencies and Gates

- KMIPKIT-0004 implementation PR #14 and KMIPKIT-0003 core types/errors are merged into the release ancestry.
- ADR-0010 is Accepted in the updated release tree.
- Proposed ADR-0011 needs review/acceptance for received Reserved Tags; the branch remains gated for that path.
- Depth is configurable from 0 to the generic model's hard maximum of 64; exceeding 64 requires a separately reviewed model change.

## Risks and Mitigations

- **Length confusion across padding types**: Maintain an explicit per-type table in `data-model.md`; tests assert both the header length and complete encoded extent.
- **Allocation denial of service**: Check total input size and declared/cumulative lengths before value allocation; enforce item/depth counters incrementally.
- **Loss of wire padding bytes**: Specify semantic canonicalization and zero-fill output; do not promise byte identity for accepted noncanonical padding.
- **Reserved-tag policy**: Keep Reserved-tag decoding gated until proposed ADR-0011 is reviewed; never fold it into unknown extension preservation.
- **Model API integration**: Reconcile implementation with the merged 004 API and its accepted ADR-0010; do not code against guessed interfaces.

## Complexity Tracking

No constitution exception or new architecture layer is proposed. The reserved-tag policy remains a formal review gate; the empty Big Integer and U32 Item Length behaviors are explicit project constraints in this draft.
