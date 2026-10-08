# Normative Traceability: Server-Generated Object Creation

**Feature**: KMIPKIT-0014

**Status**: Draft mappings; implementation symbols and executable test names are finalized in the implementation PR.

**Normative source**: OASIS KMIP Specification v2.1 pinned at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.

## Operation-specific clauses

| OASIS source | Catalog element / stable requirement / official test case | Feature requirements | Tasks | Planned implementation and executable evidence |
|---|---|---|---|---|
| §6.1.8, Tables 186–187; §6.1.8.1, Table 188; §5.1, Table 157 | `KMIPKIT-CLAUSE-SPEC-6.1.8-001`, `KMIPKIT-CLAUSE-SPEC-6.1.8.1-001`, `KMIPKIT-ELEM-OP-C2S-CREATE`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES`, `TC-CREATE-SD-1-21` §2.12 | FR-001, FR-002, FR-006, FR-010, FR-014 | T001, T006–T011 | `crates/kmipkit-protocol/src/create.rs`; `create_tests.rs` covers Table 186's required outer Attributes structure both populated and empty as permitted by §5.1, plus table-derived fields/cardinality/errors and a Create-only TTLV derivation from the pinned official XML; the source case also contains an out-of-scope Get item, so no full `TC-CREATE-SD-1-21` pass is claimed; `create_execution_tests.rs` covers success, Failure, association, and one exchange |
| §6.1.9, Tables 189–190; §6.1.9.1, Table 192; §§5.2–5.4, Tables 158–160 | `KMIPKIT-CLAUSE-SPEC-6.1.9-001/-002/-004`, `KMIPKIT-CLAUSE-SPEC-6.1.9.1-001`, `KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-2-COMMON-ATTRIBUTES`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-3-PRIVATE-KEY-ATTRIBUTES`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-4-PUBLIC-KEY-ATTRIBUTES`, `KMIPKIT-REQ-SPEC-6.1.9-001-001/-002` | FR-001, FR-003, FR-006, FR-013, FR-014 | T012–T017 | `crates/kmipkit-protocol/src/create_key_pair.rs`; `create_key_pair_tests.rs` covers ordered direct attribute items in separate groups, Table 189 union preservation, and every Table 192 Result Reason; `create_key_pair_execution_tests.rs` covers both IDs, failures, batching, and one exchange |
| §6.1.9, Table 191 plus §6.1.9 key-specific precedence | `KMIPKIT-CLAUSE-SPEC-6.1.9-006/-007`, `KMIPKIT-REQ-SPEC-6.1.9-006/-007` | FR-004 | T012–T014 | `crates/kmipkit-protocol/src/create_key_pair.rs`; `create_key_pair_tests.rs` covers all four attributes, both effective values absent, Common fallback, one/both key-specific overrides, equal overrides differing from Common, one-sided effective values, and differing effective values |
| §6.1.10, Tables 193–194; §6.1.10.1, Table 195; §5.1, Table 157 | `KMIPKIT-CLAUSE-SPEC-6.1.10-001/-002/-005`, `KMIPKIT-CLAUSE-SPEC-6.1.10.1-001`, `KMIPKIT-REQ-SPEC-6.1.10-001`, `KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES` | FR-001, FR-005, FR-006, FR-014, FR-015 | T018–T023 | `crates/kmipkit-protocol/src/create_split_key.rs`; `create_split_key_tests.rs` covers the required outer Attributes structure both populated and empty under §5.1, Table 193 optional request-field presence, the separate FR-015 caller-input policy, optional input ID, repeated response IDs, errors, cardinality, and no inferred split parameters; execution tests cover IDs, failures, Pending, and one exchange |
| §2.8, Table 9 (Split Key object); §6.1.10, Table 193 (Create Split Key request) | `KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-PRIME-FIELD-SIZE`; the object-level `KMIPKIT-REQ-SPEC-2.8-003` is not mapped as a request obligation | FR-015 (KMIPKit client policy only) | T018–T020 | `crates/kmipkit-protocol/src/create_split_key.rs`; tests distinguish Table 193's optional request field from the KMIPKit restriction requiring caller-supplied Prime Field Size for Polynomial Sharing Prime Field; no OASIS request MUST is claimed |
| §§5.1–5.4, Tables 157–160 | `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-2-COMMON-ATTRIBUTES`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-3-PRIVATE-KEY-ATTRIBUTES`, `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-4-PUBLIC-KEY-ATTRIBUTES` | FR-002, FR-003, FR-005, FR-014 | T003–T008, T012–T020 | `crates/kmipkit-protocol/src/attribute.rs`; tests verify direct §4 attribute TTLV items, tags, typed values, repetition, wire order, and permitted empty groups; Create tests separately verify the required outer Attributes wrapper |
| §4.60, Table 150 (Vendor Attribute) | `KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE` | FR-014 | T003–T005 | `crates/kmipkit-protocol/src/attribute.rs`; tests apply Table 150 only to Vendor Attribute and verify its required Vendor Identification, not as the wrapper for ordinary attributes |
| §9.12, Table 417 | `KMIPKIT-REQ-SPEC-9.12-001-001/-002/-003` | FR-007, FR-012 | T021–T023 | `crates/kmipkit-client/src/execute.rs`; `create_split_key_execution_tests.rs` asserts Maximum Response Size is `min(local response-byte limit, i32::MAX)`, accepts an exact-limit response, and rejects a one-byte-over response before decoder entry |

## Inherited message, batch, and asynchronous requirements

These requirements are defined by accepted KMIPKIT-0006, KMIPKIT-0007, KMIPKIT-0009, and KMIPKIT-0012 contracts. KMIPKIT-0014 does not redefine them; operation fake-transport tests verify they still hold for the three new operations.

| Stable obligation and OASIS source | Feature requirements | Existing contract / operation-level executable evidence |
|---|---|---|
| `KMIPKIT-REQ-SPEC-8.1-002`; §8.1 | FR-007 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0006 batch validation; each operation test covers one-item request and multi-item batch association where applicable |
| `KMIPKIT-REQ-SPEC-8.3-003/-004`; §§8.3, 8.6 | FR-007, FR-008 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0006/0007 ID and Message Extension contracts; operation tests preserve response association and repeated extensions |
| `KMIPKIT-REQ-SPEC-8-003-002`, `KMIPKIT-CLAUSE-SPEC-8.6-003`, `KMIPKIT-REQ-SPEC-9.2-001`; §§8, 8.6, 9.2; Table 399 | FR-007, FR-008 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0007/0009 Pending and asynchronous-indicator contracts; each operation test asserts the exact operation identity, Result, and correlation value |
| `KMIPKIT-REQ-SPEC-9.1-001`, `KMIPKIT-REQ-SPEC-9.19-002`; §§9.1, 9.19; Tables 400, 424 | FR-008 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0009 owns exact correlation retention and later Poll/Cancel use; KMIPKIT-0014 does not issue Poll/Cancel and tests only preservation of the Pending outcome |
| `KMIPKIT-REQ-SPEC-9.5-001-001/-002`, `9.6-001-001/-002/-003`, `9.7-001`, `9.8-001-001`, `9.9-001`; §§9.5–9.9; Tables 404–408 | FR-007 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0006/0007 common header/batch validation; operation tests cover one and multiple items and caller-selected options. Server-only `9.8-001-002/-003` are not claimed by this client feature |
| `KMIPKIT-REQ-SPEC-9.13-001-001` through `-005`; §9.13, Table 418 | FR-007, FR-011 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0007/0012 own criticality and registry behavior; operation tests verify registered extensions use the shared message path. No extension behavior is reimplemented here |
| `KMIPKIT-DISC-022`; §9.16, Table 421; ADR-0002 | FR-007, FR-011 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0007's KMIP 2.1-only policy is inherited as a documented scope exception, not a claim that §9.16 requires same-major version rejection |
| `KMIPKIT-REQ-SPEC-9.20-001-001`, `KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP`; §9.20, Table 425 | FR-007 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0006/0007 caller-value passthrough/omission policy; existing timestamp tests plus operation tests use the shared writer. Countdown-derived timestamps under `9.20-001-002` remain excluded under OD-004 |
| `KMIPKIT-REQ-SPEC-9.21-001-001/-002`; §9.21, Table 426 | FR-007 | T009–T011, T015–T017, T021–T023 | KMIPKIT-0006/0007 correlation and response matching; each operation execution test covers matching and mismatched operation/batch identity |

## Project security and diagnostic obligations

| Stable project obligation | Feature requirements | Planned executable evidence |
|---|---|---|
| KMIPKIT-0007-FR-005/013, KMIPKIT-0009-FR-002, KMIPKIT-0007 OD-006; ADR-0014; `AGENTS.md` §8 | FR-009 | T003–T005, T009–T011, T015–T017, T021–T023 | Protocol tests prove attribute-group Debug redaction with a byte sentinel; each operation-specific Client::execute test proves request-owner lifetime through simulated partial writes and initialized-range zeroization before release after success and post-write error, while secret/raw-body sentinels remain absent from Debug, Display, errors, and logs |
| `AGENTS.md` §§4, 5, 10; constitution I–II | FR-010, FR-011 | T001–T002, T026–T027 | T001 checks exact pinned clauses and the pinned OASIS Test Cases work product; upstream and local fixture bytes are each 2,725 bytes with the same recorded SHA-256. T002 assigns only the three scoped operation elements and five linked §6.1.9/§6.1.10 requirements to KMIPKIT-0014, maps `TC-CREATE-SD-1-21` to the fixture and Secret Data object, and marks availability without claiming conformance. The Create-only derived test uses the Create request/response item. The immutable-source checker protects exact `upstream/` files while permitting only README/SOURCES modifications to paths tracked in the base and additions under `fixtures/`; focused tests reject other OASIS paths, inventory add/delete/rename, and changes to existing fixtures. Both checker Red/Green/Refactor cycles are recorded in `research.md`. T026 validation also ensures source/hash validation and unrelated generated artifacts remain unchanged |

## Success-criteria coverage

| Success criterion | Tasks | Planned evidence |
|---|---|---|
| SC-001 | T006–T008, T012–T014, T018–T020 | Table-derived protocol roundtrips cover operation fields, optionals, and repeated values |
| SC-002 | T003, T006–T008, T012–T014, T018–T020 | Negative tests cover absent, duplicate, malformed, and wrong-type fields; generic TTLV retains permitted unknown values |
| SC-003 | T012–T014 | Table 191 cases and grouped/repeated Attribute preservation |
| SC-004 | T018–T020 | Multiple Table 194 identifiers survive decoding in original wire order |
| SC-005 | T001, T025–T028 | Normative source, stable feature requirement, implementation, named test, convergence, and independent QA/security review evidence are linked before review |
| SC-006 | T009–T011, T015–T017, T021–T023 | Fake transport verifies single exchange, delivery state, Pending, redaction, owner lifetime through partial writes, zeroization, and no retry for all three operations |
| SC-007 | T021–T023 | Split-key execution verifies the peer-visible maximum, exact-limit acceptance, and one-byte-over rejection before decoder entry |
| SC-008 | T024–T026 | English and Spanish guides contain Rust examples for all three operations; `scripts/test_user_guide_examples.py` compiles every marked example in both guides |

## Stable normative requirement inventory

- `KMIPKIT-REQ-SPEC-6.1.9-001-001`: Common Attributes MAY contain values shared by both keys.
- `KMIPKIT-REQ-SPEC-6.1.9-001-002`: Private Key Attributes and Public Key Attributes MAY contain key-specific values.
- `KMIPKIT-REQ-SPEC-6.1.9-006` and `-007`: Table 191 requires equal values for Cryptographic Algorithm, Cryptographic Length, Cryptographic Domain Parameters, and Cryptographic Parameters.
- `KMIPKIT-REQ-SPEC-6.1.10-001`: Create Split Key MAY include the Unique Identifier of an existing object to split.
- `KMIPKIT-REQ-SPEC-9.12-001-001/-002/-003`: Maximum Response Size is optional; a client that declares it SHALL handle that size; it SHOULD be sent primarily for requests that may return large replies. KMIPKit classifies Create Split Key as potentially large because Table 194 permits repeated identifiers and applies its local response cap as the advertised maximum.

Table-required fields, types, presence, multiplicity, response payloads, and operation-specific errors without separate requirement IDs are mapped to the stable feature requirements in the operation table above.

`KMIPKIT-REQ-SPEC-2.8-003` and `KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-PRIME-FIELD-SIZE` concern the Split Key object in §2.8/Table 9; they are not Create Split Key request-field requirements. Table 193 marks request Prime Field Size optional. FR-015 is a stricter KMIPKit client policy and is identified as such above.

## Official OASIS Test Case status

The pinned `kmip-testcases-v2.1-cn01.html` §2.12 lists `TC-CREATE-SD-1-21`. Its linked XML at `https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/test-cases/kmip-v2.1/TC-CREATE-SD-1-21.xml` is stored at `specification/oasis/kmip-2.1/fixtures/TC-CREATE-SD-1-21.xml`; OASIS identifies the work product as Version 2.1 Committee Note 01 dated 07 May 2020. The downloaded source and local fixture are each 2,725 bytes and byte-identical with SHA-256 `882e0f57ff2cc42b214e2ffb40bf9c80105489aab00f07a6ea5e2c1c395474ad`, recorded with provenance in `SOURCES.md`; the catalog maps the fixture as available. The XML contains Create and Get request/response batch items. Tests may derive only the Create request/response item and MUST NOT claim the complete two-operation official test case passes. The pinned work product contains no direct Create Key Pair or Create Split Key case. Other table-derived coverage remains distinct from official OASIS test-case evidence.

## Explicit exclusions

- The 1.0 client version restriction remains `KMIPKIT-DISC-022` under ADR-0002.
- No client obligation is inferred from server-only clauses, including `KMIPKIT-REQ-SPEC-9.8-001-002/-003`.
- No countdown-derived outgoing Time Stamp behavior is added; `KMIPKIT-REQ-SPEC-9.20-001-002` remains excluded under KMIPKIT-0007 OD-004.
- Official case `TC-CREATE-SD-1-21` has a locally available XML fixture, but only its Create request/response item is within feature scope; this derived test is not evidence that the complete case passes or establishes full Create conformance. The other operation cases remain table-derived.
