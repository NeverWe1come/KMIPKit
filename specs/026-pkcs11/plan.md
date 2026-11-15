# Implementation Plan: KMIP 2.1 PKCS#11 Forwarding

**Branch**: feature/KMIPKIT-0040-pkcs11 | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: specs/026-pkcs11/spec.md

## Summary

Add the typed PKCS#11 operation (§6.1.37, Tables 273–275) to the existing Rust protocol/client pipeline. Preserve nested function, interface, parameter, return-code structures, and correlation bytes without implementing a PKCS#11 codec or local cryptographic provider. Reuse shared TTLV, batch, result, delivery-state, asynchronous, redaction, and transport contracts.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94
**Primary Dependencies**: Existing KMIPKit workspace crates; no new dependency expected
**Storage**: N/A
**Testing**: Rust protocol tests, fake-transport client tests, source-derived TTLV vectors, catalog validation/report checks
**Target Platform**: Existing Linux, Windows, and macOS CI matrix
**Project Type**: Rust client library with protocol crate
**Performance Goals**: No new target; each explicit call performs at most one exchange and no local PKCS#11 operation
**Constraints**: KMIP 2.1 TTLV; preserve unknown values and generic structures; no retry, implicit Poll, or profile claim; redact opaque parameters/correlation
**Scale/Scope**: One operation, Tables 273–275, Rust core plus four-language parity

## Constitution Check

- Approved client-only KMIP 2.1 TTLV scope and exact source tables: PASS.
- Reuse typed messages, client execution, TLS/HTTPS, result, delivery-state, and secret-storage contracts: PASS.
- No local PKCS#11 codec, algorithm provider, server behavior, or transport is added: PASS.
- Preserve unknown/vendor values and absent optional fields: PASS.
- Require distinct Red, Green, and Refactor evidence commits: PASS.
- Four-language parity follows the approved public API manifest workflow: PASS.
- Pinned OASIS sources and generated output remain unmodified by hand: PASS.

## Dependencies and sequencing

The active release contains the typed TTLV, message/batch, client execution, transports, result, and redaction foundations. This feature has no dependency on 022–025 or 027–028. Rust protocol and client work may begin independently.

Only manifest mapping, generated signatures, adapters, and parity tests depend on the approved KMIPKIT-0043 API contract being integrated. Do not block Rust work on 0043. Coordinate edits to the shared crates/kmipkit-client/src/execute.rs dispatcher with the feature integrator.

## Design

Add crates/kmipkit-protocol/src/pkcs11.rs and public exports. Model Tables 273–274 exactly. Preserve PKCS#11 Function's catalog-defined Structure and Enumeration-or-Integer choice. Interface, input/output parameters, and return-code structures remain generic, lossless TTLV; do not decode profile-specific PKCS#11 contents. Preserve optional absence; do not synthesize the V3.0 interface. Preserve supplied correlation bytes exactly.

Admit the request to the closed Rust request/operation/response path, operation code selection, payload conversion, response association, and redacted formatting. Add an explicit client method following existing one-exchange conventions. Use the common result on Failure/Pending; do not fabricate success fields or perform retries/Poll.

Map every Table 273/274 field to its stable catalog element/tag and test. §6.1.37 has no separate client normative-clause ID or direct catalog test case. Do not assign server duties or create synthetic requirements. Regenerate the coverage report only through the pinned tool.

Tests cover required fields, type and order, absent versus empty, Enum/Integer, unknown nested fields, malformed responses, Table 275 result preservation, one exchange, Pending/correlation/delivery, no retry, and redaction. Fixtures are source-derived; no official conformance or PKCS#11 Profile claim is allowed.

## Project Structure

Documentation: specs/026-pkcs11/{plan.md,research.md,data-model.md,quickstart.md,contracts/rust-client.md,traceability.md,tasks.md}

Implementation:
- crates/kmipkit-protocol/src/pkcs11.rs; crates/kmipkit-protocol/src/lib.rs
- crates/kmipkit-protocol/tests/unit/pkcs11_operation_tests.rs
- crates/kmipkit-client/src/execute.rs; crates/kmipkit-client/src/lib.rs
- crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs
- specification/catalog/kmip-2.1.json and generated report only if required
- binding adapters only after 0043 supplies canonical mapping

**Structure Decision**: Follow the one-module-per-operation pattern. Keep PKCS#11 profile structures generic and coordinate the shared dispatcher.

## Risks and mitigations

Opaque structures may hold secrets: use shared redaction/zeroizing handling and sentinel tests. Vendor-defined functions: retain both allowed raw types. Dispatcher conflicts: integrate centrally. No linked official fixture: report local source-derived evidence only. No constitution or architecture boundary change is proposed.
