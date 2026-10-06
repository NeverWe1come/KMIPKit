# Implementation Plan: KMIP 2.1 Credentials and Attestation

**Branch**: `feature/KMIPKIT-0008-credentials-readiness` | **Date**: 2026-10-05 | **Spec**: [spec.md](spec.md)

## Summary

Define typed, lossless Rust models and structural validation for the seven KMIP 2.1 Credential Type variants, Authentication and Nonce values, and truthful Attestation Capable Indicator behavior. Implementation is gated on T004's exact-spec independent review and delegated maintainer authorization, dependency verification, and the reviewed secret ownership contract. OD-001 remains recorded as an unresolved source-classification question under `KMIPKIT-DISC-041`; its accepted catalog disposition excludes a client duty and does not block these models. OD-002 sets the Device minimum-presence rule; OD-003 limits Timestamp-monotonicity claims; OD-005 gates only future Authentication selection integration. OD-004 is resolved by scope and OD-006 by the informative catalog classification. This feature stays in-memory for credential data and adds no credential send path; it reuses the existing writer only for the non-secret capability indicator.

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
| Specification and traceability | All normative rows cite exact OASIS sections/tables and stable catalog IDs; each will map to planned implementation and tests. Preserve the accepted catalog disposition and open `KMIPKIT-DISC-041`; do not claim the disputed lowercase §9.4 clause as a client duty. |
| Test-first conformance | Separate Red, Green, Refactor commits; derived table-driven positives and negatives; no official-vector claim without linked fixture evidence. |
| One Rust core | Implement credential models in `kmipkit-protocol` with `#![forbid(unsafe_code)]`; update only the existing `kmipkit-client` header builder for the non-secret indicator. FFI and language parity are deferred to their owning API/binding specification. |
| Secure and lossless | Redacted secret types, approved zeroization, unknown enum/subtree preservation; this feature adds no production path that sends Authentication or Credential values. |
| Human governed | The maintainer's explicit delegated authorization permits autonomous specification progression after independent QA. Agents still do not approve or merge PRs, publish releases, or update the release branch directly. |

**Pre-design gate**: Constitution alignment passes. **Implementation gate**: Passes after T004 records the exact active-release base, independent review of the final specification/checklist, and the maintainer's already delegated authorization. OD-001 remains open only for the disputed source classification; the accepted catalog assigns no client implementation or verification. OD-002 resolves the minimum Device field presence; OD-003 preserves the timestamp limitation. OD-005 applies only to future Authentication selection. None of these decisions expands this feature beyond the bounded credential models and non-secret header indicator. OD-004 is resolved by scope; OD-006 is resolved because `KMIPKIT-CLAUSE-SPEC-9.11-008` is informative and has no requirement ID. Keep this feature's OD-006 distinct from KMIPKIT-0007 OD-006, which concerns future secret-bearing request lifecycle-test ownership. Verify exact 0005/0006/0007 dependency revisions and approvals. This feature never adds a credential send path.

## Research summary

See [research.md](research.md). Open decisions concern the disputed source classification (OD-001), Timestamp monotonicity policy (OD-003), and future Authentication selection boundary (OD-005). OD-001's catalog correction is already present in the active release; this feature preserves its disposition and makes no client conformance claim. OD-002 resolves the minimum Device field presence from §9.11/Table 412. OD-003 limits only the cross-request timestamp claim. OD-004 is resolved by scope; secret-memory ownership still requires review before implementation. Authentication execution integration needs a separate 0007 handoff, while standalone in-memory models do not require an execution API.

## Proposed architecture

Production code is limited to credential models in `crates/kmipkit-protocol` and one focused change to the existing private Request Header builder in `crates/kmipkit-client/src/execute.rs`. Add focused credential modules for the Authentication envelope, common Credential discriminator/value, per-variant schemas, and Nonce. Reuse `kmipkit-ttlv` owned structures and validation patterns. Use the existing workspace `zeroize` pin for KMIPKit-owned secret wrappers, after implementation review confirms the ownership and drop contract. Keep code safe Rust and preserve generic ordered subtrees for unknown values and fields. The header change reuses the existing permit/writer path and adds no Credential payload.

Typed constructors validate source-established required fields and resolved project policy. Conversion from a generic tree retains unknown values and ordered children. Authentication remains a standalone in-memory value in this feature: there is no Client/ClientBatch Authentication selection or credential-send callsite. The existing `Client::execute` builder emits Attestation Capable Indicator=True because the released credential model can construct the structure. Do not add a writer, permit, or second request/transport path. A generated public API manifest or cross-language binding is not changed here; public API and bindings work follows the complete protocol inventory.

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
│   └── message/header.rs               # read-only inbound indicator view remains unchanged
└── tests/
    ├── credential_contract.rs
    ├── credential_roundtrip.rs
    ├── credential_redaction.rs
    └── attestation_indicator.rs
crates/kmipkit-client/src/execute.rs        # indicator in the existing request-header builder
crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs # fake-transport header capture
specification/compliance/requirements/KMIPKIT-0008.csv
specs/008-credentials-attestation/
```

**Structure decision**: Focused credential models live under `kmipkit-protocol`, following accepted protocol-model patterns. The Attestation Capable Indicator is added to the already existing `Client::execute` message builder in `kmipkit-client`; the protocol header view remains read-only. Exact source paths and integration points were verified against the accepted 0006/0007 tree. This does not add Authentication selection to `Client` or `ClientBatch`.

## Implementation sequence

### Phase 0 — review gates

1. Record OD-001, OD-003, and OD-005 with their limited safe treatment; resolve OD-002 as at least one present Table 412 member without a text-length rule; record OD-004 as resolved by scope and OD-006 as resolved by the catalog's informative classification. KMIPKIT-0002 T045–T047 already corrected the OD-001 catalog records and recorded source review and independent QA. Preserve open DISC-041 and the fact that no explicit catalog-owner sign-off is named; do not edit catalog inputs or generated output here. Keep this feature's OD-006 distinct from KMIPKIT-0007 OD-006, which covers future secret-send lifecycle-test ownership.
2. Confirm the exact accepted KMIPKIT-0005 codec/secret-memory lifecycle contract, KMIPKIT-0006 Authentication header model, and KMIPKIT-0007 execution scope. OASIS §§8.1–8.3 (Tables 394–396) define one Request Header per Request Message and repeated Request Batch Items, with optional Authentication in the header; its message-wide scope is a structural implication. KMIPKIT-0007 does not define KMIPKit defaults, replacement, omission, or precedence; these remain OD-005 for a future execution-integration feature, and KMIPKIT-0008 adds no such path. Rebase this feature branch on the active release after prerequisites are present.
3. An independent reviewer checks the exact final requirements checklist and spec revision. Record the maintainer's explicit delegated authorization for autonomous specification approval; do not describe that authorization as personal line review or as a qualified human security audit. No code starts until these gates are evidenced.

### Phase 1 — Red tests

Add focused failing tests for each variant's source-defined fields/types/order, Authentication absent/non-empty/repeated behavior, unknown enums and opaque subtrees, Nonce byte preservation, Attestation Capability emission through a deterministic fake-transport request capture, secret redaction, and model lifecycle. Represent and preserve every Device field; reject no-field Device values and test presence independently of text content. Require and preserve caller Timestamp and hashed bytes and expose the effective SHA-256 default, but do not add hash calculation or monotonicity checks/tests before OD-003 review. The request-capture test must verify Authentication and Credential payload remain absent and the existing permit/writer boundary is unchanged.

Before adding any new test dependency, add `dependency-review.md` with license, maintenance/security history, MSRV, platforms, transitive footprint, and alternatives; get independent review. Prefer existing dependencies if they meet fixed-seed, bounded generation requirements.

### Phase 2 — Green implementation

Implement the smallest safe public model, typed conversions, structural validation, unknown value preservation, and redacted secret wrappers needed to pass Red tests. Update only the existing `build_request_message` path to emit the capability indicator. Do not add a writer, permit, credential-send route, or Client/ClientBatch Authentication-selection API. Keep request/batch Authentication defaults, replacement, omission, and precedence for a future execution-integration feature after its handoff is accepted. Maintain separate commits for each story's Green increment.

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

- Catalog scope errors can falsely turn server behavior into client conformance; preserve the accepted server-only/unassigned record and open `KMIPKIT-DISC-041`, and do not test/enforce “all Credentials satisfied” as a client duty.
- The caller is responsible for supplying a unique one or combination of the four §9.11 Device identifiers. The source does not define comparison scope, and client-local data cannot verify it. The separate minimum-presence interpretation requires at least one Table 412 member and imposes no text-length rule.
- Timestamp monotonicity needs an explicit owner, comparison scope, and clock behavior; do not calculate hashes or claim a monotonicity check/test until OD-003 review.
- Redaction and zeroization do not make secret transmission permissible. KMIPKIT-0008 only changes the existing request header's non-secret capability indicator; it adds no Authentication or Credential payload. Any later secret-bearing send path must satisfy its own independently reviewed approval and owner-through-transport lifecycle test evidence.
- The open source test-case fixtures are unavailable; tests remain derived conformance tests.

## Complexity tracking

No constitution exception or additional architectural layer is proposed. Any required secret writer exception is an existing separately gated proposal and is not established by this plan.
