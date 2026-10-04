# Implementation Plan: KMIP Shared Error Contract

**Branch**: feature/KMIPKIT-0003-core-types-errors | **Date**: 2026-10-04 | **Spec**: specs/003-core-types-errors/spec.md

**Input**: Bounded foundation outcome from the approved KMIPKit roadmap.

## Summary

Add lossless KMIP Result Status and Result Reason values, optional Result Message representation, a value-level status/reason invariant, categorized client failures, request delivery certainty, and redacted default error reporting. Place protocol values in kmipkit-protocol, transport delivery information in kmipkit-transport, composite client errors in kmipkit-client, and supported Rust re-exports in kmipkit. No wire parsing or network transport is part of this feature.

Implementation is gated on the normative inventory and catalog artifact from KMIPKIT-0002 being merged into release/1.0.0. The current feature branch starts from ce34179, which does not contain that open PR. Do not duplicate or hand-code generated Result Status and Result Reason assignments.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94.
**Primary Dependencies**: Existing workspace crates only; no new runtime or development dependencies.
**Storage**: In-memory values only.
**Testing**: External Rust integration tests per crate, fake transport tests, doctests, and requirement-linked result vectors.
**Target Platform**: Existing Linux x86_64/aarch64, Windows x86_64 MSVC, and macOS x86_64/aarch64 support.
**Project Type**: Synchronous multi-crate Rust client library.
**Performance Goals**: Value wrapping and error classification add no protocol-scale work; avoid retaining or formatting arbitrary cause text.
**Constraints**: Preserve unknown 32-bit enumeration values, do not log server text or sources by default, does not define or initiate retries; it preserves the client-wide no-retry boundary, unsafe forbidden, no panic-based production flow, no new dependency.
**Scale/Scope**: Shared error and result contracts only; no TTLV codec, network I/O, FFI, or language adapters.

## Constitution Check

- Specification and traceability: PASS. Each in-scope normative requirement has a stable KMIPKIT-0003-NR identifier and exact OASIS section.
- TDD and evidence: PASS when implementation starts. Tests must fail in a distinct Red commit, pass in Green, and remain passing through Refactor.
- One core and explicit boundaries: PASS. No duplicate protocol implementation or FFI surface is introduced.
- Security and losslessness: PASS. Unknown numeric values remain available; default formatting excludes server-provided text, cause text, and sensitive input; no retries are added.
- Human governance: the maintainer's direct instruction to execute the roadmap autonomously preauthorizes this bounded specification; no additional approval prompt is required. Repository rules still reserve PR approval and merge to a human. Open a terminal-created draft PR for review.
- Dependency gate: HOLD implementation until KMIPKIT-0002's catalog is merged into the active release branch and available for generated status/reason definitions.

## Design

### Layer responsibilities

- kmipkit-protocol owns result status/reason value wrappers, optional untrusted Result Message, value-level result validation, and safe protocol-processing errors.
- kmipkit-transport owns the three request delivery states and attaches the strongest known state to transport failures.
- kmipkit-client combines protocol, validation, transport, and server-result failures while preserving only safe cause categories and requiring exactly one delivery state for each local failure.
- kmipkit re-exports the supported result and client error surface.
- No changes to kmipkit-ffi, Java, or Python in this feature.

Known wire values must come from checked-in generated catalog output after KMIPKIT-0002 is merged. Open numeric wrappers retain any unknown value. Unknown statuses do not acquire inferred semantics. Only Success and Failure have the Result Reason presence rules stated in OASIS §9.18; this feature does not infer extra rules for pending, undone, or unknown status values.

Default Display text identifies a safe error category and omits source text, Result Message, request bytes, and secret-bearing fields. Arbitrary source text and payloads are discarded before retention; the public source chain exposes only safe cause-category wrappers. KMIPKit-generated logs use safe categories and delivery states only.

### Required tests and evidence

- Protocol unit and external integration tests cover every generated known status/reason value, unknown values, optional/empty message, exact Result Message text, ResultMessage/KmipOperationResult redaction, and Success/Failure reason invariants. Drop probes prove original unsafe error sources are destroyed during sanitization.
- Transport tests cover not sent, possibly sent, and response started; tests assert classification at pre-send, write-started, zero-byte read, and first-response-byte boundaries. Drop probes prove original unsafe error sources are destroyed during sanitization. Retry policy is not implemented here.
- Client tests distinguish local/protocol/transport failures from complete server operation results; verify server Result Message redaction in the complete-result client error; and verify safe cause categories survive while unsafe source text is unreachable through the public source chain.
- Secret sentinel tests cover credentials, private keys, secret key material, OTPs, tickets, server Result Message, cause text, and raw body bytes in ResultMessage/KmipOperationResult/client-error Display and Debug, the public source chain, and any existing library-generated logs. Drop probes verify the original source is not retained privately after sanitization.
- Audit the affected result/error paths for logging call sites and logger dependencies. If no such logging exists, record that finding and ensure this feature adds no logger or logging call; if logging exists, test captured output for the sentinel set.
- Test code stays outside crate src directories.
- Tests and requirement traceability are committed with implementation. TDD evidence uses separate Red, Green, and Refactor commits.

## Project Structure

### Documentation

- specs/003-core-types-errors/: approved feature, data model, contracts, validation guide, checklists, and tasks.
- specification/compliance/requirements/: stable source-to-code-to-test records for in-scope OASIS clauses.
- docs/architecture/public-api.md: result/error ownership and safe display contract.
- docs/development/testing.md: focused error and request-delivery test expectations, if current guidance needs updating.

### Source code

- crates/kmipkit-protocol/src/result.rs and crates/kmipkit-protocol/src/error.rs
- crates/kmipkit-protocol/src/error.rs
- crates/kmipkit-protocol/tests/result_contract.rs
- crates/kmipkit-transport/src/error.rs
- crates/kmipkit-transport/tests/delivery_state.rs
- crates/kmipkit-client/src/error.rs
- crates/kmipkit-client/tests/error_contract.rs
- crates/kmipkit/src/lib.rs re-exports
- specification/compliance/requirements/KMIPKIT-0003.csv

No production code is added to the FFI or binding crates.

## Phase 0: Research Decisions

Resolved in research.md. No package or toolchain comparison is needed. Existing architecture gives each layer clear ownership; the normative catalog is the single source for known KMIP values; the error display and causal chain are separate to prevent accidental disclosure. The only external prerequisite is merging KMIPKIT-0002 before implementation.

## Phase 1: Data and Contract Design

See data-model.md and contracts/error-contract.md. Quickstart scenarios are in quickstart.md.

## Constitution Re-check

PASS subject to the catalog dependency gate and later CI/TDD evidence. No accepted boundary or framework file is changed.

## Complexity Tracking

None. The feature uses the existing workspace layers and adds no dependency or crate.
