# Implementation Plan: KMIP 2.1 Remote RNG Operations

**Branch**: feature/KMIPKIT-0041-rng-operations | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: specs/027-rng-operations/spec.md

## Summary

Add typed client support for RNG Retrieve (§6.1.49, Tables 316–318) and RNG Seed (§6.1.50, Tables 319–321). Reuse the existing one-exchange client, typed batch/Pending, TTLV, result, delivery-state, and zeroizing secret-storage paths. KMIPKit only transports server RNG output and caller seed input; it implements no local random generator.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94
**Primary Dependencies**: Existing workspace crates; no new dependency expected
**Storage**: N/A
**Testing**: Protocol unit/property tests, fake-transport client tests, source-derived vectors, catalog validation and report checks
**Target Platform**: Linux, Windows, macOS
**Project Type**: Rust client library with protocol crate
**Performance Goals**: No new target; each explicit operation performs at most one exchange
**Constraints**: KMIP 2.1 TTLV; required Data Length or Data by tables; preserve unknown values and optional absence; redact/zeroize secret bytes; no retry/Poll/local RNG
**Scale/Scope**: Two operations; one request, response, error table per operation; Rust core and four-language parity

## Constitution Check

- Client-only TTLV scope and exact §6.1.49–§6.1.50 references: PASS.
- Shared client, batch, asynchronous result, delivery, transport, and TTLV constraints reused: PASS.
- No local cryptographic/RNG provider or server-policy inference: PASS.
- RNG Seed Data and RNG Retrieve response Data are treated as secrets: PASS.
- RNG Seed Data Length zero is accepted as a valid response and not converted to an error: PASS.
- Distinct Red, Green, and Refactor evidence commits are planned per operation layer: PASS.
- 0043 gates only public bindings/parity, not independent Rust operation models: PASS.

## Dependencies and sequencing

The active release supplies required codec, message/batch, client, transport, and secret handling. RNG Retrieve and RNG Seed have no dependency on each other or on specs 022–026/028; their protocol modules and tests may be developed independently. Both touch the shared client dispatcher in crates/kmipkit-client/src/execute.rs, so coordinate those edits centrally and sequence them rather than merging concurrent dispatcher changes.

Rust protocol/client work can start before KMIPKIT-0043. Hold only API-manifest mapping, generated C/Java/Python signatures, handwritten binding adapters, and parity tests until the approved 0043 public capability contract is integrated.

## Design

### Protocol models

- Add separate modules rng_retrieve.rs and rng_seed.rs and exports from crates/kmipkit-protocol/src/lib.rs.
- RNG Retrieve request has required Integer Data Length; success response has required opaque Data bytes. Preserve returned bytes; do not add an unstated exact-length rule or generate bytes locally.
- RNG Seed request has required opaque Data bytes; success response has required Integer Data Length. Preserve a zero value without treating it as an error, as required by KMIPKIT-REQ-SPEC-6.1.50-001.
- Use common Result Status/Reason/Message, Pending, correlation, unknown value, delivery, and malformed TTLV behavior. Do not retry or Poll automatically.
- Zeroize KMIPKit-owned request/response secret buffers and redact values from Debug, Display, errors, and logs.

### Client and catalog integration

Add both explicit operations to typed Rust client selection, encoding, response dispatch, and per-item association. Each performs one exchange. Reuse catalog operation values and field IDs; keep shared Data element ownership with its existing feature. Do not modify generated code by hand.

### Test strategy

For each story: table-derived protocol tests, malformed/missing required-field negatives, wire type/round-trip and unknown preservation, sentinel redaction, fake-transport operation code and one-exchange assertions, Failure/Pending/delivery preservation, and no retry. RNG Seed tests specifically exercise Data Length zero as success. Official fixtures are not linked, so local vectors are source-derived and cannot support an official vector or interoperability claim.

## Project Structure

Documentation: specs/027-rng-operations/{plan.md,research.md,data-model.md,quickstart.md,contracts/rust-client.md,traceability.md,tasks.md}

Implementation:
- crates/kmipkit-protocol/src/rng_retrieve.rs; rng_seed.rs; lib.rs
- crates/kmipkit-protocol/tests/unit/rng_retrieve_operation_tests.rs; rng_seed_operation_tests.rs
- crates/kmipkit-client/src/execute.rs; lib.rs
- crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs; rng_seed_execution_tests.rs
- specification/catalog/kmip-2.1.json and generated report only if traceability changes require them
- API manifest/adapters only after 0043

**Structure Decision**: One protocol module and test suite per operation; the shared execute.rs integration is centrally coordinated.

## Risks and mitigations

RNG inputs/outputs are secret: reuse zeroizing ownership and redaction tests. Do not conflate a zero seed-use length with failure. Do not assume returned output length must match requested length unless a normative rule requires it. No local RNG or profile claim. No constitution or architecture boundary change is proposed.
