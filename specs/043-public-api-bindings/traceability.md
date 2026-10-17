# Traceability: KMIPKIT-0043 Public API Manifest and Binding Parity

## Normative source

**Key Management Interoperability Protocol Specification Version 2.1**, OASIS Standard, 14 December 2020, pinned at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.

The API feature itself is project-defined. Each operation inventory row maps to its exact normative OASIS operation section in [the operation dependency matrix](contracts/operation-dependencies.md), §§6.1.1–6.1.57. Normative field and behavior requirements remain traced by the approved owner operation/model specification and the checked-in catalog. This feature does not create a new protocol interpretation.

## Requirement and verification map

| Stable requirement | Authority / source | Design and implementation task coverage | Verification evidence |
|---|---|---|---|
| KMIPKIT-0043-FR-001 | Project definition §§2.1, 5; constitution product constraints | `manifest-v2.md`; T004–T010 | Strict schema tests; complete manifest review |
| KMIPKIT-0043-FR-002 | KMIP Specification v2.1 §§6.1.1–6.1.57; normative catalog `client_1_0` entries | `operation-dependencies.md`; T002, T006–T007, T070 | Exact 56-entry catalog equality and section match; §6.1.54 is deferred to 1.1 |
| KMIPKIT-0043-FR-003 | AGENTS.md §§1, 3, 5; constitution §§I, V | Operation readiness contract; T003, T007, T031–T069 | Missing/unapproved spec rejection; all owners resolve at release |
| KMIPKIT-0043-FR-004 | Constitution §I; AGENTS.md §§3, 5, 9 | Manifest and parity contracts; T003–T007, T078, T081 | Referential-integrity and 100% traceability checks, including the approved 025 guard capability and the narrowly scoped KMIPKIT-0021-FR-015 response-TTLV capability |
| KMIPKIT-0043-FR-005 | Project definition §§2.1 and 5; ADR-0006; ADR-0003/ADR-0014 exception | Parity matrix; T012–T030, T031–T069, T077–T078, T081 | Fixture `api-levels.shared.v1` verifies the local three-level query in Rust, C, Java, and Python with zero network exchanges; T078 maps the approved local Usage Allocation guard; T081 verifies caller-owned generic TTLV access only for approved UID shape errors in successful MAC, MAC Verify, Sign, and Signature Verify responses; transport exception is separate |
| KMIPKIT-0043-FR-006 | Constitution §III; ADR-0006; approved operation specifications | Plan phases B/D; T013–T034, T037–T069 | Compare adapter output to canonical Rust behavior; no adapter protocol path |
| KMIPKIT-0043-FR-007 | `docs/architecture/ffi-and-bindings.md`, ABI and ownership sections | Manifest C mapping; T014–T019, T076, T078, T081 | Compiled C ABI consumer, ABI compatibility, ownership/error, typed-client, and local guard tests; T081 compiles a public-header consumer that traverses and releases TTLV only for an approved UID shape error; T076 blocked-export prototypes are not public-header evidence, and T021/T024 must supply real foreign mappings before generation |
| KMIPKIT-0043-FR-008 | ADR-0006; `docs/architecture/ffi-and-bindings.md` Java section | Java adapter; T020–T022, T026, T028–T030, T077–T078, T081 | Java 17 JNI parity, local API-level discovery, guard lifecycle, load, mismatch, and approved UID-shape TLS ownership/redaction tests |
| KMIPKIT-0043-FR-009 | ADR-0006; `docs/architecture/ffi-and-bindings.md` Python section | Python adapter; T023–T025, T027–T030, T077–T078, T081 | Python 3.12 CFFI parity, local API-level discovery, guard typing/lifecycle, load, and approved UID-shape ownership/redaction tests |
| KMIPKIT-0043-FR-010 | Constitution §III; ADR-0006; ADR-0003/ADR-0014; project definition §5 | Parity contract; T011–T030, T031–T071, T076–T078, T081 | Fixture `api-levels.shared.v1` checks identical local API-level results across all four languages, distinguishes them from Discover Versions, and leaves operation-mapping completeness unasserted; T078 proves equivalent local guard behavior; T081 limits generic response access to the KMIPKIT-0021-FR-015 UID-shape cases; only accepted Rust-only ADR exception |
| KMIPKIT-0043-FR-011 | Constitution §IV; generic TTLV specification 0004; approved model specs | Parity contract; T012, T029, T070 | Unknown enum/bitmask/tag/value preservation fixtures |
| KMIPKIT-0043-FR-012 | Project definition §2.1; draft dependency spec 0009; client spec 0007 | Parity contract; T012, T035, T071, T076, T078 | Batch order, Pending/correlation, delivery state, local guard reservation transitions, and no-auto-follow-up tests |
| KMIPKIT-0043-FR-013 | Constitution §IV; AGENTS.md §8; accepted error/secret model specs | Manifest security/error records; T015–T018, T021–T025, T029, T076, T078, T081 | Redaction, error-category, and delivery-state parity tests, including guard validation errors and diagnostics that never disclose the retained response |
| KMIPKIT-0043-FR-014 | Current v1 schema; registry manifest contract in spec 0012 | `manifest-v2.md`; T001, T004–T005 | v2 schema identifier/version and migration tests |
| KMIPKIT-0043-FR-015 | ADR-0004; AGENTS.md §§10 and 12; `docs/development/api-manifest.md` | Generator plan; T006, T008–T011, T070 | Deterministic offline write/check, stale output and path tests |
| KMIPKIT-0043-FR-016 | `docs/architecture/ffi-and-bindings.md` native loading; AGENTS.md §8 | Phase C; T026–T028, T072 | Offline package load tests on every currently supported target |
| KMIPKIT-0043-FR-017 | Constitution §§I–III; AGENTS.md §§5–7, 12 | Parity/release gates; T070–T078 | 56/56 in-scope inventory, no blocked row, full coverage/traceability evidence |
| KMIPKIT-0043-FR-018 | AGENTS.md §§9 and 11; roadmap phases 3–5 | Documentation and metadata; T019, T030, T073–T075 | Docs/API/package version cross-check and release review |
| KMIPKIT-0043-FR-019 | AGENTS.md §§1, 3, 5; constitution §I; shared dependency statuses | `shared-dependencies.md`; T013, T035–T036, T070, T076, T078 | Unapproved or unresolved shared capability rejection; the production client foundation and 025 guard map only after their approved owner/API prerequisites are satisfied |
| KMIPKIT-0043-FR-020 | ADR-0003; `docs/design/project-definition.md` §12; ADR-0014 | Package inventory and transport boundary; T001, T005–T006, T011, T013 | Exact five Rust crate packages plus C/Java/Python families; no unpublished crates; transport exception is ADR-backed |
| KMIPKIT-0043-FR-021 | ADR-0003; ADR-0006; `docs/architecture/ffi-and-bindings.md`; AGENTS.md §§5, 9–10 | Public declaration model and export audit; T001, T004–T006, T009, T011, T070, T076–T078, T081 | Every exported declaration resolves to a capability or exact approved exclusion, including the 025 Usage Allocation guard and the KMIPKIT-0021-owned UID-shape response accessor |
| KMIPKIT-0043-FR-022 | AGENTS.md §3; constitution §V | TDD and implementation PR gates; all implementation tasks, T074 | Ordered Red/Green/Refactor evidence exists in distinct development commits with commands/results |

## Success-criteria map

| Success criterion | Verification | Task |
|---|---|---|
| KMIPKIT-0043-SC-001 | Schema positive/negative contract suite | T001, T004, T011 |
| KMIPKIT-0043-SC-002 | Exact catalog inventory and section comparison | T002, T006, T070 |
| KMIPKIT-0043-SC-003 | Owner-spec readiness and no-blocked release assertion | T003, T007, T031–T070, T075 |
| KMIPKIT-0043-SC-004 | Four-language shared and operation parity corpus | T012, T017, T020–T025, T031–T071, T076–T078 |
| KMIPKIT-0043-SC-005 | Repeat write/check and stale-output negative test | T008–T011 |
| KMIPKIT-0043-SC-006 | C/Java/Python consumers and offline native package loading | T017, T028, T072, T076, T078 |
| KMIPKIT-0043-SC-007 | Traceability validator and reviewer checklist | T070, T073–T075 |
| KMIPKIT-0043-SC-008 | Approved owner resolution and complete shared-capability parity | T003, T013, T035–T036, T070–T078 |
| KMIPKIT-0043-SC-009 | Public symbol inventory and accepted Rust-only transport scope audit | T006, T011, T013, T070, T075 |
| KMIPKIT-0043-SC-010 | Package/version inventory and export comparison | T001, T005–T006, T011, T073, T075 |
| KMIPKIT-0043-SC-011 | Implementation commit history and PR verification record | T034, T042, T074 |

## Operation-level traceability rule

For every capability that corresponds to a KMIP operation, its manifest declaration and conformance evidence cite the exact section from `contracts/operation-dependencies.md` and the owning approved operation specification's stable requirement IDs. This table is an API parity contract, not a substitute for operation requirement traceability. Do not mark an OASIS clause verified merely because generated bindings compile.

## T043 — Certify mapping (reopened for generic TTLV parity)

| Task | Approved owner and normative source | Contract and stable trace IDs | Four-language mapping and parity evidence |
|---|---|---|---|
| T043 | `specs/022-object-lifecycle/spec.md` (`KMIPKIT-0036`, approved 2026-10-11); OASIS KMIP Specification v2.1 §6.1.6, Tables 179–181 | Certify request construction, open 32-bit Certificate Request Type, sensitive Certificate Request Value, optional request structures, generated response Unique Identifier, absent-versus-present response state, success-only generic TTLV, and redacted errors; `KMIPKIT-0036-FR-001`–`FR-006` and `FR-008`, plus `KMIPKIT-REQ-SPEC-6.1.6-001`, `-002-001`–`-002-003`, `-003`, and `-007` | Rust `CertifyResponse::with_ttlv`; C `kmipkit_client_batch_item_response_certify_ttlv`; Java `CertifyResponse.responseTtlv()`/`close()`; Python `CertifyResponse.response_ttlv`/`close()`. The compiled C consumer checks order and an unknown field after releasing the item handle; Java JNI and Python return caller-owned copies. Focused language tests pass locally. T043 remains open pending current-head cross-platform CI and coverage. |

## T044 — Check mapping

| Task | Approved owner and normative source | Contract and stable trace IDs | Four-language mapping and parity evidence |
|---|---|---|---|
| T044 | `specs/022-object-lifecycle/spec.md` (`KMIPKIT-0036`, approved 2026-10-11); OASIS KMIP Specification v2.1 §6.1.7, Tables 183–185 | Optional request Unique Identifier, Usage Limits Count, Cryptographic Usage Mask, and Lease Time; typed response policy fields on either result, Unique Identifier only on success, and complete ordered generic response TTLV; `KMIPKIT-0036-FR-001`–`FR-006` and `FR-008`, `KMIPKIT-REQ-SPEC-6.1.7-001`, `-002-001`, `-002-002`, `-003`, `-004-001`, `-004-002`, and `-005` | Rust `CheckResponse::with_ttlv`; C `kmipkit_client_batch_item_response_check` and `_check_ttlv`; Java `CheckResponse` accessors/`responseTtlv()`/`close()`; Python `CheckResponse` fields/`response_ttlv`/`close()`. Protocol, client, compiled C ABI consumer, Java/JNI, and Python/CFFI tests exercise failed-result identifier masking, policy-field preservation, ordered unknown TTLV, ownership, and Lease Time boundaries. Global binding export audit passes with C 295/295, Java 539/539, and Python 511/511 exact matches. Current-head cross-platform CI and coverage remain pending. |

## T045 — Delegated Login mapping

| Task | Approved owner and normative source | Contract and stable trace IDs | Four-language mapping and parity evidence |
|---|---|---|---|
| T045 | `specs/024-credential-sessions/spec.md` (`KMIPKIT-0038`, approved 2026-10-11); OASIS KMIP Specification v2.1 §6.1.12, Tables 199–201 | Required generic Rights; optional Lease Time Interval or Date Time, Request Count, and generic Usage Limits in Table 199 order; required Table 200 Ticket with raw-preserving Ticket Type, secret Ticket Value, complete ordered TTLV, and unknown children; Table 201 failure alternatives and common result/delivery evidence. Stable traces: `KMIPKIT-024-T199-REQUEST`, `KMIPKIT-024-T200-RESPONSE`, `KMIPKIT-024-T201-ERROR`, operation/value IDs, and the Table 199–201 tag IDs in `specs/024-credential-sessions/traceability.md`. Owner requirements `KMIPKIT-0038-FR-001`–`FR-006` and `FR-008`; API parity requirements `KMIPKIT-0043-FR-004/005/007–013/017/018/021/022`. | Rust `ClientRequest::delegated_login`, `DelegatedLoginRequest`, typed response, and `Ticket` accessors; C `kmipkit_client_batch_item_delegated_login`, `_response_delegated_login`, and `_response_delegated_login_ttlv`; Java `ClientBatchItem.delegatedLogin` and `ClientBatchItemResult.DelegatedLoginResponse`; Python `client_batch_item_delegated_login` and `client.DelegatedLoginResponse`. C returns a response-item-borrowed secret span and an owned ordered Ticket TTLV copy. Java/Python own and wipe their internal byte buffers and close their generic-TTLV copy; Java external `byte[]` and Python immutable `bytes` copies are caller/runtime copies. Focused Rust FFI, compiled C consumer, Java/JNI, Python/CFFI, and manifest parity tests cover bounds, required Rights, unknown Ticket Type and children, redaction, cleanup, and failure-without-Ticket shape. The compiled C consumer checks the declarations and fail-closed output clearing; successful C accessor value semantics are not separately exercised by that consumer. TDD commits and exact command results are in `implementation-evidence.md`. No OASIS certification claim is made. |

## T049 — Get Usage Allocation mapping

| Task | Approved owner and normative source | Contract and stable trace IDs | Four-language mapping and parity evidence |
|---|---|---|---|
| T049 | `specs/025-policy-controls/spec.md` (`KMIPKIT-0039`, approved 2026-10-11); OASIS KMIP Specification v2.1 §6.1.23 Tables 232–234 and §7.40 Table 392 | Optional Unique Identifier, required exact signed Long Integer Usage Limits Count, required resolved Unique Identifier and complete generic response TTLV on success, and common Success/Pending/Failure plus delivery state; `KMIPKIT-025-T232-REQUEST`, `KMIPKIT-025-T233-RESPONSE`, `KMIPKIT-025-T234-ERROR`, `KMIPKIT-0039-FR-001`–`FR-008` (excluding guard-only owner FR-009), `KMIPKIT-REQ-SPEC-6.1.23-002-001`, and `KMIPKIT-0043-FR-004/005/007–010/012/013/015/017/018/021/022` | Rust `GetUsageAllocationRequest`/`ClientRequest`/typed response and `GetUsageAllocationResponse::with_ttlv`; C `kmipkit_client_batch_item_response_get_usage_allocation_ttlv` returns a caller-owned copy on success; Java `GetUsageAllocationResponse.responseTtlv()` returns the owned structure and the result closes it; Python `GetUsageAllocationResponse.response_ttlv` owns the copied structure and `close()` releases it. Evidence: Rust protocol/client tests, FFI ABI tests for complete ordered copies, unknown fields, failure absence and invalid-envelope paths, compiled C consumer lifetime test, Java JNI/JUnit ownership tests, Python CFFI lifecycle tests, and generated manifest parity fixture. Pending and failures expose no generic response TTLV; diagnostics remain redacted and no retry occurs. T081's separate error-only retention exception is unchanged. |
