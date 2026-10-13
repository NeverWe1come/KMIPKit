# Implementation Plan: KMIP 2.1 Credential Sessions

**Branch**: feature/KMIPKIT-0038-credential-sessions | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: Approved feature specification in specs/024-credential-sessions/spec.md

## Summary

Implement typed client models and explicit execution for Delegated Login (§6.1.12, Tables 199–201), Login (§6.1.30, Tables 253–255), and Logout (§6.1.31, Tables 256–258). Reuse the common KMIP message, batch, result, TTLV, secret, and transport contracts. Preserve the returned Ticket structure and its Ticket Value bytes as secret data. Each call remains explicit, sends through the existing typed client once, and leaves any later ticket use to the caller.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.

**Primary Dependencies**: Existing KMIPKit workspace crates; no new runtime dependency expected.

**Storage**: N/A. No implicit session or ticket store is introduced.

**Testing**: Protocol model tests; source-derived TTLV vectors; malformed and negative input tests; deterministic client tests through the existing fake transport; manifest parity tests after the approved public API manifest is available.

**Target Platform**: Existing Linux, Windows, and macOS CI matrix; Rust core, with C, Java, and Python public surfaces generated or adapted under the canonical API manifest.

**Project Type**: KMIP 2.1 client library with a Rust protocol crate and cross-language bindings.

**Performance Goals**: No new latency or throughput target. An explicit operation call performs at most one request exchange and no local cryptographic computation.

**Constraints**: TTLV only; KMIP 2.1 client-to-server direction; existing typed batch and delivery-state semantics; preserve absent and unknown fields; redact ticket and body bytes; zeroize KMIPKit-owned ticket bytes; no automatic retry, Poll, session installation, or post-login credential substitution.

**Scale/Scope**: Three operations. Delegated Login and Login return the §7.39 Ticket structure. Logout submits that structure. Rights and Usage Limits are nested structures from the caller, while Lease Time preserves the table-approved Interval-or-Date-Time representation.

## Constitution Check

- Exact normative sources and table contracts are recorded in spec.md and traceability.md: PASS.
- The work uses the approved client-only TTLV, typed batch, TLS/HTTPS, and Rust-core boundaries: PASS.
- Secret handling reuses the existing SecretBytes lifecycle and redaction rules; no ticket value is logged or formatted: PASS.
- No local cryptography, implicit server policy, session state, retry, or implicit Poll is introduced: PASS.
- Red, Green, and Refactor evidence is required in distinct commits for each user story: PASS.
- Rust protocol/client work does not depend on the public API manifest feature. C/Java/Python parity and generated outputs must follow the approved canonical manifest in specification/api/public-api.json and docs/development/api-manifest.md: PASS, with a cross-feature sequencing gate.
- No pinned OASIS source or generated file is edited by hand: PASS.
- No unresolved implementation-blocking normative ambiguity identified in this feature scope: PASS.

## Design

### Dependencies and sequencing

The accepted common protocol, credential/secret, typed-batch, asynchronous-result, transport, and extension foundations are present in the release branch. The operation-specific Rust models can proceed without the API-manifest work.

The client integration points crates/kmipkit-protocol/src/lib.rs, crates/kmipkit-client/src/execute.rs, and crates/kmipkit-client/src/lib.rs are central files shared with other operation features. Keep their edits in the coordinated integration sequence; implement isolated protocol modules and tests first. Add C, Java, and Python operation surfaces only through the approved manifest/generator workflow. KMIPKIT-0043 owns the release-roadmap gate for complete manifest coverage and language parity before claiming 1.0.0 API completeness; its merge gates this feature binding slice, not standalone Rust models or typed Rust client.

### Protocol and client model

- Add a reusable operation Ticket model for the §7.39 Ticket structure. Ticket Type remains an open raw Enumeration; Ticket Value is owned secret bytes with zeroization and redacted Debug/error behavior.
- Add DelegatedLoginRequest/Response, LoginRequest/Response, and LogoutRequest/Response modules. Preserve every optional request field and all structurally valid response fields. Keep Rights and Usage Limits as generic nested TTLV structures where no existing typed model is approved.
- Represent Lease Time according to the approved operation-table contract as either Interval or Date Time; do not normalize or infer server interpretation.
- Add explicit ClientRequest variants and typed ClientResponseView accessors for all three operations. Preserve common Result Status, Result Reason, permitted Result Message, asynchronous correlation, and transport delivery evidence through the existing client boundary.
- Returned tickets remain ordinary caller-owned results. No automatic credential mutation, authentication reuse, local authorization, or ticket persistence is introduced. Logout is explicit and accepts the caller-provided Ticket.

### TDD and review gates

For each operation, make and retain a Red commit with focused failing model/fake-transport tests, a Green commit with the minimal implementation that passes those tests, and a separate Refactor commit with behavior unchanged. Each commit records the exact command and result. Never combine the three evidence stages into one commit.

### Project Structure

Documentation for this feature:

specs/024-credential-sessions/
├── plan.md
├── research.md
├── data-model.md
├── traceability.md
├── quickstart.md
├── contracts/
│   └── operation-api.md
└── tasks.md

Implementation and verification locations:

crates/kmipkit-protocol/src/{ticket.rs,delegated_login.rs,login.rs,logout.rs}
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/{ticket_model_tests.rs,delegated_login_operation_tests.rs,login_operation_tests.rs,logout_operation_tests.rs}
crates/kmipkit-client/src/{execute.rs,lib.rs}
crates/kmipkit-client/tests/unit/{delegated_login_execution_tests.rs,login_execution_tests.rs,logout_execution_tests.rs}
crates/kmipkit/src/lib.rs
specification/catalog/kmip-2.1.json
specification/api/public-api.json
crates/kmipkit-ffi/src/lib.rs
bindings/c/include/kmipkit.h (generated)
bindings/c/tests/credential_sessions.c
bindings/java/native/kmipkit_jni.cpp
bindings/java/src/main/java/org/kmipkit/CredentialSessionOperations.java
bindings/python/src/kmipkit/credential_sessions.py
docs/user-guide/en/credential-sessions.md
docs/user-guide/es/sesiones-credenciales.md

**Structure Decision**: Extend the existing Rust protocol and client crates. Keep operation payload models in individual files and use the current shared executor, fake transport, TTLV model, and secret types. Cross-language surfaces follow the source-of-truth manifest and its pinned generator; generated files are never edited by hand.
