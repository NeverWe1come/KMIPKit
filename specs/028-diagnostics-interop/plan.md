# Implementation Plan: KMIP 2.1 Interop and Diagnostics

**Branch**: feature/KMIPKIT-0042-diagnostics-interop | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: specs/028-diagnostics-interop/spec.md

## Summary

Add typed client models and explicit client calls for Interop (§6.1.26, Tables 241–243), Log (§6.1.29, Tables 250–252), and Validate (§6.1.57, Tables 340–342). Reuse existing TTLV, message/batch, one-exchange client, result, delivery, redaction, and transport foundations. Do not implement a server, perform local certificate validation, or claim interoperability.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94
**Primary Dependencies**: Existing workspace crates; no new dependencies expected
**Storage**: N/A
**Testing**: Protocol table tests, fake-transport client tests, source-derived vectors, catalog validation/report checks
**Target Platform**: Linux, Windows, macOS
**Project Type**: Rust client library with typed protocol crate
**Performance Goals**: No new target; each explicit call performs at most one exchange
**Constraints**: KMIP 2.1 TTLV; preserve unknown values and optional/repeated fields; redact Log message and generic payloads; no retry/Poll; no local certificate-chain policy
**Scale/Scope**: Three client operations, their request/response/error tables, Rust core and four-language parity

## Constitution Check

- Approved client-initiated KMIP 2.1 TTLV scope and exact OASIS sections/tables: PASS.
- Reuse current client, batch, Pending, result, delivery, transport, generic-TTLV, and redaction contracts: PASS.
- No local certificate validation, trust-anchor policy, server behavior, or server operation implementation is added: PASS.
- Interop server-only production restriction is not misapplied to the client library: PASS.
- KMIPKIT-DISC-017 assigns the mislabeled Query error prose in §6.1.29 to Log; use approved Table 252.
- Six Validate client requirement IDs are explicitly traced and verified.
- Distinct Red, Green, Refactor evidence commits are required per story/layer.
- 0043 gates only public cross-language manifest/adapters/parity; Rust work remains independent.

## Dependencies and sequencing

The active release supplies the typed TTLV, message/batch, client execution, TLS/HTTPS, result, delivery, and redaction foundations. The three protocol modules and test files are independent of one another and of specs 022–027. Each requires an addition to shared crates/kmipkit-client/src/execute.rs; coordinate that dispatcher work centrally and sequence operation integrations to avoid conflicting edits.

Rust protocol/client work can start before KMIPKIT-0043. Only public API manifest mapping, generated signatures, handwritten C/Java/Python adapters, and parity tests wait for 0043's approved mapping to be integrated.

## Design

### Protocol models

- Add interop.rs, log.rs, and validate.rs protocol modules with public exports.
- Interop: required Interop Function Enumeration and Interop Identifier text; successful operation payload is empty. Keep future enum values lossless. The normative prohibition on an Interop operation being available on a production server applies to servers; KMIPKit is a client and does not implement or assert server availability.
- Log: required Log Message text; successful operation payload is empty. Honor KMIPKIT-DISC-017 / approved spec: caller must not submit secrets. Regardless, redact the entire message from logs, Debug, Display, and errors. Use Table 252 for errors.
- Validate: accept one certificate chain represented by the repeated Certificate and/or Unique Identifier fields, optionally with Validity Date. Preserve list/wire order and mixed representation. Do not perform local chain construction, date evaluation, or trust-anchor/policy validation. If Validity Date is omitted, leave it absent; the server assumes current time as specified. Successful response requires and exposes the open Validity Indicator enumeration.
- Preserve common Success/Failure/Pending, result reason/message, unknown values, delivery evidence, generic response tree, and no-retry/no-Poll behavior.

### Catalog, traceability, and tests

Map operation IDs, field/tag IDs, all six Validate requirement IDs, source tables, implementation paths, and named tests in traceability.md. Keep error result reasons server-produced and preserve them through common result parsing. No linked direct official Test Case exists for these operations; local vectors are source-derived only.

### Test strategy

Each operation receives protocol positive/negative table tests, wrong-operation/type and missing-required checks, optional/repeated-field preservation, unknown value round trips, redaction, and fake-transport tests proving one exchange, correct operation code, result/Pending/delivery preservation, and no retry. Validate tests cover all six stable requirement IDs, including both input sources, repeats, optional date, date omission, and one-chain boundary without imposing an unapproved local chain policy. Log tests use secret sentinels. No server/profile/interoperability claim is inferred.

## Project Structure

Documentation: specs/028-diagnostics-interop/{plan.md,research.md,data-model.md,quickstart.md,contracts/rust-client.md,traceability.md,tasks.md}

Implementation:
- crates/kmipkit-protocol/src/{interop.rs,log.rs,validate.rs,lib.rs}
- crates/kmipkit-protocol/tests/unit/{interop_operation_tests.rs,log_operation_tests.rs,validate_operation_tests.rs}
- crates/kmipkit-client/src/{execute.rs,lib.rs}
- crates/kmipkit-client/tests/unit/{interop_execution_tests.rs,log_execution_tests.rs,validate_execution_tests.rs}
- specification/catalog/kmip-2.1.json and generated report only when traceability changes require them
- manifest/adapters only after KMIPKIT-0043

**Structure Decision**: Keep operation models and protocol tests in separate modules/files. Treat execute.rs as the shared integration hotspot and coordinate its changes centrally.

## Risks and mitigations

Log text and Validate certificates may contain sensitive data: always redact generic payload diagnostics and use existing secret handling. Do not enforce a client-side certificate validation policy. Keep the Interop production-server restriction server-only. Apply approved DISC-017 error-table mapping. No constitution or architecture boundary change is proposed.
