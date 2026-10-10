# KMIPKIT-0017 Requirement Traceability

**Status**: Planned source/spec/code/test mapping. Code and executable verification references remain pending implementation.<br>
**Normative source**: Pinned OASIS KMIP Specification v2.1.<br>
**Scope count**: 15 direct catalog inventory links: 2 Get and 13 Locate. KMIPKIT-DEC-005 classifies 5 Locate rows as server-only and retires 1 false extraction; the 7 remaining Locate rows are client-applicable.

## Release-contract baseline

The reviewed baseline is `release/1.0.0` at `4e15a8f15c4c8529426b004737aaf16961d1c073`, after KMIPKIT-0021 and the KMIPKIT-0017 inventory dispositions merged. The current client has non-exhaustive `ClientRequest`, `ClientOperation`, and `ClientBatchOutcome` surfaces with typed, one-exchange response handling; Get and Locate are additive variants. `ClientResponseView` is the unified typed-response access path, including for Pending outcomes. `ResponseBatchItemView::with_ttlv` provides callback-scoped access to the complete ordered response item. Any Get Any Object data retained in `GetResponse` must be copied into owned, redacted, zeroizing TTLV before that callback ends; KMIPKIT-0017 adds `Item::try_clone` to copy just that item. `AttributeSet` remains the direct-item Locate criterion representation; its current contract includes fallible validation of catalogued TTLV types and Vendor Attribute shape/order, added after its initial KMIPKIT-0014 introduction. The current refresh must be reviewed and merged before implementation tests.

## Operation elements

| Catalog element | OASIS source | Specification | Planned code | Planned executable verification |
| --- | --- | --- | --- | --- |
| KMIPKIT-ELEM-OP-C2S-GET | §6.1.19, Tables 220–222; §9.19 and Table 399 for Pending metadata | spec.md US1, US3; FR-001–005, FR-011–013 | crates/kmipkit-ttlv/src/item.rs (`Item::try_clone`); crates/kmipkit-protocol/src/get.rs; crates/kmipkit-client/src/execute.rs (`PendingResponse`, `ClientResponseRef`, `ClientResponseView`) | crates/kmipkit-ttlv/tests/public_item_contract.rs; crates/kmipkit-protocol/tests/unit/get_tests.rs; crates/kmipkit-client/tests/unit/object_read_execution_tests.rs (completed and valid Pending outcomes) |
| KMIPKIT-ELEM-OP-C2S-LOCATE | §6.1.28, Tables 247–249; §9.19 and Table 399 for Pending metadata | spec.md US2, US3; FR-001–002, FR-006–013 | crates/kmipkit-protocol/src/locate.rs; crates/kmipkit-client/src/execute.rs (`PendingResponse`, `ClientResponseRef`, `ClientResponseView`) | crates/kmipkit-protocol/tests/unit/locate_tests.rs; crates/kmipkit-client/tests/unit/object_read_execution_tests.rs (completed and valid Pending outcomes) |

## Normative rows and verification intent<br>

| Stable requirement ID | Source | Strength / actor assessment | Spec | Planned code destination | Verification intent |
| --- | --- | --- | --- | --- | --- |
| KMIPKIT-REQ-SPEC-6.1.19-001 | §6.1.19 | MAY; client request | FR-005; US1 AC-6 | get.rs optional fields | Preserve explicit wrapping option and absence; no local wrapping |
| KMIPKIT-REQ-SPEC-6.1.19-002 | §6.1.19 | SHALL; client key-format duty, distinct from DEC-003's lowercase PKCS#12 guidance | FR-005; US1 AC-3, AC-6 | get.rs request fields | Preserve formats and restrictions; do not validate the returned PKCS#12 container locally |
| KMIPKIT-REQ-SPEC-6.1.28-001 | §6.1.28 | MAY; client request | FR-006; US2 AC-1 | locate.rs Maximum Items | Omitted and explicit value vectors |
| KMIPKIT-REQ-SPEC-6.1.28-002 | §6.1.28 | MAY; client request | FR-006; US2 AC-2 | locate.rs Offset Items | Omitted and explicit-zero vectors; official fixtures unavailable |
| KMIPKIT-REQ-SPEC-6.1.28-004-001 | §6.1.28 | MUST; client workflow | FR-010; US2 AC-10 | Retrieval documentation | Document Recover then Get; no Recover implementation |
| KMIPKIT-REQ-SPEC-6.1.28-004-002 | §6.1.28 | SHALL; server-only ID Placeholder behavior per DEC-005 | FR-010; US2 AC-9 | No client implementation of server state | Test exact batch preservation separately; do not claim client controls placeholder |
| KMIPKIT-REQ-SPEC-6.1.28-007 | §6.1.28 | MUST; client request | FR-007; US2 AC-4 | locate.rs attributes | Preserve partial structured attribute criteria |
| KMIPKIT-REQ-SPEC-6.1.28-008-001 | §6.1.28 | SHALL; server-only Fresh matching per DEC-005 | FR-007; US2 AC-5 | No client implementation of matching | Client forwards Group Member Fresh; server matching is separate |
| KMIPKIT-REQ-SPEC-6.1.28-008-002 | §6.1.28 | Retired extraction per DEC-005; Default sentence has no separate SHALL | FR-007; US2 AC-5 | No normative code destination | Client forwards Group Member Default under the Table 247 request model |
| KMIPKIT-REQ-SPEC-6.1.28-009-001 | §6.1.28 | SHALL NOT; server-only destroyed response rule per DEC-005 | FR-009; US2 AC-6; Edge Cases | No client filtering | Client vector proves no invented mask/filtering; it cannot prove server conformance |
| KMIPKIT-REQ-SPEC-6.1.28-009-002 | §6.1.28 | SHALL NOT; server-only archived response rule per DEC-005 | FR-009; US2 AC-6; Edge Cases | No client filtering | Client vector proves no invented mask/filtering; it cannot prove server conformance |
| KMIPKIT-REQ-SPEC-6.1.28-011 | §6.1.28 | MAY; client request | FR-006; US2 AC-1 | locate.rs Maximum Items | Preserve omission and explicit value |
| KMIPKIT-REQ-SPEC-6.1.28-012 | §6.1.28 Table 247 | SHALL; server-only online default per DEC-005 | FR-009; US2 AC-6 | No client filtering | Preserve omitted mask and returned values; no claim about server's returned IDs |
| KMIPKIT-REQ-SPEC-6.1.28-013-001 | §6.1.28 | MUST; client request | FR-007; US2 AC-4 | locate.rs AttributeSet encoding | Exact ordered criteria; no local matching |
| KMIPKIT-REQ-SPEC-6.1.28-013-002 | §6.1.28 | MAY; client request | FR-006; US2 AC-3 | locate.rs required Attributes Structure | Preserve present empty structure |

## Official OASIS case links

Catalog IDs and official case labels are separate fields. The two Get PKCS#12 fixtures are available; the other five listed fixtures remain unavailable. For the Get cases, the pinned Test Cases §§2.68–2.69 headings and hyperlink targets use `TC-PKCS12-…`, while visible link text inserts a hyphen as `TC-PKCS-12-…`. KMIPKIT-DEC-004 selects the heading/target identifiers and pins both XML files with URL and checksum; no official-case pass is claimed by fixture presence alone.

| Catalog test ID | Official case label in catalog | Operation / evidence class | Fixture and claim |
| --- | --- | --- | --- |
| KMIPKIT-TEST-CN01-2-68 | TC-PKCS12-1-21 | Get / official conformance | Available pinned XML; full case not yet executed, no official pass |
| KMIPKIT-TEST-CN01-2-69 | TC-PKCS12-2-21 | Get / official conformance | Available pinned XML; full case not yet executed, no official pass |
| KMIPKIT-TEST-PROF-5-12-6-3 | TL-M-3-21 | Get among profile operations | Unavailable; mapping only, no profile-support claim |
| KMIPKIT-TEST-CN01-2-62 | TC-MDO-2-21 | Locate / official conformance | Unavailable; derived criteria vector only |
| KMIPKIT-TEST-CN01-2-63 | TC-MDO-3-21 | Locate / official conformance | Unavailable; derived attribute vector only |
| KMIPKIT-TEST-CN01-2-64 | TC-OFFSET-1-21 | Locate / official conformance | Unavailable; derived offset vector only |
| KMIPKIT-TEST-CN01-2-65 | TC-OFFSET-2-21 | Locate / official conformance | Unavailable; derived offset boundary vector only |

The implementation PR replaces planned file paths with verified symbols and tests, adds catalog implementation_refs and verification_refs only for demonstrated client obligations, regenerates coverage-report.md from the pinned repository tool, and records server-only obligations separately. Do not convert fixture availability or server behavior into a client conformance claim.
