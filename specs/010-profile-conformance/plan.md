# Implementation Plan: KMIP 2.1 Client Profile Conformance

**Branch**: `feature/KMIPKIT-0010-profile-conformance` | **Date**: 2026-10-07 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/010-profile-conformance/spec.md`

## Summary

Implement explicit client profile validation and a deterministic profile evidence/readiness report from the reviewed KMIP 2.1 catalog. Record Baseline Client as the initial 1.0 target without making a claim. Keep conditional profiles gated, preserve all profile requirements and official test evidence links, and fail closed while the 16 Baseline mandatory fixtures remain unavailable.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94; Python 3.12+ standard-library tooling for offline catalog generation/reporting.

**Primary Dependencies**: Existing workspace crates and `tools/normative_catalog`; no new runtime dependency is planned.

**Storage**: Checked-in `specification/catalog/kmip-2.1.json` remains the source of normative profile facts; project-owned `specification/catalog/profile-targets.json` records the selected target independently of profile lifecycle state. Generated Rust profile metadata and the generated compliance report are committed for review.

**Testing**: Rust unit and fake-transport integration tests; Python standard-library generator/report tests; catalog traceability and deterministic-regeneration checks; official OASIS tests only when their pinned fixtures are available.

**Target Platform**: Rust workspace platforms Linux, Windows, and macOS. Catalog generation and report checks run offline in CI.

**Project Type**: Multi-crate Rust client library with C/Java/Python bindings planned in later API parity specifications.

**Performance Goals**: Profile lookup and validation perform bounded work proportional to selected profile requirements and dependency edges; no network discovery or source parsing occurs at runtime.

**Constraints**: KMIP 2.1 only; TTLV only; TLS 1.3 mutual TLS; raw TTLV/TLS and HTTP/1.1 over HTTPS. Keep server-initiated operations, JSON/XML, server profiles, and certification outside this feature. Do not reject extra valid operations based solely on a profile's minimum operation list. Missing official fixtures prevent claims.

Profile target input is fixed to `specification/catalog/profile-targets.json`, read through safe repository I/O with a 64 KiB bound and strict JSON/schema validation for generation/reporting only. Runtime selection uses explicit caller-selected profiles, independent of target membership. The source catalog is loaded through safe repository I/O and `load_validated_catalog`. Generated Rust literals use a dedicated Rust-compatible encoder. Readiness recomputes current evidence and fails closed on stale or contradictory catalog lifecycle state.

**Scale/Scope**: 18 catalogued client profiles, 15 currently `client_1_0` or `conditional`; Baseline Client is the initial target, but a profile is runtime-selectable only when every applicable requirement has a typed rule mapping and executable test. Other profiles without complete mappings remain visible candidates. All 16 Baseline mandatory fixture records currently report unavailable.

## Constitution Check

- Specification and traceability: PASS. Exact OASIS citations and catalog IDs are in `spec.md`; implementation tasks must map every selected profile requirement to code and tests.
- Test-first conformance: PASS. Tasks require distinct Red, Green, and Refactor commits, negative cases, fake transport checks, and official tests only for pinned available fixtures.
- One Rust core and explicit language boundaries: PASS. Profile semantics live in Rust protocol/client core; bindings consume parity contracts later.
- Secure, lossless defaults: PASS. No secret data in errors/reports, no hidden requests/retries, and cryptographic choices remain explicit.
- Human-governed changes: PASS. This draft is not approved by being authored; the feature branch must receive review before implementation/merge as required by the repository workflow.
- Release claims: PASS. This feature never publishes a claim; the missing Baseline fixtures keep release eligibility blocked.

## Project Structure

### Documentation (this feature)

```text
specs/010-profile-conformance/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── profile-validation.md
│   └── profile-evidence.md
├── checklists/
│   ├── requirements.md
│   ├── profile-coverage.md
│   └── claim-readiness.md
└── tasks.md
```

### Source Code

```text
crates/kmipkit-protocol/src/
├── profile.rs
└── profiles_generated.rs
crates/kmipkit-protocol/tests/
├── profile_validation.rs
└── profile_evidence.rs
crates/kmipkit-client/tests/unit/
└── profile_preflight_tests.rs
tools/normative_catalog/
├── generate_profiles.py
└── tests/
    ├── test_generate_profiles.py
    ├── test_validate.py                 # normative-inclusion closure validation
    └── fixtures/profile-closure-v2.1.json # independent pinned-source test oracle
scripts/
├── ci_summary.py                   # creates and consumes only ephemeral same-run evidence
└── tests/test_ci_summary.py
specification/catalog/
├── kmip-2.1.json
├── profile-targets.json
└── coverage-report.md          # regenerated, never hand edited
docs/compliance/
└── conformance.md              # updated if behavior/report contract changes
```

**Structure Decision**: Keep profile validation beside shared KMIP protocol rules, generate immutable Rust profile records from the checked-in catalog, and extend the existing offline normative report path. Operation-specific generated public bindings remain the responsibility of the later API manifest/parity work.

Runtime validation is an explicit per-exchange opt-in through `Client::execute_with_profiles(request, selection)`; the existing `execute` path remains unchanged and no production constructor or hidden client-wide selection state is introduced. Generated profile records must carry the full recursively normative clause/subclause closure and every applicable requirement ID. Validation errors expose stable codes and source IDs while preserving request delivery state, never free-form causes.

Readiness may treat test results as verified only when a same-run aggregate job in `.github/workflows/ci.yml` creates a strict manifest during a `push` to `refs/heads/release/1.0.0` and GitHub reports `ref_protected: true`. A PR workflow can change its own workflow or manifest producer, so pull-request runs cannot certify evidence; scheduled runs also remain ineligible. Every eligible run must query the live branch-protection API with a short-lived read-only Administration GitHub App token and verify that review is required, stale approvals are dismissed, the `bypass_pull_request_allowances` object is present with empty `users`, `teams`, and `apps` arrays, admins are subject to the policy, and force pushes/deletions are disabled. Missing or malformed bypass policy, any bypass actor, a missing token, API error, or insufficient policy blocks readiness. The token is not available to PR/schedule jobs or logged. The manifest is not downloaded from or replaced by user-supplied artifacts. This manifest and the resulting readiness evaluation are ephemeral; the committed coverage report and catalog contain no run-specific data and remain deterministic from checked-in inputs. Local reports without this context remain evidence-incomplete.

## Design and execution phases

1. **Phase 0 - Research and invariants**: lock catalog fields, transitive normative clause closure, dependency/claim states, minimum-capability semantics, current fixture blockers, per-call selection, and current-CI evidence provenance. No unresolved interpretation is silently selected.
2. **Phase 1 - Data and contracts**: define complete generated profile clause/requirement closure (including whole-source normative inclusions), explicit per-call runtime selection, stable source-backed validation errors, a strict same-run protected-release-push verification manifest, readiness results, deterministic report fields, and exact source traceability.
3. **Phase 2 - Red tests**: add failing tests for inherited clause/subclause closure, per-call selection, selection conditions/dependencies, typed-rule completeness, no allowlist behavior, default preservation, no network side effects on invalid requests, missing-fixture blockers, Query/Discover non-inference, report determinism, and stale/forged/mismatched CI evidence.
4. **Phase 3 - Green implementation**: generate Rust records from the catalog; implement explicit per-call profile selection, requirement/capability checks, redacted source-backed errors, strict protected-release-push same-run evidence-manifest generation/validation, and readiness aggregation.
5. **Phase 4 - Refactor and generated outputs**: centralize repeated profile metadata/rules; regenerate report and Rust outputs; verify byte-identical generation and workspace lint.
6. **Phase 5 - Conformance and review**: execute every available pinned profile fixture, mark absent official fixtures as blocked, verify every selected profile's full normative clause closure and catalog traceability, then run independent QA/security review and CI. Do not claim Baseline conformance while required fixtures are unavailable.

## Complexity Tracking

No constitution violations or new architecture boundary changes are proposed.
