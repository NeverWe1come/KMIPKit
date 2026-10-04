# Implementation Plan: KMIP TTLV Wire Codec

**Branch**: `feature/KMIPKIT-0005-ttlv-wire-codec` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification for a bounded TTLV encoder and decoder, following the generic model.

## Summary

Add canonical TTLV encoding and bounded decoding for the eleven KMIP 2.1 Item Types represented by the KMIPKIT-0004 generic tree. Apply OASIS big-endian headers, exact type-specific lengths, value widths, and padding. Preserve child order, unknown enum/mask bits, and accepted extension Tags. Check message, Structure depth, and Item-count limits before allocation. Keep transport and operation/schema handling outside the codec.

The design is ready for review, but implementation is blocked until the 004 model implementation lands in `release/1.0.0`, the Reserved-tag receipt discrepancy is resolved, and the depth-configurability boundary is reconciled.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.
**Primary Dependencies**: `kmipkit-ttlv` generic model; standard library only unless implementation evidence justifies another dependency.
**Storage**: In-memory only; output uses an owned byte vector and decoded values own their payloads through the approved model.
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
| IV. Secure defaults and lossless handling | Pass with open gates | Input lengths and limits are checked before allocation; model values and order are preserved. Reserved-tag receive policy and depth configurability need resolution. |
| V. Human governed, reviewable changes | Pass with hard gates | Dedicated worktree and branch. No code until this spec is approved and dependencies/policies are resolved. Only a human approves or merges the PR. |

### Post-design gate

No architecture boundary changes are proposed. Codec remains below protocol/message and transport layers. The parser operates on bounded slices and the encoder on model data. The design does not claim OASIS profile conformance or schema validity. The two open policy gates are recorded in the spec, checklist, and ADR/dependency notes; they must be resolved before implementation starts.

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

### Source and verification after the 004 implementation lands

```text
crates/kmipkit-ttlv/src/
├── codec.rs                # public facade, shared codec options/errors
├── encoder.rs              # bounded canonical writer
└── decoder.rs              # checked slice parser and Structure decode
crates/kmipkit-ttlv/tests/
├── codec_vectors.rs        # per-type, exact OASIS wire vectors
├── codec_negative.rs      # malformed, unsupported, and limit cases
└── codec_roundtrip.rs     # generic model properties and canonicalization
specification/catalog/kmip-2.1.json # requirement-to-spec/code/test references
```

**Structure Decision**: Keep codec implementation in the existing `kmipkit-ttlv` crate and keep generated catalog files generated from reviewed inputs. Final module boundaries may follow the actual merged 004 layout but must retain these responsibilities.

## Design Phases

### Phase 0 — Prerequisite confirmation and normative review

- Confirm the source digest and clauses in the pinned local OASIS copy.
- Verify PR #14 or successor has landed and inspect the actual public model API.
- Resolve `KMIPKIT-DISC-037` and the depth-limit configurability boundary; update this spec and design artifacts before coding.

### Phase 1 — Contracts and data invariants

- Define per-call limits, error classes, exact one-item API, and canonical output in `contracts/rust-ttlv-codec.md`.
- Define Item Length/padding accounting for each Item Type in `data-model.md`.
- Keep operation field-order/schema checking and transport framing outside this crate.

### Phase 2 — Encoder (strict TDD)

- Write failing exact byte vectors for all eleven types and nested Structures.
- Implement bounded length calculation and canonical encoding, including checked big-endian headers and padding.
- Refactor the writer for one focused responsibility, document invariants, and verify checked arithmetic and error redaction.

### Phase 3 — Decoder (strict TDD)

- Write failing tests for valid OASIS vectors and all malformed boundaries before implementation.
- Parse one complete item with checked offsets, parent bounds, supported type lengths, UTF-8/Boolean checks, and resolved Tag policy.
- Refactor bounded Structure traversal and fallible allocation; reject trailing bytes and never expose raw input.

### Phase 4 — Limits and cross-cutting validation

- Add exact-boundary and one-over tests for byte, depth, and element limits.
- Add bounded property tests, normative catalog traceability, documentation, security review, platform CI, and coverage records.
- Run repository-required fmt, Clippy, workspace test, and coverage automation after the tests exist and the implementation is complete.

## Dependencies and Gates

- KMIPKIT-0004 implementation PR #14 (currently an open draft at planning time) must be merged into `release/1.0.0`; rebase this branch then.
- KMIPKIT-0003 core types/errors is already merged in the release ancestry.
- ADR-0010 and the 004 model gate need their actual status rechecked on the release branch; the local release copy currently labels ADR-0010 Proposed.
- `KMIPKIT-DISC-037` needs a separate reviewed resolution for received Reserved Tags.
- Maximum decoder Structure depth must agree with the model's 64-level construction cap and the configurable-limits security rule.

## Risks and Mitigations

- **Length confusion across padding types**: Maintain an explicit per-type table in `data-model.md`; tests assert both the header length and complete encoded extent.
- **Allocation denial of service**: Check total input size and declared/cumulative lengths before value allocation; enforce item/depth counters incrementally.
- **Loss of wire padding bytes**: Specify semantic canonicalization and zero-fill output; do not promise byte identity for accepted noncanonical padding.
- **Unresolved Tag policy**: Keep Reserved-tag decode out of implementation until `KMIPKIT-DISC-037` is decided; never fold it into unknown extension preservation.
- **Dependency on unmerged model API**: Do not code against guessed interfaces; inspect the merged 004 API after the release base changes.

## Complexity Tracking

No constitution exception or new architecture layer is proposed. Resolving the two policy gates may require a narrowly scoped spec/ADR amendment before implementation.
