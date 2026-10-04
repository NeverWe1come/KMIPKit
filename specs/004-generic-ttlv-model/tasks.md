# Tasks: KMIP Generic TTLV Value Model

**Input**: Design documents from `/specs/004-generic-ttlv-model/`

**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/rust-value-model.md`, `quickstart.md`.

**Implementation gate**: The feature specification and ADR-0010 were accepted in PR #10 (`3638c6c7929992e8ced59a3903a0a6640f847069`); the KMIPKIT-0003 implementation was merged in PR #13 (`b52df30648312f8c7703f511afe80a412cda66cd`). T001 records the active release base and dedicated worktree evidence.

**Organization**: Tasks are grouped by user story. Red, Green, and Refactor evidence must be separate development commits. Every commit must include the DCO sign-off required by `CONTRIBUTING.md` and `GOVERNANCE.md`.

## Phase 1: Setup

**Purpose**: Reconfirm approved inputs and develop the deterministic catalog generator test-first.

- [x] T001 Confirm approved spec, accepted ADR-0010, merged KMIPKIT-0003 implementation, active release base, and dedicated feature worktree; record gate evidence here and carry it into the draft PR description.
- [x] T002 [P] Write and run focused generator tests for assigned/reserved exact entries, aggregate ranges, malformed/duplicate/out-of-range catalog data, stable numeric ordering, and `--check`/`--write` behavior in `tools/normative_catalog/tests/test_generate_ttlv_tags.py` (Red commit `e25f65c`; read-only assertion fix `11047cb`; reviewed).
- [x] T003 Implement `tools/normative_catalog/generate_ttlv_tags.py` using `load_validated_catalog` from `tools/normative_catalog/validate.py`, safe I/O helpers, private numeric tag/range metadata, deterministic `--write`, and read-only `--check` modes (Green `6b7c9cd`; redaction fix `fe5be3f`; reviewed).
- [x] T004 Refactor the generator to reuse existing validation and atomic-write conventions, document its local input/output, and confirm idempotent deterministic output in `tools/normative_catalog/generate_ttlv_tags.py` (Refactor `602ac97`; reviewed).

## Phase 2: Foundational — checked tags and generated allocation data

**Purpose**: Make allocation-checked Tags available before any public Item can be constructed.

- [x] T005 Generate private tag allocation metadata from `specification/catalog/kmip-2.1.json` into `crates/kmipkit-ttlv/src/generated/tag_allocations.rs`; include every individual assigned/Reserved tag and all numeric ranges, and verify `--check` detects stale output. (Verified: 374 exact records: 354 Assigned, 20 Reserved; five ranges; `--check` passed; repeat `--write` was byte-identical; 15 focused generator tests passed.)
- [x] T006 Write and run failing Rust tests for `RawTag` width boundaries and `RawTag::try_checked(&self)`, including exact assignments `0x420173–0x420176`, individual Reserved entries, residual Reserved boundaries, the complete Extensions-range endpoints, and rejected unused values in `crates/kmipkit-ttlv/tests/tag_allocation.rs` (Red commits `37dd4e0`, `dcf158f`; catalog-alignment correction `ce24309`; native `cargo test -p kmipkit-ttlv --test tag_allocation` fails only on the intentionally absent `RawTag`/`Tag` API; both receiver and catalog corrections independently reviewed clean).
- [x] T007 Implement `RawTag`, allocation-checked `Tag`, exact-entry-first lookup, and payload-free tag errors using `crates/kmipkit-ttlv/src/generated/tag_allocations.rs` in `crates/kmipkit-ttlv/src/tag.rs` and `crates/kmipkit-ttlv/src/error.rs` (Green commit `ca21b4b`; Rust 1.94 focused tests 5/5; reviewed clean; workspace formatting has a known T006 test diff that T008 must close).
- [x] T008 Refactor tag parsing and lookup into documented modules, wire the generated module into `crates/kmipkit-ttlv/src/lib.rs`, and retain `#![forbid(unsafe_code)]` (Refactor commit `5df8e2a`; Rust 1.94 format, crate tests 5/5, and Clippy pass; review approved).
- [x] T009 Add `python3 -B tools/normative_catalog/generate_ttlv_tags.py --repo-root . --check` beside catalog validation/report in `.github/workflows/ci.yml` (commit `d9aea17`; generator freshness check and 158 catalog tests passed with 7 platform-dependent skips; reviewed clean).

## Phase 3: User Story 1 — Construct and inspect all KMIP value types (Priority: P1)

**Goal**: Construct every KMIP 2.1 Item Type and inspect its exact in-memory value through a borrowed closure view.

**Independent test**: `cargo test -p kmipkit-ttlv` verifies all eleven variants, exact numeric widths/sign, Unicode/text and byte preservation, exact Big Integer Item Value octets, Item Type derivation, child order, and required construction errors.

- [ ] T010 [US1] Write and run failing tests for all eleven Item Types, public constructors, Item Type derivation, numeric boundaries, Unicode, bytes, exact Big Integer octets, ordered/repeated Structure children, and payload-free construction errors in `crates/kmipkit-ttlv/tests/value_model.rs` (Red commit).
- [ ] T011 [US1] Implement opaque `Value` constructors, `Item`, ordered `Structure`, borrowed `ValueView` exposure, and local `ModelError` in `crates/kmipkit-ttlv/src/value.rs`, `crates/kmipkit-ttlv/src/item.rs`, `crates/kmipkit-ttlv/src/structure.rs`, and `crates/kmipkit-ttlv/src/error.rs` (Green commit).
- [ ] T012 [US1] Refactor the public modules and rustdoc to match `contracts/rust-value-model.md`; verify `ItemType` is derived from the private value representation and no unsafe code is introduced (Refactor commit).

## Phase 4: User Story 2 — Preserve tags, unknown values, and Structure bounds (Priority: P1)

**Goal**: Demonstrate complete forward-compatible preservation and safe bounded nesting against the reviewed catalog.

**Independent test**: Table-driven Rust tests cover every exact tag record, all tag range boundaries, all 354 assigned and 20 Reserved tag values, unknown Enumeration values, high-bit masks, repeated tags, ordered nesting, and depths 1–64 versus rejected depth 65.

- [ ] T013 [US2] Write and run exhaustive catalog-backed tests for every exact assigned/Reserved tag, the overlapping exact-entry/range cases, all range boundaries, unknown Enumeration values, Integer bit patterns, duplicate child tags, caller order, and Structure depth 64/65 in `crates/kmipkit-ttlv/tests/tag_allocation.rs` and `crates/kmipkit-ttlv/tests/value_model.rs` (Red commit for any uncovered behavior).
- [ ] T014 [US2] Implement any preservation gaps exposed by T013, including the 64-level Structure construction bound and failure-without-mutation behavior, in `crates/kmipkit-ttlv/src/structure.rs`, `crates/kmipkit-ttlv/src/value.rs`, and `crates/kmipkit-ttlv/src/error.rs` (Green commit).
- [ ] T015 [US2] Refactor allocation and nesting errors to remain value-free; document that the nesting limit is KMIPKit policy and not an OASIS protocol constraint in `crates/kmipkit-ttlv/src/structure.rs` and `crates/kmipkit-ttlv/src/tag.rs` (Refactor commit).

## Phase 5: User Story 3 — Protect payloads and redact diagnostics (Priority: P1)

**Goal**: Zeroize current model-owned payload storage and make payload disclosure explicit.

**Independent test**: `trybuild` UI fixtures with checked-in diagnostic snapshots reject escaped borrowed references, `serde::Serialize`, `Clone`, and `Copy` on `Value`, `ValueView`, `StructureView`, `Item`, and `Structure` with the intended trait/lifetime errors; runtime tests permit intentional copies; a safe spy verifies wrapper Drop invocation; live-object unit tests verify each payload variant and nested Structure is recursively zeroized; an address-stability check confirms Structure growth does not move payload boxes; sentinel checks cover direct and nested formatting and errors.

- [ ] T016 [US3] Add `trybuild` UI fixtures and expected diagnostics for missing `serde::Serialize`/`Clone`/`Copy` bounds on `Value`/`ValueView`/`StructureView`/`Item`/`Structure` and for closure-borrow escape; write failing sentinel, intentional-copy, safe `Zeroize` Drop spy, live-object recursive-zeroize unit tests, and payload-address stability tests in `crates/kmipkit-ttlv/tests/public_api.rs`, `crates/kmipkit-ttlv/tests/ui/`, `crates/kmipkit-ttlv/tests/redaction.rs`, `crates/kmipkit-ttlv/tests/zeroization.rs`, and `crates/kmipkit-ttlv/src/value.rs` (Red commit; run the focused Rust tests and `cargo test -p kmipkit-ttlv --test public_api --test redaction --test zeroization`).
- [ ] T017 [US3] Add reviewed `zeroize` 1.9.0 workspace configuration plus test-only `trybuild` 1.0.121 and `serde` dependencies; implement private `Secret<T>` owning `Box<T>` whose Drop calls `Zeroize` on the boxed payload, recursive safe `Zeroize`, immutable-after-ownership String/Vec payloads, closure-scoped borrowed views, and redacted model errors in `Cargo.toml`, `crates/kmipkit-ttlv/Cargo.toml`, `crates/kmipkit-ttlv/src/value.rs`, and `crates/kmipkit-ttlv/src/error.rs` (Green commit).
- [ ] T018 [US3] Refactor all payload-bearing `Debug`/optional `Display` implementations to remain redacted and ensure no public payload serializer or payload-bearing log path exists in `crates/kmipkit-ttlv/src/` (Refactor commit).
- [ ] T019 [US3] Document the exact locked `zeroize` guarantee, current String/Vec backing-capacity behavior, and caller/runtime/prior-allocation limits in `docs/architecture/public-api.md` and Rustdoc in `crates/kmipkit-ttlv/src/value.rs`.

## Phase 6: Polish and cross-cutting verification

- [ ] T020 [P] Update generic TTLV architecture text to distinguish the in-memory model from wire framing, lengths, padding, decoder limits, and schema validation in `docs/architecture/public-api.md`.
- [ ] T021 Run generator checks, Rust tests, formatting, Clippy, workspace tests, `cargo llvm-cov --workspace --all-features` (or repository automation when available), and applicable dependency/license/security checks for `tools/normative_catalog/`, `crates/kmipkit-ttlv/`, and `.github/workflows/ci.yml`; verify at least 95% coverage for changed code and the `kmipkit-ttlv`/protocol-model crates, at least 90% workspace-wide, and at least 85% for transport/FFI only if affected; record actual commands/results and Red/Green/Refactor commit IDs in the draft PR description.
- [ ] T022 Reconcile every `KMIPKIT-0004-FR-*` and `KMIPKIT-0004-NR-*` row in `specs/004-generic-ttlv-model/spec.md` with implementation and test paths. In `specification/catalog/kmip-2.1.json`, update `feature_spec`, `implementation_refs`, and `verification_refs` only on the `requirements[]` records whose `source_clause_ids` map to NR-001 through NR-007, plus the matching `elements[]` records with `kind: "tag"`; confirm generated tag output remains current.
- [ ] T023 Prepare a draft PR from `feature/KMIPKIT-0004-generic-ttlv-model-implementation` to the active `release/1.0.0` branch with scope, rationale, verification evidence, limitations, and dependency/security notes; request independent QA and security review before human review.

### T001 gate evidence

- Approved specification and accepted ADR-0010: PR #10 merged into `release/1.0.0` at `3638c6c7929992e8ced59a3903a0a6640f847069`.
- KMIPKIT-0003 implementation dependency: PR #13 merged into `release/1.0.0` at `b52df30648312f8c7703f511afe80a412cda66cd`.
- Active release base at implementation branch creation: `b52df30648312f8c7703f511afe80a412cda66cd` (`origin/release/1.0.0`).
- Dedicated worktree: `C:\Users\ramp1953\.codex\worktrees\kmipkit-0004-ttlv-spec\KMIPKit`; branch: `feature/KMIPKIT-0004-generic-ttlv-model-implementation`.
- The final draft PR description will repeat these gate SHAs and add the task verification evidence.

## Dependencies and execution order

### Phase dependencies

- Phase 1 is blocked by the implementation gate stated above.
- Phase 2 depends on the generator's Red/Green/Refactor sequence and completes checked `Tag` construction before public `Item` construction; it blocks the user stories.
- User Stories 1, 2, and 3 are all P1 and proceed sequentially because they share `Value`, `Item`, `Structure`, and `Tag` contracts.
- Polish depends on all three stories.

### User story dependencies

- **US1**: Can start after Phase 2. Defines public value constructors, Items, and Structures.
- **US2**: Depends on the US1 model and completes exhaustive preservation and bounded-depth conformance evidence.
- **US3**: Depends on stable US1/US2 storage and public access surfaces.

### Parallel opportunities

- Generator tests (T002) can proceed independently from normative source review before the tag gate implementation.
- Redaction and compile-fail tests (T016) may be prepared in parallel files after US1 stabilizes its API, but all must run and be failing before the US3 Green commit.
- Architecture documentation (T020) can proceed separately from traceability reconciliation (T022) after behavior stabilizes.

## Implementation strategy

Deliver checked tag allocation first so every public Item has a valid Tag. Then implement and test the typed value model (US1), exhaustive preservation and bounded nesting (US2), and zeroization/redaction (US3). Keep Red, Green, and Refactor in separate commits. Do not claim TTLV wire validity. Do not open the implementation PR until all implementation gates and required checks are satisfied; the design PR must be reviewed before those gates can be met.
