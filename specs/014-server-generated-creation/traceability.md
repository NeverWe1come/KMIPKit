# Normative Traceability: Server-Generated Object Creation

**Feature**: KMIPKIT-0014

**Status**: Draft mappings; implementation symbols and executable test names are finalized in the implementation PR.

**Normative source**: OASIS KMIP Specification v2.1 pinned at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.

## Operation-specific clauses

| OASIS source | Catalog element / stable requirement / official test case | Feature requirements | Tasks | Planned implementation and executable evidence |
|---|---|---|---|---|
| §6.1.8, Tables 186–187; §6.1.8.1, Table 188 | `KMIPKIT-CLAUSE-SPEC-6.1.8-001`, `KMIPKIT-CLAUSE-SPEC-6.1.8.1-001`, `KMIPKIT-ELEM-OP-C2S-CREATE`, `TC-CREATE-SD-1-21` §2.12 | FR-001, FR-002, FR-006, FR-010 | T001, T006–T011 | `crates/kmipkit-protocol/src/create.rs`; `create_tests.rs` covers a Create-only TTLV derivation from the pinned official XML and table-derived field/cardinality/error cases; it does not claim full `TC-CREATE-SD-1-21` coverage because the source case also contains an out-of-scope Get item; `create_execution_tests.rs` covers success, Failure, association, and one exchange |
| §6.1.9, Tables 189–190; §6.1.9.1, Table 192 | `KMIPKIT-CLAUSE-SPEC-6.1.9-001/-002/-004`, `KMIPKIT-CLAUSE-SPEC-6.1.9.1-001`, `KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR`, `KMIPKIT-REQ-SPEC-6.1.9-001-001/-002` | FR-001, FR-003, FR-006, FR-013 | T012–T017 | `crates/kmipkit-protocol/src/create_key_pair.rs`; `create_key_pair_tests.rs` for grouped/repeated attributes, Table 189 union preservation, and every Table 192 Result Reason; `create_key_pair_execution_tests.rs` for both IDs, failures, batching, and one exchange |
| §6.1.9, Table 191 plus §6.1.9 key-specific precedence | `KMIPKIT-CLAUSE-SPEC-6.1.9-006/-007`, `KMIPKIT-REQ-SPEC-6.1.9-006/-007` | FR-004 | T012–T014 | `crates/kmipkit-protocol/src/create_key_pair.rs`; `create_key_pair_tests.rs` covers all four attributes, both effective values absent, Common fallback, one/both key-specific overrides, equal overrides differing from Common, one-sided effective values, and differing effective values |
| §6.1.10, Tables 193–194; §6.1.10.1, Table 195 | `KMIPKIT-CLAUSE-SPEC-6.1.10-001/-002/-005`, `KMIPKIT-CLAUSE-SPEC-6.1.10.1-001`, `KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY` | FR-001, FR-005, FR-006, FR-015 | T018–T023 | `crates/kmipkit-protocol/src/create_split_key.rs`; `create_split_key_tests.rs` for every field, conditional Prime Field Size presence, optional input ID, repeated response IDs, errors, cardinality, and no inferred split parameters; execution tests cover IDs, failures, Pending, and one exchange |
| §2.8, Split Key Parameter requirements; §6.1.10 Table 193 | `KMIPKIT-REQ-SPEC-2.8-003` | FR-015 | T001, T018–T020 | `crates/kmipkit-protocol/src/create_split_key.rs`; tests require caller-supplied Prime Field Size for Polynomial Sharing Prime Field and verify other method conditions without synthesizing values |
| §§7.1–7.2, operation payload and Attribute structures, especially §7.2 Table 150 | Operation element IDs above; table fields without standalone requirement IDs remain obligations under FR-002/003/005/010/014 | FR-001, FR-002, FR-003, FR-005, FR-010, FR-014 | T003–T008, T012–T020 | Three operation modules plus `crates/kmipkit-protocol/src/attribute.rs`; tests verify Table 150 Attribute Name Text String, tagged generic Attribute Value Item, presence, cardinality, order, repeated entries, and unknown names/vendor tags |
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
| KMIPKIT-0007-FR-005/013, KMIPKIT-0009-FR-002, KMIPKIT-0007 OD-006; ADR-0014; `AGENTS.md` §8 | FR-009 | T003–T005, T009–T011, T015–T017, T021–T023 | Protocol tests prove AttributeEntry Debug redaction with a byte sentinel; each operation-specific Client::execute test proves request-owner lifetime through simulated partial writes and initialized-range zeroization before release after success and post-write error, while secret/raw-body sentinels remain absent from Debug, Display, errors, and logs |
| `AGENTS.md` §§4, 5, 10; constitution I–II | FR-010, FR-011 | T001–T002, T026–T027 | T001 checks exact pinned clauses and the pinned OASIS Test Cases work product. T002 updates only checked-in catalog input and regenerates its report. Validation checks ensure upstream OASIS files and unrelated generated artifacts are unchanged |

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

## Stable normative requirement inventory

- `KMIPKIT-REQ-SPEC-6.1.9-001-001`: Common Attributes MAY contain values shared by both keys.
- `KMIPKIT-REQ-SPEC-6.1.9-001-002`: Private Key Attributes and Public Key Attributes MAY contain key-specific values.
- `KMIPKIT-REQ-SPEC-6.1.9-006` and `-007`: Table 191 requires equal values for Cryptographic Algorithm, Cryptographic Length, Cryptographic Domain Parameters, and Cryptographic Parameters.
- `KMIPKIT-REQ-SPEC-6.1.10-001`: Create Split Key MAY include the Unique Identifier of an existing object to split.
- `KMIPKIT-REQ-SPEC-9.12-001-001/-002/-003`: Maximum Response Size is optional; a client that declares it SHALL handle that size; it SHOULD be sent primarily for requests that may return large replies. KMIPKit classifies Create Split Key as potentially large because Table 194 permits repeated identifiers and applies its local response cap as the advertised maximum.

Table-required fields, types, presence, multiplicity, response payloads, and operation-specific errors without separate requirement IDs are mapped to the stable feature requirements in the operation table above.

## Official OASIS Test Case status

The pinned `kmip-testcases-v2.1-cn01.html` §2.12 lists `TC-CREATE-SD-1-21`. Its linked XML at `https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/test-cases/kmip-v2.1/TC-CREATE-SD-1-21.xml` contains a Create/Secret Data item and a subsequent Get item. T001 verifies and pins the byte-identical OASIS XML outside `upstream/`, with source provenance and checksum; implementation tests may derive only the Create TTLV item from it and MUST NOT claim the complete two-operation test case passes. The pinned work product contains no direct Create Key Pair or Create Split Key case. Other table-derived coverage remains distinct from official OASIS test-case evidence.

## Explicit exclusions

- The 1.0 client version restriction remains `KMIPKIT-DISC-022` under ADR-0002.
- No client obligation is inferred from server-only clauses, including `KMIPKIT-REQ-SPEC-9.8-001-002/-003`.
- No countdown-derived outgoing Time Stamp behavior is added; `KMIPKIT-REQ-SPEC-9.20-001-002` remains excluded under KMIPKIT-0007 OD-004.
- Official case `TC-CREATE-SD-1-21` is limited to its Secret Data scenario and is not evidence of full Create conformance; the other operation cases remain table-derived.
