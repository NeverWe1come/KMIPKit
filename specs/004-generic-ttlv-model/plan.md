# Implementation Plan: KMIP Generic TTLV Value Model

**Branch**: `feature/KMIPKIT-0004-generic-ttlv-model` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-generic-ttlv-model/spec.md`

## Summary

Define the lossless in-memory Rust model for all eleven KMIP 2.1 Item Types, preserving 24-bit tag identity, numeric bits, exact Big Integer Item Value octets, and caller-specified Structure order. A catalog-driven, deterministic Rust tag-allocation table will implement the proposed allocation gate. Payload values use a private zeroizing wrapper, redacted diagnostics, and closure-borrowed inspection. This feature does not encode or decode TTLV and cannot claim a model is wire-valid.

Implementation is blocked until (1) this specification is approved, (2) ADR-0010's tag precedence is accepted, and (3) the KMIPKIT-0003 implementation PR is merged into the release branch. This plan prepares design and task artifacts only; it does not authorize bypassing those gates.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94.
**Primary Dependencies**: Workspace crates; proposed `zeroize` 1.9.0 workspace dependency with default features disabled and only `alloc` enabled. No derive or serialization feature is planned. The implementation PR must still record the dependency rationale required by `docs/development/coding-standards.md` and lock the exact resolution.
**Storage**: In-memory only; no persistence.
**Testing**: `cargo test -p kmipkit-ttlv`, deterministic generator unit tests and `--check`, compile-time/compile-fail public API checks, workspace formatting/Clippy/tests, and `cargo llvm-cov --workspace --all-features` (or repository automation when available).
**Target Platform**: Rust workspace supported platforms; no platform-specific implementation.
**Project Type**: Rust library crate (`crates/kmipkit-ttlv`) with a deterministic repository generator.
**Performance Goals**: No throughput target is specified for this value-model feature. Tag allocation lookup must be bounded and allocation-free; do not add a benchmark gate without a separately approved criterion.
**Constraints**: No `unsafe` in `kmipkit-ttlv`; preserve unknown values and order; do not expose payloads in formatting, serialization, or model errors; zeroize KMIPKit-owned current payload storage before release; no OASIS source scraping or manual edits to generated files.
**Scale/Scope**: Eleven assigned Item Types; 24-bit Raw Tags; all catalogued tag entries and ranges; ordered Structures with a KMIPKit safety cap of 64 nested Structure levels. TTLV decoder limits and hostile wire input are out of scope for this model.

## Constitution Check

### Pre-design gate

| Principle | Result | Evidence / condition |
|---|---|---|
| I. Specification and traceability | Pass for design; implementation gated | spec.md gives stable FR/NR identifiers and exact OASIS clauses; implementation must add code/test links before completion. |
| II. Test first and evidence based conformance | Pass with mandatory TDD tasks | Tasks require distinct Red, Green, Refactor commits with DCO sign-off; model tests do not substitute for later OASIS codec conformance. |
| III. One core, explicit language boundaries | Pass | Rust-only model crate, which already forbids unsafe; FFI and bindings are excluded. |
| IV. Secure defaults and lossless protocol handling | Pass with security review | Unknown values and child order are preserved; payloads use zeroization and redacted diagnostics. A safe probe verifies the Drop path without inspecting freed memory. |
| V. Human governed, reviewable changes | Pass with hard gates | Dedicated feature worktree and branch; ADR/spec review and KMIPKIT-0003 merge precede implementation; only a human approves or merges the PR. |

No architectural boundary changes are proposed. The unresolved §11.56 overlap is made explicit as project policy in Proposed ADR-0010 and is a hard implementation gate, not an assumed OASIS interpretation.

### Post-design gate

Pass, subject to the same implementation gates. The generated allocation table is derived only from checked-in catalog records and ranges. Model errors remain local to `kmipkit-ttlv`; no dependency cycle is introduced. Serialization is not added. The memory-erasure contract is limited to current KMIPKit-owned buffers and explicitly excludes caller/runtime copies and prior allocations that KMIPKit no longer owns.

## Design and implementation phases

### Phase 0 — Research and scope resolution

- Confirm source clauses in the pinned KMIP 2.1 OASIS copy and catalog (see [research.md](research.md)).
- Record generator, tag precedence, secret storage, closure borrowing, and error-layer decisions with alternatives.
- Keep ADR-0010 Proposed until maintainer acceptance; do not represent its policy as OASIS clarification.

### Phase 1 — Model and contracts

- Define the value entities and invariants in [data-model.md](data-model.md).
- Define the Rust public contract in [contracts/rust-value-model.md](contracts/rust-value-model.md).
- Provide runnable, gated validation scenarios in [quickstart.md](quickstart.md).
- Keep the architecture's existing Generic TTLV description staged: this feature establishes the in-memory model; a separate codec feature must implement wire framing, lengths, padding, and limits before the full generic API description can be claimed.

### Phase 2 — Prerequisites and deterministic allocation data

- Before implementation begins, verify the three approval/dependency gates above against the release branch.
- Add a deterministic generator under `tools/normative_catalog/` that reads only `specification/catalog/kmip-2.1.json` and reuses `load_validated_catalog` from `validate.py`; select `elements` with `kind: "tag"` and consume `tag_ranges`.
- Generate private Rust allocation metadata (not public tag declarations or APIs) in `crates/kmipkit-ttlv/src/generated/`; sort records by numeric value and emit exact assignments/classifications and numeric range data. Keep runtime precedence lookup hand-written and reviewable in `tag.rs`.
- Provide `--write` and read-only `--check` modes using the catalog tooling's safe I/O conventions. Test malformed/duplicate/out-of-range input, precedence, boundaries, stable output, clean/stale/missing generated files, and idempotent writing. Add `--check` to `.github/workflows/ci.yml` beside catalog validation/report. Do not create the future cross-language public API manifest in this feature.

### Phase 3 — P1 User Story 1: typed in-memory values

- Use strict Red, Green, Refactor TDD to implement all eleven value representations, generic items, and ordered Structures on the checked-tag foundation from Phase 2.
- Preserve the exact signedness/width, Unicode text, byte sequence, and Big Integer Item Value octets; the in-memory model does not perform wire validation.
- Derive Item Type from the value variant. Keep error types local to `kmipkit-ttlv` and payload-free.

### Phase 4 — P1 User Story 2: lossless tags, values, and order

- Exhaustively verify catalog-backed tag allocation against every individual catalog entry and range boundary.
- Verify preservation of unknown Enumeration values, Integer bit patterns, repeated child tags, caller order, and the 64-level Structure bound without operation-specific schema validation.
- Add generated-catalog trace links from each applicable requirement to implementation and executable tests.

### Phase 5 — P1 User Story 3: secret handling and diagnostics

- Wrap payload-bearing values in a private boxed zeroizing secret type; implement recursive zeroization for nested Structures without unsafe code.
- Provide explicit closure-scoped borrowed views whose references cannot escape; document that caller copies and formatting are intentional disclosure paths outside KMIPKit zeroization.
- Redact `Debug`, any `Display`, and model-local errors. Add compile-time checks for lack of `serde::Serialize`, `Clone`, and `Copy`; safe test-only zeroization and payload-address stability probes cover every leaf variant and nested Structure.

### Phase 6 — Polish and feature verification

- Update `docs/architecture/public-api.md` to distinguish this in-memory model from the later wire codec.
- Complete English technical documentation and requirement traceability in the same implementation PR.
- Run focused tests, generator checks, workspace format/lint/test checks, and relevant dependency/license/security checks. Coverage must meet at least 95% for changed code and the `kmipkit-ttlv`/protocol-model crates, and 90% workspace-wide; apply the 85% transport/FFI threshold only if those paths are changed. Record the `cargo llvm-cov --workspace --all-features` (or repository automation) results and Red, Green, Refactor commits in the draft PR.
- A later codec feature owns OASIS wire vectors, malformed TTLV parsing, decoder limits, and wire round trips; they are not falsely claimed by this model feature.

## Project Structure

### Documentation (this feature)

```text
specs/004-generic-ttlv-model/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/rust-value-model.md
├── checklists/ttlv-model.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/kmipkit-ttlv/
├── src/lib.rs
├── src/tag.rs
├── src/value.rs
├── src/item.rs
├── src/structure.rs
├── src/error.rs
└── src/generated/tag_allocations.rs
tools/normative_catalog/
└── generate_ttlv_tags.py
specification/catalog/kmip-2.1.json
docs/architecture/public-api.md
```

Tests belong beside the Rust crate (`crates/kmipkit-ttlv/tests/`) and with the generator (`tools/normative_catalog/tests/`). Exact module boundaries may be adjusted during implementation without changing the public contract.

**Structure Decision**: Keep protocol-model code in the existing `kmipkit-ttlv` crate. Put deterministic catalog generation beside existing normative-catalog tooling. This feature adds neither a new crate nor a public API manifest.

## Complexity Tracking

No constitution violations or architecture exceptions are proposed.
