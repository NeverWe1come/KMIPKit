# Implementation Plan: KMIPKIT-0051 Pre-Send Validation

**Branch**: `feature/KMIPKIT-0051-pre-send-validation` | **Date**: 2026-11-05 | **Spec**: [spec.md](spec.md)

**Input**: Maintainer-approved Phase 5 plan, with the normative-source decisions recorded in [research.md](research.md).

## Summary

Add a redacted, structured Rust validation report for typed client requests and explicitly schema-identified TTLV structures. Inventory every one of the 56 client-initiated operation payloads against its KMIP 2.1 Request Payload table. Reuse existing validation where correct, add only OASIS-backed missing checks, and invoke the same borrowed validators from the client preflight before request encoding and transport. Any local error returns `NotSent` with no transport write; locally valid requests keep their current bytes and behavior.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94.
**Primary Dependencies**: Existing `kmipkit-protocol`, `kmipkit-ttlv`, `kmipkit-client`, and zeroization types; no new dependencies expected.
**Storage**: None; reports borrow nothing after construction and contain safe metadata only.
**Testing**: Rust unit/integration tests with real model values, OASIS-backed schema fixtures, fake/scripted transport, compile-time public API consumers, traceability checks.
**Target Platform**: Existing KMIPKit Rust workspace platforms; local verification may be Windows only while hosted billing is unresolved.
**Project Type**: Rust client library.
**Performance Goals**: Validation is a bounded single pass over supplied request values and introduces no additional exchange, serialization, retry, or Poll.
**Constraints**: Exact KMIP 2.1 structure-specific rules; preserve unknown/future data; do not clone/format secrets; report `NotSent` only before transport; no server-side policy decisions.
**Scale/Scope**: All 56 client-initiated KMIP 2.1 operations and the explicitly listed shared structures used by their typed requests.

## Constitution Check

- **Client/protocol boundary**: no server implementation or 1.1 operation is added.
- **OASIS authority**: every normative rule has an exact clause/table and a stable traceability ID; informative usage text cannot create rules.
- **Lossless model**: unknown tags, future enum values, and unknown bitmask bits remain represented; unknown fields are not rejected solely because they are unknown.
- **Secret handling**: validation is borrowed and redacted; no request bodies or sensitive values appear in reports, errors, or Debug output.
- **Delivery evidence**: all local rejections occur before `transport.exchange` and report `NotSent`; no automatic retry or Poll is introduced.
- **Compatibility**: public additions only; current typed and generic request APIs and successful wire bytes remain unchanged.
- **Testing/traceability**: TDD, focused and workspace tests, exact source links, manifest generation and route audit are required.

## Design Decisions

1. **Validation data model**: Add a public validation report, issue code/path/severity/reference model in `kmipkit-protocol`. Codes are stable identifiers; paths consist only of known field names/tag identifiers and repeated-item indices. Issues never own or format the offending value.
2. **Schema identity**: Add an explicit, exhaustive identity for the 56 in-scope operation Request Payload schemas and the shared structures supported by this release. KMIP v2.1 §6.1 has 57 client-initiated operations; Set Endpoint Role (§6.1.54, Table 331) is client-initiated but deferred to 1.1 by the approved 1.0 product scope. §6.2 server-initiated operations are also out of scope. Generic structures require an explicit identity; there is no tag-based inference. The supported schema inventory and unsupported cases are recorded in `gap-matrix.md`.
3. **Borrowed model validators**: Add a common public request-validation trait or equivalent typed methods, implemented for the request types in scope. Validators inspect existing typed fields and generic `StructureView`s without cloning, serializing, or consuming a request. Where a normative check currently lives only inside a consuming `into_ttlv_payload(self)` path, first add a failing borrowed-validation test, then extract the check into `validate(&self)` and have the serializer reuse that same check; do not duplicate validation logic.
4. **Rule engine**: Use small deterministic schema descriptors for field type, order, requiredness, repeatability, and locally decidable conditions. Operation payload tables are explicit; no tag-number sort and no runtime OASIS scraping. Give each normative rule a stable `KMIPKIT-0051-R###` identifier. Unknown fields remain opaque and preserved; the validator must not reject them merely because its descriptor registry does not recognize them, and must not claim that an unrecognized field is conformant to a closed OASIS structure.
5. **Execution integration**: A single `ClientBatch` preflight validates all request items before `build_request_message`, encoding, or transport exchange. Error-severity issues become a payload-free client validation error with `NotSent`; warnings do not block. Existing request-shape, profile, extension, size-limit, and transport-delivery checks retain their order unless the test evidence proves a safe consolidation.
6. **Conditionals and server decisions**: Apply conditions only when the OASIS clause and supplied request make the condition determinable. Do not infer absent key attributes, server capability, authorization, object state, or vendor policy. KMIPKIT-0050 Cryptographic Parameters behavior is reused without broadening.
7. **Testing/traceability**: Tests prove each schema rule can fail correctly before send, all 56 operations are accounted for, valid bytes are unchanged, and secret canaries never enter diagnostics. `traceability.md` maps each normative rule to exact source, code, and tests.

## Project Structure

```text
crates/kmipkit-protocol/src/validation.rs
crates/kmipkit-protocol/src/message/validation.rs
crates/kmipkit-protocol/src/<operation request modules>
crates/kmipkit-client/src/execute.rs
crates/kmipkit-protocol/tests/validation_*.rs
crates/kmipkit-client/tests/unit/request_validation_tests.rs
crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs
specification/api/public-api.json and generated outputs
specs/051-pre-send-validation/*
```

**Structure Decision**: Keep protocol validation/report types in `kmipkit-protocol`; let `kmipkit-client` adapt the report to existing client errors and delivery states. Do not move transport or server policy into the protocol validator.

## Verification Strategy

- Finish the 56-operation OASIS request-payload matrix and shared-structure inventory before implementing operation rules.
- For every implementation group, add Red tests first, observe the expected failure, then Green and Refactor in distinct commits.
- Test generic and known-schema paths for order, TTLV type, cardinality, requiredness, conditionals, unknown tags, and non-disclosure.
- Use the scripted transport to prove invalid requests return `NotSent` with zero exchange calls/writes and valid requests preserve bytes and one-exchange semantics.
- Run `cargo fmt --all --check`, workspace Clippy and tests, local coverage gate inputs available on Windows, manifest generator `--check`, public route audit, requirement/feature traceability tests, and executable documentation examples.
- Keep the stacked PR in draft while Linux/macOS, the separate C-consumer coverage artifact, or required hosted CI remains unavailable.

## Risks

- KMIP operation Request Payload tables vary substantially; incorrect or incomplete schema inventory can cause false rejection or missed violations.
- Some field requirements depend on request context while others depend on server state; their sources and condition scopes must be distinguished.
- Unrecognized tags are preserved, but the standard does not permit arbitrary unknown children in every Structure; validators must not claim those children are conformant.
- Reports add API surface and become compatibility commitments; codes and paths must be stable and value-free.
- Existing checks may be duplicated; refactor only after equivalence tests preserve `NotSent`, request bytes, profile behavior, and delivery state.
