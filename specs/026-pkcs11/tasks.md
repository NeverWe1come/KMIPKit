---
description: "Implementation task list for KMIP 2.1 PKCS#11 forwarding"
---

# Tasks: KMIP 2.1 PKCS#11 Forwarding

**Input**: Design documents from specs/026-pkcs11/

**Prerequisites**: Approved spec.md; plan.md, research.md, data-model.md, contracts/rust-client.md, traceability.md, quickstart.md

**Testing**: Tests are required by the approved acceptance scenarios and repository TDD rules.

**Organization**: One user story for PKCS#11. Keep protocol model work isolated; coordinate the shared client dispatcher centrally.

## Phase 1: Setup

**Purpose**: Confirm source, branch, and existing integration conventions.

- [x] T001 [P] Verify the approved KMIPKIT-0040 feature branch, pinned OASIS source, and baseline catalog using specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html and python tools/normative_catalog/validate.py.
- [x] T002 [P] Review existing one-operation protocol and fake-client patterns in crates/kmipkit-protocol/src/ping.rs, crates/kmipkit-client/src/execute.rs, and crates/kmipkit-client/tests/unit/ping_execution_tests.rs.

## Phase 2: Foundational

**Purpose**: Establish operation-specific traceability and verify inputs before behavior changes.

- [x] T003 Map every §6.1.37 Table 273/274 row, Table 275 result, and KMIPKIT-ELEM identifier to a verification target in specs/026-pkcs11/traceability.md; do not create a synthetic normative client clause.
- [x] T004 Confirm required operation and tag records in specification/catalog/kmip-2.1.json; if a required record is absent, add source-backed catalog input and run its pinned generator rather than editing generated files.
- [x] T005 Record the operation implementation references and planned named tests in specs/026-pkcs11/traceability.md before coding.

## Phase 3: User Story 1 — PKCS#11 (Priority: P1)

**Goal**: Construct, encode, send, and inspect one PKCS#11 request while preserving optional fields, opaque structures, correlation, common results, and security behavior.

**Independent Test**: A fake transport captures one Table 273 request and returns Table 274 success or Table 275 failure; assertions cover fields/order, result, delivery, one exchange, and no retry.

### Protocol TDD

- [x] T006 [P] [US1] Add failing source-derived Table 273/274 tests in crates/kmipkit-protocol/tests/unit/pkcs11_operation_tests.rs for required Function/Return Code, optional absence, Enum-versus-Integer Function, malformed types, unknown nested members, and redacted sentinel data; run cargo test -p kmipkit-protocol pkcs11_operation_tests and retain the expected failure as Red evidence.
- [x] T007 [US1] Commit only the failing protocol tests in crates/kmipkit-protocol/tests/unit/pkcs11_operation_tests.rs as the distinct Red evidence commit.
- [x] T008 [US1] Implement Pkcs11Request/Pkcs11Response/Pkcs11Error and Table 273/274 conversion in crates/kmipkit-protocol/src/pkcs11.rs; export them from crates/kmipkit-protocol/src/lib.rs; pass the focused protocol tests.
- [x] T009 [US1] Commit the minimal passing protocol implementation in crates/kmipkit-protocol/src/pkcs11.rs and crates/kmipkit-protocol/src/lib.rs as the distinct Green evidence commit.
- [x] T010 [US1] Refactor protocol conversion without changing wire behavior in crates/kmipkit-protocol/src/pkcs11.rs; rerun cargo test -p kmipkit-protocol pkcs11_operation_tests and formatting.
- [x] T011 [US1] Commit protocol-only refactoring in crates/kmipkit-protocol/src/pkcs11.rs as the distinct Refactor evidence commit.
- [x] T012 [P] [US1] Add failing fake-transport tests in crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs for operation selection, exact one exchange, result/Pending/correlation/delivery preservation, no retry, and request redaction; retain expected failures as Red evidence.
- [x] T013 [US1] Commit only the failing client tests in crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs as the distinct Red evidence commit.
- [x] T014 [US1] Add PKCS#11 request/response association, operation code, and explicit one-exchange client method to crates/kmipkit-client/src/execute.rs and exports to crates/kmipkit-client/src/lib.rs; pass the focused client suite.
- [x] T015 [US1] Commit the minimal passing client integration in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs as the distinct Green evidence commit; coordinate this shared dispatcher edit with the feature integrator.
- [x] T016 [US1] Refactor PKCS#11 client dispatch and tests in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs; rerun focused protocol/client suites, formatting, and Clippy.
- [x] T017 [US1] Commit client-only refactoring in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs as the distinct Refactor evidence commit.

## Phase 4: Polish and Cross-Cutting Concerns

**Purpose**: Close traceability, security, parity, documentation, and quality gates.

- [x] T018 Update implementation and verification references for the PKCS#11 operation and fields in specification/catalog/kmip-2.1.json; regenerate specification/catalog/coverage-report.md with python tools/normative_catalog/report.py --write, then validate the catalog and report.
- [x] T019 Add the Rust operation to `specification/api/public-api.json`, implement the mapped C/Java/Python adapters, and verify shared-fixture parity under the KMIPKIT-0043 T057 mapping. The direct human instruction dated 2026-10-20 authorized implementation of the still-approved-for-implementation contract; `formalAcceptance` remains false. Generated outputs use only the pinned `python -B tools/api_manifest/generate.py --write` tool. Release integration, profile/interoperability evidence, and T022 remain separate gates.
- [x] T020 Update specs/026-pkcs11/traceability.md and specs/026-pkcs11/quickstart.md with actual test names/results, catalog IDs, redaction evidence, and unavailable official fixture status; do not claim profile support or interoperability.
- [x] T021 Verify sentinel Function/Correlation/Input/Output values are absent from Debug, Display, and sanitized errors in crates/kmipkit-protocol/tests/unit/pkcs11_operation_tests.rs and crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs.
- [ ] T022 Run required checks from specs/026-pkcs11/quickstart.md; record command/results and enforce the documented coverage and 100% traceability gates. Before 1.0, exercise PKCS#11 against at least two independent KMIP server implementations and record evidence in docs/development/kmip-interoperability.md; unsupported cases remain SKIP/N/A, not passing evidence.

## Dependencies and Execution Order

### Phase dependencies

- Setup has no dependency.
- Foundational traceability/source checks precede implementation.
- Protocol tests and model form the first independently reviewable increment.
- Client fake-transport integration follows the protocol model and requires coordinated edit of execute.rs.
- Polish follows implementation.

### Parallel opportunities

- T001 and T002 are independent.
- Protocol model/test work is operation-specific and can proceed independently of other 022–028 protocol modules.
- Client test design is separate, but execute.rs integration is a shared hotspot and must be coordinated centrally.
- C/Java/Python parity remains gated only on KMIPKIT-0043; it does not block Rust protocol work.

### Implementation strategy

Complete the protocol Red/Green/Refactor cycle, then the client Red/Green/Refactor cycle. Finish traceability and security evidence, integrate the 0043 binding slice when available, then run workspace gates.

## Notes

- Every implementation phase uses three distinct DCO-signed evidence commits: failing tests (Red), minimal implementation (Green), and behavior-preserving cleanup (Refactor).
- Never edit generated output or pinned upstream OASIS files by hand.
- PKCS#11 structures stay opaque; no profile codec, local cryptographic algorithm, retry, Poll, or server behavior is added.


## Execution status — 2026-10-12

T001–T018 and T020–T021 are complete for the Rust increment. Their source,
TDD, named-test, generated-report and diagnostic evidence is recorded in
`traceability.md` and `implementation-evidence.md`.

- **T019 COMPLETE on the KMIPKIT-0043 feature branch**: Rust, public C ABI,
  Java/JNI, and Python/CFFI mappings now consume the same response fixture.
  The owner's status remains **Approved for implementation**; the direct human
  implementation authorization dated 2026-10-20 is separately recorded and
  `formalAcceptance` remains `false`. This does not claim that the feature
  branch has been integrated into `release/1.0.0`.
- **T022 OPEN**: full local tests/coverage/security and actual CI platform/
  adapter/C-consumer/aggregate gates passed; results are recorded in the evidence
  file. Real exchanges against two independent PKCS#11-capable KMIP 2.1 servers
  remain the outstanding release condition. The
  interoperability matrix retains Pending status. This specification is not
  complete and its PR must remain a draft while either task is open.


Independent QA corrections were completed with a further Red/Green/Refactor
cycle: Tables 379–381 known nested scalar/cardinality validation, local NotSent
and received ResponseStarted tests, preserved unknown children, and explicit
Function vendor Integer source-context trace `KMIPKIT-026-SOURCE-001`.
The focused suite now has 15 protocol and 9 client tests; T019/T022 remain open.
