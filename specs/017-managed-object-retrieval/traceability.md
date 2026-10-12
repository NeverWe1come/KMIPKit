# KMIPKIT-0017 Requirement Traceability

**Status**: Incremental traceability. The T037 source-item and derived-vector rows below have executable evidence; other entries retain their task-specific status.<br>
**Normative source**: Pinned OASIS KMIP Specification v2.1.<br>
**Scope count**: 15 direct catalog inventory links: 2 Get and 13 Locate. KMIPKIT-DEC-005 classifies 5 Locate rows as server-only and retires 1 false extraction; the 7 remaining Locate rows are client-applicable.

## Release-contract baseline

The historical release-contract snapshot reviewed for the design refresh was `release/1.0.0` at `4e15a8f15c4c8529426b004737aaf16961d1c073`. The verified implementation baseline is active `release/1.0.0` at `fd6b7782b4efe81c9234999cf2faacef64e5fcf6`, where the release-contract refresh merged in PR #81. The current client has non-exhaustive `ClientRequest`, `ClientOperation`, and `ClientBatchOutcome` surfaces with typed, one-exchange response handling; Get and Locate are planned additive variants. `ClientResponseView` is the unified typed-response access path, including for Pending outcomes. `ResponseBatchItemView::with_ttlv` provides callback-scoped access to the complete ordered response item. Any Get Any Object data retained in `GetResponse` must be copied into owned, redacted, zeroizing TTLV before that callback ends; KMIPKIT-0017 adds `Item::try_clone` to copy just that item. `AttributeSet` remains the direct-item Locate criterion representation; its current contract includes fallible validation of catalogued TTLV types and Vendor Attribute shape/order, added after its initial KMIPKIT-0014 introduction. T001 records the baseline and verifies these shared contracts before implementation tests.

## Operation elements

| Catalog element | OASIS source | Specification | Planned code | Planned executable verification |
| --- | --- | --- | --- | --- |
| KMIPKIT-ELEM-OP-C2S-GET | §6.1.19, Tables 220–222; §8.6/Table 399 and §9.1/Table 400 for Pending metadata | spec.md US1, US3; FR-001–005, FR-011–013 | crates/kmipkit-ttlv/src/item.rs (`Item::try_clone`); crates/kmipkit-protocol/src/get.rs; crates/kmipkit-client/src/execute.rs (`PendingResponse`, `ClientResponseRef`, `ClientResponseView`) | crates/kmipkit-ttlv/tests/public_item_contract.rs; crates/kmipkit-protocol/tests/unit/get_tests.rs; crates/kmipkit-client/tests/unit/object_read_execution_tests.rs (completed and valid Pending outcomes) |
| KMIPKIT-ELEM-OP-C2S-LOCATE | §6.1.28, Tables 247–249; §8.6/Table 399 and §9.1/Table 400 for Pending metadata | spec.md US2, US3; FR-001–002, FR-006–013 | crates/kmipkit-protocol/src/locate.rs; crates/kmipkit-client/src/execute.rs (`PendingResponse`, `ClientResponseRef`, `ClientResponseView`) | crates/kmipkit-protocol/tests/unit/locate_tests.rs; crates/kmipkit-client/tests/unit/object_read_execution_tests.rs (completed and valid Pending outcomes) |

## Normative rows and verification intent<br>

| Stable requirement ID | Source | Strength / actor assessment | Spec | Planned code destination | Verification intent |
| --- | --- | --- | --- | --- | --- |
| KMIPKIT-REQ-SPEC-6.1.19-001 | §6.1.19; §7.18/Table 370; Table 16; Table 494 | MAY; client request | FR-005; US1 AC-6 | `crates/kmipkit-protocol/src/get.rs::GetRequest::with_key_wrapping_specification` | `crates/kmipkit-protocol/tests/unit/get_tests.rs::request_encodes_a_valid_wrapped_get_specification` verifies Wrapping Method Encrypt plus Encryption Key Information containing Unique Identifier; optional crypto parameters, MAC/signature information, and encoding option remain omitted, preserving the TTLV default. No cryptographic processing or official case pass is claimed. |
| KMIPKIT-REQ-SPEC-6.1.19-002 | §6.1.19; KMIPKIT-DEC-003 | SHALL; client assumes same-as-registered server capability; other conversions are optional server capabilities | FR-005; US1 AC-3, AC-6 | `crates/kmipkit-protocol/src/get.rs::GetRequest::with_key_format_type`; English/Spanish retrieval guides | `crates/kmipkit-protocol/tests/unit/get_tests.rs::request_preserves_the_optional_key_format_type_enumeration`; `crates/kmipkit-test-support/tests/fixtures/oasis_get_fixture_tests.rs::pkcs12_case_1_preserves_get_item_at_correlation_step_8`; `...::pkcs12_case_2_preserves_get_items_at_correlation_steps_5_7_and_9`. These local checks do not establish server capabilities, conversion behavior, or PKCS#12 container validity. |
| KMIPKIT-REQ-SPEC-6.1.28-001 | §6.1.28 | MAY; client request | FR-006; US2 AC-1 | locate.rs Maximum Items | Omitted and explicit value vectors |
| KMIPKIT-REQ-SPEC-6.1.28-002 | §6.1.28 | MAY; client request | FR-006; US2 AC-2 | locate.rs Offset Items | Omitted and explicit-zero vectors; official fixtures unavailable |
| KMIPKIT-REQ-SPEC-6.1.28-004-001 | §6.1.28 | REQUIRED; caller-directed client workflow | FR-010; US2 AC-10 | `crates/kmipkit-client/src/execute.rs::Client::execute`; English/Spanish retrieval guides | `crates/kmipkit-client/tests/unit/object_read_execution_tests.rs::caller_directs_archived_locate_recover_then_get_as_three_exchanges` verifies three separate caller-issued requests and no automatic follow-up. Fake responses verify the client sequence only; no server behavior is claimed. |
| KMIPKIT-REQ-SPEC-6.1.28-004-002 | §6.1.28 | SHALL; server-only ID Placeholder behavior per DEC-005 | FR-010; US2 AC-9 | No client implementation of server state | Test exact batch preservation separately; do not claim client controls placeholder |
| KMIPKIT-REQ-SPEC-6.1.28-007 | §6.1.28 | MUST; client request | FR-007; US2 AC-4 | locate.rs attributes | Preserve partial structured attribute criteria |
| KMIPKIT-REQ-SPEC-6.1.28-008-001 | §6.1.28 | SHALL; server-only Fresh matching per DEC-005 | FR-007; US2 AC-5 | No client implementation of matching | Client forwards Group Member Fresh; server matching is separate |
| KMIPKIT-REQ-SPEC-6.1.28-008-002 | §6.1.28 | Retired extraction per DEC-005; Default sentence has no separate SHALL | FR-007; US2 AC-5 | No normative code destination | Client forwards Group Member Default under the Table 247 request model |
| KMIPKIT-REQ-SPEC-6.1.28-009-001 | §6.1.28 | SHALL NOT; server-only destroyed response rule per DEC-005 | FR-009; US2 AC-6; Edge Cases | No client filtering | Client vector proves no invented mask/filtering; it cannot prove server conformance |
| KMIPKIT-REQ-SPEC-6.1.28-009-002 | §6.1.28 | SHALL NOT; server-only archived response rule per DEC-005 | FR-009; US2 AC-6; Edge Cases | No client filtering | Client vector proves no invented mask/filtering; it cannot prove server conformance |
| KMIPKIT-REQ-SPEC-6.1.28-001 | §6.1.28 | MAY; client request | FR-006; US2 AC-1 | locate.rs Maximum Items | Preserve omission and explicit value |
| KMIPKIT-REQ-SPEC-6.1.28-012 | §6.1.28 Table 247 | SHALL; server-only online default per DEC-005 | FR-009; US2 AC-6 | No client filtering | Preserve omitted mask and returned values; no claim about server's returned IDs |
| KMIPKIT-REQ-SPEC-6.1.28-013-001 | §6.1.28 | MUST; client request | FR-007; US2 AC-4 | locate.rs AttributeSet encoding | Exact ordered criteria; no local matching |
| KMIPKIT-REQ-SPEC-6.1.28-013-002 | §6.1.28 | MAY; client request | FR-006; US2 AC-3 | locate.rs required Attributes Structure | Preserve present empty structure |

## Optional Table 247 request field evidence

| Source row | Approved feature mapping | Implementation and verification | Evidence boundary |
| --- | --- | --- | --- |
| KMIP v2.1 §6.1.28, Table 247, Storage Status Mask (Required=No) | KMIPKIT-ELEM-OP-C2S-LOCATE; KMIPKIT-0017 FR-006 and FR-009 | `crates/kmipkit-protocol/src/locate.rs::LocateRequest::to_ttlv_payload`; protocol tests `request_emits_supplied_optional_fields_in_table_247_order` and `storage_status_mask_preserves_assigned_and_unknown_bits`; client tests `locate_forwards_assigned_and_unknown_storage_status_bits_unchanged` and `locate_without_storage_status_mask_preserves_all_server_returned_identifiers` | Verifies client request omission/value and bit preservation only. Omission defaults and archived/destroyed response exclusions are server duties; no per-cell REQ identifier or server conformance claim is created. |

## Pinned OASIS and profile test-case evidence

Catalog test IDs, source case IDs, fixture availability, and mapping confidence are kept in separate columns. The profile case has its own table so it is not presented as an OASIS conformance case. Fixture availability and mapping confidence match the corresponding catalog records. For the Get cases, the pinned Test Cases §§2.68–2.69 headings and hyperlink targets use `TC-PKCS12-…`, while visible link text inserts a hyphen as `TC-PKCS-12-…`. KMIPKIT-DEC-004 selects the heading/target identifiers and pins both XML files with URL and checksum; no official-case pass is claimed by fixture presence alone.

### OASIS conformance cases

| Catalog test ID | Official OASIS case ID | Operation / evidence class | Pinned fixture availability | Mapping confidence | Local evidence and claim |
| --- | --- | --- | --- | --- | --- |
| KMIPKIT-TEST-CN01-2-68 | TC-PKCS12-1-21 | Get / selected source item | Available: `specification/oasis/kmip-2.1/fixtures/TC-PKCS12-1-21.xml` | explicit | `crates/kmipkit-test-support/tests/fixtures/oasis_get_fixture_tests.rs::pkcs12_case_1_preserves_get_item_at_correlation_step_8` checks one selected pair and opaque Any Object; full case not executed, no official pass. |
| KMIPKIT-TEST-CN01-2-69 | TC-PKCS12-2-21 | Get / selected source items | Available: `specification/oasis/kmip-2.1/fixtures/TC-PKCS12-2-21.xml` | explicit | `crates/kmipkit-test-support/tests/fixtures/oasis_get_fixture_tests.rs::pkcs12_case_2_preserves_get_items_at_correlation_steps_5_7_and_9` checks three selected pairs and opaque Any Objects; full case not executed, no official pass. |
| KMIPKIT-TEST-CN01-2-62 | TC-MDO-2-21 | Locate / derived local vector | Unavailable in pinned OASIS files | explicit | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-62-TC-MDO-2-21-derived.json`; `derived_metadata_only_case_vector_contains_a_locate_item_without_server_results` checks criteria only, no matching or server-return claim. |
| KMIPKIT-TEST-CN01-2-63 | TC-MDO-3-21 | Locate / derived local vector | Unavailable in pinned OASIS files | explicit | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-63-TC-MDO-3-21-derived.json`; `derived_key_value_location_case_vector_contains_url_locate_criteria` checks a synthetic URL criterion only. |
| KMIPKIT-TEST-CN01-2-64 | TC-OFFSET-1-21 | Locate / derived local vector | Unavailable in pinned OASIS files | explicit | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-64-TC-OFFSET-1-21-derived.json`; `derived_offset_case_vector_contains_locate_offset_input` checks synthetic offset input only. |
| KMIPKIT-TEST-CN01-2-65 | TC-OFFSET-2-21 | Locate / derived local vector | Unavailable in pinned OASIS files | explicit | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-65-TC-OFFSET-2-21-derived.json`; `derived_symmetric_key_offset_case_vector_contains_locate_offset_input` checks synthetic offset input and catalog Symmetric Key context only. |

### Profile case

| Catalog test ID | Profile case ID | Operation / evidence class | Pinned fixture availability | Mapping confidence | Local evidence and claim |
| --- | --- | --- | --- | --- | --- |
| KMIPKIT-TEST-PROF-5-12-6-3 | TL-M-3-21 | Get/Locate / profile test case | Unavailable in pinned OASIS files | explicit | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-PROF-5-12-6-3-TL-M-3-21-derived.json`; `derived_profile_tape_library_vector_covers_only_locate_and_get_payloads` checks synthetic payloads only; no profile-support claim. |

The implementation PR replaces planned file paths with verified symbols and tests, adds catalog implementation_refs and verification_refs only for demonstrated client obligations, regenerates coverage-report.md from the pinned repository tool, and records server-only obligations separately. Do not convert fixture availability or server behavior into a client conformance claim.

## T035 Green dispositions

- **T030 — Get secret redaction and owned-payload zeroization:** No additional production change was required. `crates/kmipkit-protocol/tests/get_secret_handling.rs` passed 4/4, covering wrapping-field redaction, sanitized wrapping errors, owned nested Any Object data after the callback source is dropped, and redacted malformed-object errors. `crates/kmipkit-ttlv/tests/value_zeroization.rs` passed 2/2, directly verifying that TTLV payload variants and nested structures are cleared on zeroization/drop. The protocol ownership test establishes the Get copy and drop contract; the TTLV owner tests establish clearing. These local tests do not establish an official OASIS case or live-server behavior.
- **T032 — Locate omission, unknown mask bits, and no client filtering:** No additional production change was required. `crates/kmipkit-client/tests/unit/object_read_execution_tests.rs` passed 8/8, including captured-request evidence that an omitted Storage Status Mask stays omitted, assigned and unknown mask bits reach the wire unchanged, and all server-returned identifiers remain exposed in order. This verifies client preservation only; it does not claim the server follows its own online/archived/destroyed response obligations or establish live-server behavior.

## T037 fixture evidence

- The pinned `TC-PKCS12-1-21.xml` request/response pair at correlation step 8 and the three `TC-PKCS12-2-21.xml` pairs at steps 5, 7, and 9 are checked by test-support unit tests. These are four selected Get item pairs total, not complete OASIS case runs. The request's explicit PKCS#12 Key Format Type is checked where present; response Any Object items are structurally/value-compared with a zeroizing clone, with constant non-payload diagnostics.
- Five unavailable catalog mappings have one clearly labeled synthetic local TTLV vector and one executable check each, as linked in the table. Their tests validate request evidence only; they do not invent Locate matches, returned object IDs, official XML bytes, or profile behavior.
- For source parsing only under unit-test configuration, unresolved Unique Identifier symbols are mapped to stable local text aliases. The unresolved private-key `KeyMaterial` symbol in the pinned Get response is represented as an opaque one-byte test sentinel so generic TTLV shape and clone preservation can be checked without outputting key material. The sentinel is not claimed to equal official key bytes, and no key payload is logged or interpolated into assertions.

## T045 convergence evidence

- **KMIPKIT-REQ-SPEC-6.1.19-001:** The valid wrapped Get request test follows OASIS KMIP v2.1 §7.18/Table 370: Wrapping Method is present and Encryption Key Information contains the required Unique Identifier from Table 16. Wrapping Method Encrypt uses the value in Table 494. No crypto-parameter, MAC/signature, or encoding-option field is needed for this minimal structure; omitted Encoding Option retains the TTLV default. This verifies client request construction, not wrapping execution or server support.
- **KMIPKIT-REQ-SPEC-6.1.19-002:** §6.1.19 says the client SHALL assume a server can return a key in the format used at registration; conversion to another format MAY be supported by the server. The client model preserves the caller's Key Format Type and the opaque returned TTLV. Existing enum-preservation and selected pinned PKCS#12 item tests verify only those local behaviors. They do not test remote server capability, conversion, or PKCS#12 container validity; DEC-003 continues to classify the lowercase PKCS#12 paragraph as informative guidance.
- **KMIPKIT-REQ-SPEC-6.1.28-004-001:** The archived retrieval test invokes Locate, Recover, and Get through three explicit `Client::execute` calls, uses the returned Unique Identifier in Recover and Get, and checks the request count after each call. It proves the caller-directed client sequence without implementing Recover in KMIPKIT-0017, automatic orchestration, or server-side archive behavior.
