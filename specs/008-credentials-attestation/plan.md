# Implementation Plan: KMIP 2.1 Credentials and Attestation

**Branch**: `feature/KMIPKIT-0008-credentials-attestation` | **Date**: 2026-10-05 | **Spec**: [spec.md](spec.md)

## Summary

Define typed, lossless Rust models and structural validation for the seven KMIP 2.1 Credential Type variants, Authentication and Nonce values, and truthful Attestation Capable Indicator behavior. Implementation must be gated on exact feature approval, dependencies, OD-001 catalog-owner review, OD-002 Device validation scope, OD-003 timestamp policy, the OD-005 execution handoff, and the approved secret lifecycle contract. OD-004 is resolved by scope and OD-006 by the informative catalog classification. This feature remains permanently in-memory for its scope and makes no production credential send path available.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94.<br>
**Primary Dependencies**: `kmipkit-protocol`, `kmipkit-ttlv`, workspace-pinned `zeroize` 1.9.0 with alloc feature for KMIPKit-owned secret memory. No new production dependency is planned.<br>
**Storage**: None. No credential persistence or timestamp-state store.<br>
**Testing**: Rust unit and external contract tests; deterministic bounded property tests; generic TTLV conversion tests; redaction/lifecycle checks.<br>
**Target Platform**: Existing Rust workspace Linux, Windows, and macOS CI matrix.<br>
**Project Type**: Rust protocol library feature.<br>
**Performance Goals**: Validation is linear in credential count and child count; no independent latency target is specified. Respect the existing message, depth, and element limits before allocation.<br>
**Constraints**: KMIP 2.1/TTLV; no local cryptographic algorithms; unknown value preservation; no unsafe code; no secret in diagnostics; client-only; no server/profile claims.<br>
**Scale/Scope**: Seven named Credential Type values plus unknown values; one or more ordered Credential values per present Authentication; no new transport or binding implementation.

## Constitution Check

| Principle | Plan compliance |
|---|---|
| Specification and traceability | All normative rows cite exact OASIS sections/tables and stable catalog IDs; each will map to planned implementation and tests. Catalog misclassifications remain review blockers. |
| Test-first conformance | Separate Red, Green, Refactor commits; derived table-driven positives and negatives; no official-vector claim without linked fixture evidence. |
| One Rust core | Implement only in `kmipkit-protocol` with `#![forbid(unsafe_code)]`; FFI and language parity are deferred to their owning API/binding specification. |
| Secure and lossless | Redacted secret types, approved zeroization, unknown enum/subtree preservation; this feature adds no production credential send path. |
| Human governed | Draft stays Draft; reviewer checklists and approval gates remain unchecked; no self-approval, merge, release, or direct release-branch update. |

**Pre-design gate**: Constitution alignment passes. **Implementation gate**: Not passed. OD-001, OD-002, OD-003, and OD-005 remain open. OD-004 is resolved by this feature's permanent in-memory scope; OD-006 is resolved because `KMIPKIT-CLAUSE-SPEC-9.11-008` is cataloged as informative and has no requirement ID. Keep this feature's OD-006 distinct from KMIPKIT-0007 OD-006, which concerns future secret-bearing request lifecycle-test ownership. Verify exact 0005/0006/0007 dependency revisions and approvals. Independent review and human approval must apply to the final specification revision and updated checklist after the separate catalog and interface workflows. This feature never adds a production credential send path.

## Research summary

See [research.md](research.md). Open decisions concern catalog-owner review (OD-001), Device empty/minimum-field validation (OD-002), Timestamp monotonicity policy (OD-003), and the execution boundary (OD-005). OD-004 is resolved by scope; secret-memory lifecycle remains a separate dependency gate. Authentication execution integration also needs an explicit 0007 handoff, while standalone in-memory models do not require an execution API. These are gates, not implementation-time discretion.

## Proposed architecture

The only production-code layer in scope is `crates/kmipkit-protocol`. Add focused credential modules for the Authentication envelope, common Credential discriminator/value, per-variant schemas, Nonce, and the header indicator. Reuse `kmipkit-ttlv` owned structures and validation patterns. Use the existing workspace `zeroize` pin for KMIPKit-owned secret wrappers, after implementation review confirms the ownership and drop contract. Keep code safe Rust and preserve generic ordered subtrees for unknown values and fields.

Typed constructors validate source-established required fields and resolved project policy. Conversion from a generic tree retains unknown values and ordered children. The execution layer obtains Authentication only through the accepted 0007 request contract; this feature must not introduce a second request/transport path. A generated public API manifest or cross-language binding is not changed here; public API and bindings work follows the complete protocol inventory.

## Project structure

```text
crates/kmipkit-protocol/
├── src/
│   ├── credential/
│   │   ├── mod.rs
│   │   ├── authentication.rs
│   │   ├── credential.rs
│   │   ├── variants.rs
│   │   ├── conversion.rs
│   │   ├── hashed_password.rs
│   │   ├── attestation.rs
│   │   ├── nonce.rs
│   │   ├── secret.rs
│   │   └── validation.rs
│   └── message/header.rs               # indicator integration only if 0006 API accepts it
└── tests/
    ├── credential_contract.rs
    ├── credential_roundtrip.rs
    ├── credential_redaction.rs
    └── attestation_indicator.rs
specification/compliance/requirements/KMIPKIT-0008.csv
specs/008-credentials-attestation/
```

**Structure decision**: A focused module under the current `kmipkit-protocol` crate, which currently contains only result/error modules. Exact source paths and integration points will be checked against the accepted 0006/0007 tree before coding; any mismatch returns to this plan and requires review.

## Implementation sequence

### Phase 0 — review gates

1. Record OD-001, OD-002, OD-003, and OD-005 as open with their limited safe treatment; record OD-004 as resolved by scope and OD-006 as resolved by the catalog's informative classification. Track the exact OD-001 catalog corrections and owner-review evidence in a separate catalog workflow item; do not edit catalog inputs or generated output in this feature task. Keep this feature's OD-006 distinct from KMIPKIT-0007 OD-006, which covers future secret-send lifecycle-test ownership.
2. Confirm exact accepted KMIPKIT-0005 codec/secret-memory lifecycle contract, KMIPKIT-0006 Authentication header model, and KMIPKIT-0007 execution handoff. The 0007 handoff must settle inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the whole batch; it governs execution integration only. Rebase the feature branch on active release after prerequisites are present.
3. An independent reviewer checks the requirements checklist. Human approval of the exact spec revision is required under repository governance. No code starts until these gates are evidenced.

### Phase 1 — Red tests

Add focused failing tests for each variant's source-defined fields/types/order, Authentication absent/non-empty/repeated behavior, unknown enums and opaque subtrees, Nonce byte preservation, Attestation Capability derivation, secret redaction, and model lifecycle. Represent and preserve every Device field; do not test empty/minimum-field validation before OD-002 review. Require and preserve caller Timestamp and hashed bytes and expose the effective SHA-256 default, but do not add hash calculation or monotonicity checks/tests before OD-003 review. All tests in this feature remain in-memory; this feature never adds a production credential send path.

Before adding any new test dependency, add `dependency-review.md` with license, maintenance/security history, MSRV, platforms, transitive footprint, and alternatives; get independent review. Prefer existing dependencies if they meet fixed-seed, bounded generation requirements.

### Phase 2 — Green implementation

Implement the smallest safe public model, typed conversions, structural validation, unknown value preservation, and redacted secret wrappers needed to pass Red tests. Do not add network/encoder callsites. Complete Authentication integration only at the 0007-owned request boundary after the handoff is accepted. Maintain separate commits for each story's Green increment.

### Phase 3 — Refactor and docs

Refactor shared schema validation without changing Red/Green outcomes; document every public Rust type; add executable examples/tests; add the normative traceability CSV; update affected architecture/API and English/Spanish user documentation when user-visible behavior is accepted. Do not claim Java/Python zeroization of copies.

### Phase 4 — verification and review

Run focused tests, format, Clippy, workspace tests, docs, catalog validation/report regeneration, immutable-source checks, and coverage. Target at least 95% for changed code/protocol model, at least 95% protocol model crate, and at least 90% workspace as required by the repository. Perform sequential independent QA and security reviews; resolve findings; run supported CI; open a draft PR through terminal and verify it on GitHub.

## Verification strategy

- **Red evidence**: focused test commands fail for the intended missing API before implementation, captured in distinct commits.
- **Green evidence**: focused tests pass; round-trip, requiredness, and safety behavior demonstrated.
- **Refactor evidence**: same focused tests and new workspace gates pass with no behavior regression.
- **Normative evidence**: every applicable requirement has 100% mapping to exact source ID/clause, code, and test; every exclusion or unresolved item is explicit.
- **Security evidence**: sentinel redaction covers all credential variants and error paths; owned-memory zeroization test follows reviewed semantics; no real secret fixture appears in repository; no automatic retry; confirm this feature adds no production credential send path.
- **Interoperability**: no server-level authentication success claim in this feature. External implementation evidence belongs to the release-wide 1.0 interoperability gate.

## Risks and limits

- Catalog scope errors can falsely turn server behavior into client conformance; OD-001 remains open for the separate catalog-owner workflow, and the client must not test/enforce “all Credentials satisfied.”
- Device identity uniqueness is not locally verifiable, and the minimum-field set remains open; represent and preserve all fields while gating only empty/minimum-field validation.
- Timestamp monotonicity needs an explicit owner, comparison scope, and clock behavior; do not calculate hashes or claim a monotonicity check/test until OD-003 review.
- Redaction and zeroization do not make secret transmission permissible. KMIPKIT-0008 adds no production send path; any later client feature must satisfy its own independently reviewed approval and owner-through-transport lifecycle test evidence.
- The open source test-case fixtures are unavailable; tests remain derived conformance tests.

## Complexity tracking

No constitution exception or additional architectural layer is proposed. Any required secret writer exception is an existing separately gated proposal and is not established by this plan.
