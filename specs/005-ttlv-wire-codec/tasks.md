# Tasks: KMIP TTLV Wire Codec

**Input**: Design documents in `/specs/005-ttlv-wire-codec/`.

**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/rust-ttlv-codec.md`, and `quickstart.md`.

**Implementation gate**: Before T001, verify the feature specification has been approved, PR #14 or its successor has been merged into `release/1.0.0`, the branch is rebased on that release head, the 004 Tag/Item APIs are inspected, `KMIPKIT-DISC-037` has a reviewed decision, and the depth configurability contract is reconciled. If any gate is unmet, do not start implementation tasks; update this gate evidence and stop.

**Organization**: Strict Red, Green, Refactor commits with DCO sign-off. Each phase's Red commit records failing tests before production code changes.

## Phase 1: Prerequisite setup

- [ ] T001 Verify every implementation gate above, confirm the dedicated feature worktree is based on the active `release/1.0.0` head, inspect the landed 004 public API, and record exact gate evidence in `specs/005-ttlv-wire-codec/tasks.md`.

## Phase 2: User Story 1 — Encode all assigned TTLV Item Types (Priority: P1)

**Goal**: Encode one generic Item into canonical bytes with exact type-specific header, length, value, and padding behavior.

**Independent test**: `cargo test -p kmipkit-ttlv --test codec_vectors` compares all eleven type families and nested Structure encodings byte for byte.

**Requirements**: `KMIPKIT-0005-FR-001`–`FR-003`; `KMIPKIT-0005-NR-001`–`NR-005`; `KMIPKIT-0005-SC-001`.

- [ ] T002 [US1] Write and run failing exact OASIS-derived encoder vectors for all eleven Item Types, big-endian boundaries, child order, repeated tags, lengths, Big Integer sign extension, and each padding family in `crates/kmipkit-ttlv/tests/codec_vectors.rs` (Red commit).
- [ ] T003 [US1] Implement checked output-size calculation and bounded canonical encoding for all eleven Item Types in `crates/kmipkit-ttlv/src/encoder.rs` (Green commit).
- [ ] T004 [US1] Refactor the encoder into documented per-type length/value/padding helpers, preserve child order, and keep encode errors payload-free in `crates/kmipkit-ttlv/src/encoder.rs` and `crates/kmipkit-ttlv/src/codec.rs` (Refactor commit).

## Phase 3: User Story 2 — Decode a complete generic TTLV item (Priority: P1)

**Goal**: Decode valid supported TTLV without losing modeled values or Structure ordering, and reject malformed inputs without a partial public Item.

**Independent test**: `cargo test -p kmipkit-ttlv --test codec_negative --test codec_roundtrip` exercises OASIS vectors, malformed byte cases, and canonical re-encoding.

**Requirements**: `KMIPKIT-0005-FR-004`–`FR-006`, `FR-010`; `KMIPKIT-0005-NR-001`–`NR-006`; `KMIPKIT-0005-SC-002`–`SC-003`.

- [ ] T005 [US2] Write and run failing decoder vectors and negative cases for truncated headers/values, all fixed widths, invalid UTF-8/Boolean, unsupported Type, nested boundaries, trailing bytes, assigned/extension Tags, the resolved Reserved-tag behavior, and padding extents in `crates/kmipkit-ttlv/tests/codec_negative.rs` (Red commit).
- [ ] T006 [US2] Implement one-item checked parsing, type-specific lengths, value decoding, ordered Structure construction, and resolved Tag disposition in `crates/kmipkit-ttlv/src/decoder.rs` (Green commit).
- [ ] T007 [US2] Refactor offset/boundary handling and errors into focused helpers; verify no input or payload is retained/formatted and no unsafe code is added in `crates/kmipkit-ttlv/src/decoder.rs` and `crates/kmipkit-ttlv/src/codec.rs` (Refactor commit).
- [ ] T008 [US2] Add bounded property tests that construct generic values, encode/decode them, and compare represented values plus child order in `crates/kmipkit-ttlv/tests/codec_roundtrip.rs`.

## Phase 4: User Story 3 — Enforce configurable resource limits (Priority: P1)

**Goal**: Bound parser/encoder work and allocation by per-call byte, nesting, and element limits.

**Independent test**: `cargo test -p kmipkit-ttlv --test codec_limits` exercises exact defaults, caller configuration, and one-over boundaries.

**Requirements**: `KMIPKIT-0005-FR-007`–`FR-009`; `KMIPKIT-0005-SC-004`.

- [ ] T009 [US3] Write and run failing tests for 16 MiB, 64 Structure levels, 100,000 Items, lowered depth limits, raised/lowered message and item-count limits, arithmetic overflow, and pre-allocation rejection in `crates/kmipkit-ttlv/tests/codec_limits.rs` (Red commit).
- [ ] T010 [US3] Implement immutable `CodecLimits`, checked constructors, per-call counters, preflight checks, and fallible reservations in `crates/kmipkit-ttlv/src/codec.rs`, `crates/kmipkit-ttlv/src/decoder.rs`, and `crates/kmipkit-ttlv/src/encoder.rs` (Green commit).
- [ ] T011 [US3] Refactor limit accounting to shared documented invariants and verify every rejection occurs before size-driven allocation in `crates/kmipkit-ttlv/src/codec.rs` and `crates/kmipkit-ttlv/src/decoder.rs` (Refactor commit).

## Phase 5: Traceability, documentation, and hardening

- [ ] T012 Update only applicable `requirements[]` rows in `specification/catalog/kmip-2.1.json` with `feature_spec`, implementation, and verification references for `KMIPKIT-0005-NR-*` and mapped OASIS requirement IDs; regenerate every affected artifact with its pinned repository generator.
- [ ] T013 Turn `specs/005-ttlv-wire-codec/quickstart.md` scenarios into an executable example and tested documentation in the final `kmipkit-ttlv` public API docs after the merged API is stable.
- [ ] T014 Add reviewed OASIS vectors and malformed-input fixtures under `crates/kmipkit-ttlv/tests/fixtures/` with exact source document, section, and requirement ID attribution; add a bounded decoder fuzz target in `fuzz/fuzz_targets/ttlv_decode.rs` and its package wiring in `fuzz/Cargo.toml`.
- [ ] T015 Run repository automation for format, Clippy, focused/workspace tests, property tests, coverage, generator `--check`, dependency/license/security scans, and supported-platform CI; record actual results and confirm at least 95% coverage for changed codec/model code and workspace gates in the draft PR.
- [ ] T016 Obtain independent QA and security reviews of the final code/test/doc diff, fix findings, reconcile `spec.md`, catalog traceability, and generated output, then prepare a terminal-created draft PR to `release/1.0.0` with Red/Green/Refactor commit IDs and verification evidence.

## Dependencies and execution order

- T001 is a hard gate and must complete before any code or test task.
- Encoder, decoder, and limit implementation share the same crate and public contract; one implementer should execute the phases sequentially.
- Each Red task precedes its Green task, and each Green task precedes its Refactor task. T008 depends on the decoder and encoder.
- Traceability and examples depend on stable public APIs; full CI/coverage and independent reviews follow all implementation changes.
- No task may implement a Reserved-tag decoder policy until `KMIPKIT-DISC-037` is resolved and this spec is updated.

## Parallel opportunities

No production implementation task is marked parallel. The codec phases share length, model, error, and limit contracts. Independent review can start after the complete diff is available without editing implementation files.

## Implementation strategy

First unblock the feature at T001. Then implement encoder, decoder, and limits in sequential Red/Green/Refactor commits. Finish with properties, traceability, executable documentation, generated artifacts, full CI/coverage, and independent QA/security review. Open the implementation PR only after all gates pass; never merge it or claim release readiness from this feature alone.

## Traceability map

| Requirement | Planned task(s) | Executable verification artifact |
|---|---|---|
| `KMIPKIT-0005-FR-001`–`FR-003`; `NR-001`–`NR-005` | T002–T004 | `crates/kmipkit-ttlv/tests/codec_vectors.rs` |
| `KMIPKIT-0005-FR-004`–`FR-006`; `NR-001`–`NR-006` | T005–T008 | `crates/kmipkit-ttlv/tests/codec_negative.rs`, `codec_roundtrip.rs` |
| `KMIPKIT-0005-FR-007`–`FR-008` | T009–T011 | `crates/kmipkit-ttlv/tests/codec_limits.rs` |
| `KMIPKIT-0005-FR-009` | T007, T015 | `crates/kmipkit-ttlv/tests/codec_negative.rs` and redaction checks in `codec_vectors.rs` |
| `KMIPKIT-0005-FR-010`; `NR-006` | T001, T005, T006, T012 | Tag-policy tests in `codec_negative.rs` after `KMIPKIT-DISC-037` is decided |
| `KMIPKIT-0005-FR-011`; `KMIPKIT-0005-SC-005` | T012, T014–T016 | normative catalog `implementation_refs`/`verification_refs`, CI, and coverage report |
| `KMIPKIT-0005-SC-001` | T002 | `codec_vectors.rs` exact-byte cases for all eleven Item Types |
| `KMIPKIT-0005-SC-002` | T008 | `crates/kmipkit-ttlv/tests/codec_roundtrip.rs` |
| `KMIPKIT-0005-SC-003` | T005, T007 | `codec_negative.rs` and error-redaction assertions |
| `KMIPKIT-0005-SC-004` | T009–T011 | `crates/kmipkit-ttlv/tests/codec_limits.rs` |
