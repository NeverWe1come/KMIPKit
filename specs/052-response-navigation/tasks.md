# Tasks: KMIPKIT-0052 Typed Responses and TTLV Navigation

**Input**: `spec.md`, `plan.md`, `gap-matrix.md`, `traceability.md`, and ADR-0018.

**Prerequisites**: KMIPKIT-0051 is the immediate stacked predecessor. Scope and branch base were approved in `approval-record.md`.

## Phase 1: Red — executable contracts

- [x] T001 [P] Add a public TTLV test proving an Item containing a Structure returns a borrowed StructureView and non-Structure values return absence (`KMIPKIT-0052-R060`).
- [x] T002 Add client tests for raw Response Payload and Response Header views, distinguishing absent payload from present empty Structure and preserving child order/repetition (`R057`, `R058`, `P002`, `P003`).
- [x] T003 Add a client response fixture with vendor tags at arbitrary nested depth, including Key Block and Server Information, and assert traversal and value preservation without exposing Debug values (`R004`, `R005`, `R059`, `R060`, `P006`). Rust Get and Query response tests plus compiled C, Java TLS, and Python TLS consumers cover both nested structures.
- [x] T004 Add async typed-reader tests for Cancel, Poll, Process, and Query Asynchronous Requests for matching and mismatching outcomes; preserve the unresolved Table 286 payload generically (`R001`, `P005`).
- [x] T005 Add response ownership/lifetime tests proving raw Rust views borrow the result owner and the retained message tree is zeroized when its last owner is dropped (`R007`, `R009`, `P001`, `P004`). Borrowing is compile-fail tested at the Item/Structure boundary; reordered batch items prove one shared `Arc<ResponseMessage>`, each associated raw payload remains readable, and the final weak reference cannot upgrade after the result owners drop. The retained TTLV owner recursively zeroizes payloads on drop.
- [x] T006 Add C, Java, and Python consumer tests for owned raw payload/header copies, explicit release/close, nested vendor tags, empty/absent payload, and redaction (`R007`–`R009`). The C consumer traverses after releasing the response; Java and Python traverse owned copies after closing their results.
- [x] T007 Run the focused tests and record the expected Red failures before production implementation. The protocol accessor test failed to compile for the absent `.raw()` methods; the client compile reported all absent async readers and client raw methods.

## Phase 2: Green — Rust and TTLV surface

- [x] T008 Add a direct borrowed `Item::as_structure()` accessor and API documentation; do not change value ownership or formatting.
- [x] T009 Add direct ordered raw access to `ResponseHeaderView` and `ResponseBatchItemView` while preserving existing callback accessors.
- [x] T010 Retain one shared `Arc<ResponseMessage>` and response-item index in high-level batch results; implement `ClientBatchItemResponse::raw()` and batch response header access without cloning a payload per result.
- [x] T011 Add direct `ClientOperationOutcome::raw()` and typed readers for Cancel, Poll, Process, and Query Asynchronous Requests, returning `None` for a different operation.
- [x] T012 Run focused Rust/TTLV tests and verify Red tests turn Green without changing response validation or asynchronous semantics.

## Phase 3: Green — C, Java, and Python consumers

- [x] T013 Add fixed-width C ABI functions returning explicitly owned, zeroizing Structure handles for high-level response payloads and headers; preserve null/absent conventions and panic containment.
- [x] T014 Add compiled C consumer coverage for arbitrary nested traversal, vendor tags in Key Block and Server Information, release behavior, and response lifetime independence.
- [x] T015 Add Java owned response-tree wrappers with `AutoCloseable`, JNI ownership transfer, nested traversal, and close tests.
- [x] T016 Add Python owned response-tree wrappers with `close()` and context-manager behavior, nested traversal, and lifetime tests.
- [x] T017 Run real consumer tests in C, Java, and Python and confirm sensitive payload canaries remain absent from automatic formatting and error messages. Focused JNI and Python TLS integration tests pass; the compiled C consumer is exercised by the focused Rust FFI test.

## Phase 4: Refactor and contract reconciliation

- [x] T018 Reconcile every new public route and parity mapping in the reviewed API manifest; regenerate all outputs with `tools/api_manifest/generate.py`. The reconciler adds explicit records for new routes and two exact Java JNI bridge exclusions.
- [x] T019 Run `tools/api_manifest/generate.py --check`, focused export/parity checks, and review generated diffs; do not edit generated output manually. Rust audit: 7,757/7,757 exact; C, Java, and Python export audits report no missing or unobserved entries.
- [x] T020 Update English and Spanish user/API documentation for `.raw()`, absence semantics, borrowed Rust lifetime, foreign owned handles, discarded padding, redaction, and external-runtime copy limitations.
- [x] T021 Update the 56-operation gap matrix and traceability implementation/test links; close every checklist line only with executable evidence. The matrix records all 56 operation identities and the four asynchronous typed readers.
- [x] T022 Run formatting, focused and workspace tests, Clippy, C/JNI/CFFI binding checks, traceability, coverage, and applicable security checks; record hosted CI evidence before readiness. Hosted CI run [42487112326](https://github.com/NeverWe1come/KMIPKit/actions/runs/42487112326) passed on head `a0a8a10300139b116cc4737157f38c3d045a6394`. Its selected coverage gate passed at 570/600 (95.00%) changed production coverage, 99.20% TTLV, 96.22% protocol, 94.12% transport, 91.90% FFI, 96.95% Java, 96.51% Python, 85.73% JNI, and 95.15% workspace coverage. The hosted matrix also passed Rust 1.94/stable on Linux, Windows, and macOS; C consumers; Java/JNI and Python bindings; sanitizers; fuzz smoke; dependency/security checks; traceability; and immutable-source inventory checks. Platform-specific coverage evidence is recorded in `specs/053-api-parity-facade/implementation-evidence.md`.
- [x] T023 Review final diffs, ensure Red/Green/Refactor commits are distinct, commit the implementation, push the feature branch, and open/update a PR stacked on the 0051 branch. The canonical release-synced, review-ready PR [#149](https://github.com/NeverWe1come/KMIPKit/pull/149) targets `feature/KMIPKIT-0051-pre-send-validation-release`; it carries the same implementation commit as superseded PR #123. Red, Green, and Refactor commits are distinct.

## Phase 5: Async typed-reader parity convergence

- [x] T024 Add C, Java, and Python operation-specific readers for Cancel, Poll, Process, and Query Asynchronous Requests. Typed fields mirror the Rust outcome readers; mismatched operations return absence. Correlation access remains owner-backed and generic `.raw()` preserves present payloads, including the required empty Pending Process payload.
- [x] T025 Add Red-first C/JNI/Python cases for all four matching operations, cross-operation mismatch, result status, cancellation result, pending state, payload presence/member count, generic Query Async payload, correlation access, and closed-owner behavior. Do not edit the separate KMIPKIT-0053 consumer-parity directory.
- [x] T026 Reconcile the four actual public reader routes and fixed-width C v1 record in the API manifest; regenerate outputs only with the official tool, update the four gap-matrix rows and OASIS traceability, and run focused binding tests plus generator, export, formatting, and lint checks. Evidence and exact results are recorded in the follow-up verification entry in `traceability.md`.
- [x] T027 Resolve independent review findings: copy Python correlation spans while the owner lock is held; expose Query Async Pending status and response correlation consistently in C, Java, and Python; success-gate Cancel echo; and reconcile direct test evidence in the manifest. Red, Green, and Refactor evidence is recorded in the review-corrections entry in `traceability.md`.

## Phase 6: Typed Get and Export managed-object access

- [x] T028 [Red] Add Get and Export response tests requiring typed managed-object and Key Block accessors. Verify object type, Key Block format, nested vendor extension retention, and Debug redaction; run before implementation and record the expected missing-method compile failure (`KMIPKIT-0052-FR-013`, `P008`).
- [x] T029 [Green] Add `GetResponse::{managed_object,key_block}`, `ExportResponse::{managed_object,key_block}`, and `ManagedObject::key_block`. Keep original response TTLV intact; return owned, zeroizing typed copies and sanitized errors.
- [ ] T030 [Refactor] Reconcile specification, traceability, API docs, and the Rust API manifest in the final parity branch; run focused and workspace verification. Keep manifest-generated artifacts untouched in this stacked PR; #150 owns the combined public-route audit and official regeneration.
