# Implementation Plan: KMIP 2.1 Hash, MAC, and Signature Operations

**Branch**: `feature/KMIPKIT-0021-hash-mac-signature-operations` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/021-hash-mac-signature-operations/spec.md`

## Summary

Add typed Rust client support for Hash, MAC, MAC Verify, Sign, and Signature Verify through the existing KMIP request, response, batch, and transport pipeline. Reuse KMIPKIT-0019's generic Cryptographic Parameters, Data, and multipart field contracts; preserve unknown values and response trees; and perform no local cryptography. Keep the final multipart Validity Indicator discrepancy open and tolerate both source-described final forms without claiming server conformance.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94

**Primary Dependencies**: Existing KMIPKit workspace crates only; no new dependency expected

**Storage**: N/A

**Testing**: Rust unit and integration tests, source-derived TTLV payload vectors, fake client transport, normative catalog validator, deterministic coverage report, and existing CI platform matrix

**Target Platform**: Existing Linux, Windows, and macOS CI matrix

**Project Type**: Rust client library with protocol crate

**Performance Goals**: No new latency or throughput target; each explicit operation call performs at most one exchange and no local cryptographic computation

**Constraints**: KMIP 2.1 TTLV; reuse KMIPKIT-0019 Cryptographic Parameters, operation Data, Correlation Value, Init Indicator, and Final Indicator models; preserve open enum values and generic TTLV; existing TLS 1.3/mTLS transports; no automatic retry; redact operation bytes

**Scale/Scope**: Five client operations; complete request, response, and error tables; all standard Hashing Algorithm, Cryptographic Algorithm, Digital Signature Algorithm, and Validity Indicator values plus extension/future preservation

## Constitution Check

- Exact OASIS sections/tables and stable applicable client requirement IDs are in the spec and traceability: PASS.
- Client-only TTLV scope and existing transport, TLS, batch, and language boundaries are unchanged: PASS.
- The final multipart response conflict is represented as open `KMIPKIT-DISC-048`; no disputed server behavior is selected: PASS.
- Server key lookup, Usage Limits, operation execution, and result production are not implemented by the client feature: PASS.
- Strict Red, Green, Refactor implementation commits and independent QA/security review are planned: PASS.
- Secret redaction, zeroization reuse, unknown value retention, and one-exchange behavior are explicit: PASS.
- Shared cryptographic types are reused from KMIPKIT-0019; implementation must wait until that dependency is merged: PASS.
- No code implementation or change to pinned OASIS sources occurs in this design/specification PR: PASS.

## Design

### Dependency and sequencing gate

KMIPKIT-0019 owns the initial shared Cryptographic Parameters, `OperationData`, Data, Correlation Value, Init Indicator, and Final Indicator contracts. The active release contains its specification but not its runtime operation model. Keep one protocol/client implementer active and implement KMIPKIT-0019 before KMIPKIT-0021. Begin #21 Red tests only after the shared types are available on the active release branch. This prevents parallel changes to the common public model.

### Protocol model

- Add one protocol module each for Hash, MAC, MAC Verify, Sign, and Signature Verify under `crates/kmipkit-protocol/src/`. Export public request, response, and sanitized model-error types from `lib.rs`.
- Requests own caller input and serialize only operation fields present in Tables 235, 259, 262, 334, and 337. `Option` distinguishes absent from supplied values; ordered generic Cryptographic Parameters preserve unknown members. Use the shared `OperationData` type for §7.9 Data, including Byte String, Enumeration, and Integer forms where the operation table permits Data.
- Model MAC Data, Signature Data, and Digested Data as opaque byte strings. Owned KMIPKit byte storage uses the existing zeroizing type; Debug/Display never prints contents.
- Use open raw enumeration wrappers for algorithm and Validity Indicator values. Do not introduce closed-enum conversion that rejects vendor or future values.
- Validate locally decidable field presence and type rules before transmission: Hash Cryptographic Parameters; required single-part input; Sign Data-or-Digested-Data form; and verification inputs. Do not inspect server-held attributes or reject a choice based on unknown remote state.
- Borrow response data from `ResponseBatchItemView` where practical and retain the source generic response item. Success payloads expose the table's fields; non-success retains the shared result without a fabricated successful payload. Successful MAC, MAC Verify, Sign, and Signature Verify payloads validate exactly one well-formed Unique Identifier, returning a sanitized typed shape error for missing, duplicate, or malformed values while leaving generic response access available.
- For final multi-part MAC Verify and Signature Verify responses, accept either Validity Indicator presence or absence while `KMIPKIT-DISC-048` is open. For non-final responses, reject an unexpected indicator in the typed response shape while retaining the generic TTLV source for inspection.
- Assign no local semantics to the listed operation error reasons. Preserve any KMIP result value allowed by the common result contract, including unknown codes.

### Client execution

- Add five variants to the closed `ClientRequest` and `ClientOperation` pipelines in `crates/kmipkit-client/src/execute.rs`, including operation code, payload encoding, response dispatch, identity/correlation, and redacted request formatting.
- Add explicit synchronous client conveniences for each request following existing operation methods and `RequestOptions` overloads. They use the shared batch writer and exchange exactly once.
- Keep batching and per-item result association in the shared execution layer. Do not add operation retries, automatic multipart continuations, polling, local cryptography, server state lookup, or additional transport behavior.
- Add response accessors to `ClientResponseView`/`ClientBatchItemResponse` according to existing conventions; keep generic response tree access available for unrecognized fields.

### Normative catalog

- Assign all five operation elements and operation enum values, operation-specific Data value elements tracked by the catalog, MAC Data and Signature Data structures, and the complete relevant algorithm and Validity Indicator enumeration domains to KMIPKIT-0021. Digested Data remains assigned to the shared tag-registry feature KMIPKIT-0004; this feature traces its Sign and Signature Verify behavior through operation FRs and the linked §2.38 test case.
- Assign only the six source-backed client `MAY` records to the feature; Hash table contracts are traced by feature FRs and operation table metadata, without synthetic per-cell requirement IDs.
- Correct prior actor and extraction mistakes without changing stable IDs: retain the eight ID Placeholder/parameter failure records as server-only; retire five records that were not independent client normative requirements; leave those records unassigned to this client feature.
- Add open discrepancy `KMIPKIT-DISC-048` for the final multi-part Validity Indicator conflict. Retain all existing official test-case links and unavailable fixture status.
- Regenerate `specification/catalog/coverage-report.md` using the pinned report tool; never edit it by hand.

### Test strategy for the implementation PR

- **Protocol model**: positive and negative payload construction/conversion tests for every request/response/error table, field types, requiredness, optionality, single/multi-part shapes, output omission, and exact wire enumeration values. Include missing, duplicate, and malformed Unique Identifier negatives for successful MAC, MAC Verify, Sign, and Signature Verify responses.
- **Response discrepancy**: single-part indicator required; non-final multipart indicator is a typed shape error; final multipart indicator present and absent both decode; generic TTLV preserves the original received tree in every case.
- **Losslessness**: property and round-trip tests for all standard/extension enum values, unknown raw values, generic parameters, byte strings, repeated/unknown fields, and response-tree borrowing.
- **Client execution**: fake-transport tests prove correct operation code, exactly one exchange, request/response association in batches, Result Status/Reason/Message preservation, Pending passthrough, request delivery state, and no retry.
- **Security**: redaction tests use sentinel inputs for Data, Digested Data, MAC Data, Signature Data, and parameters across Debug, Display, and error formatting; owned bytes continue to use the shared zeroizing owner.
- **OASIS vectors**: test only the operation payloads derivable from the pinned tables and record them as source-derived vectors. Linked official fixtures are unavailable; no fixture pass or profile claim is permitted.
- Run formatting, workspace Clippy, tests, coverage gates, catalog validation/report checks, generation drift checks, and CI on Linux, Windows, and macOS before review.

## Project Structure

### Documentation (this feature)

```text
specs/021-hash-mac-signature-operations/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── traceability.md
├── checklists/
│   └── requirements.md
└── contracts/
    └── public-rust.md
```

### Source code (implementation phase)

```text
crates/kmipkit-protocol/src/hash.rs
crates/kmipkit-protocol/src/mac.rs
crates/kmipkit-protocol/src/mac_verify.rs
crates/kmipkit-protocol/src/sign.rs
crates/kmipkit-protocol/src/signature_verify.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs
crates/kmipkit-protocol/tests/unit/mac_operation_tests.rs
crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs
crates/kmipkit-protocol/tests/unit/sign_operation_tests.rs
crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/src/lib.rs
crates/kmipkit-client/tests/unit/hash_execution_tests.rs
crates/kmipkit-client/tests/unit/mac_execution_tests.rs
crates/kmipkit-client/tests/unit/mac_verify_execution_tests.rs
crates/kmipkit-client/tests/unit/sign_execution_tests.rs
crates/kmipkit-client/tests/unit/signature_verify_execution_tests.rs
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md  # generated by report.py
specs/021-hash-mac-signature-operations/traceability.md
```

**Structure Decision**: Follow the existing one-module-per-operation pattern and closed typed `ClientRequest` execution design. Keep operation-specific payload conversion with each protocol model and fake transport assertions in the client test suite. Do not modify generated tag files; all tags needed by these operations already exist in the normative catalog and generated tag registry.

## Risks and mitigations

- **Shared model dependency not yet implemented**: sequence after KMIPKIT-0019 and reuse its merged public types; do not create parallel `OperationData` or Cryptographic Parameters models.
- **Large standardized enumeration sets**: test every assigned value and extension preservation through table-driven vectors; generate no public binding output in this feature.
- **Conflicting final multipart verification shape**: keep `KMIPKIT-DISC-048` open, accept both final forms, reject indicators on non-final typed responses, and retain generic TTLV evidence.
- **Unavailable official fixtures**: label tests as source-derived and report official fixtures as unavailable; do not imply official test-case passes.
- **Sensitive request and response bytes**: use shared secret storage/redaction patterns and test all formatting/error boundaries with sentinel bytes.
- **Release branch advances during implementation**: update from the active release branch, resolve conflicts, validate immutable source hashes, and rerun catalog, formatting, lint, tests, coverage, and security checks.

## Complexity Tracking

No constitution violation or accepted architecture boundary change is proposed.
