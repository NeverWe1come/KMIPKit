# KMIPKIT-0050 Requirement Traceability

| Requirement | Normative/project source | Planned implementation | Verification |
|---|---|---|---|
| KMIPKIT-0050-FR-001 | KMIP v2.1 §4.16 Table 59; §11.56; catalog Cryptographic Parameters and Table 59 member elements | `crates/kmipkit-protocol/src/cryptographic_parameters.rs`; public export in `src/lib.rs` | New unit tests for each Table 59 member and raw enumeration preservation |
| KMIPKIT-0050-FR-002 | KMIP v2.1 §10.1.2; project API compatibility §9 of `AGENTS.md` | Typed builder and lossless generic TTLV bridge | Setter permutation canonical-order test; unknown/vendor member round trip |
| KMIPKIT-0050-FR-003 | KMIP v2.1 §4.16 Table 59 and §10.1.2 | Typed parser and payload-free validation error | Negative tests for wrong types, duplicate singleton, known-member order |
| KMIPKIT-0050-FR-004 | Existing operation specs; project additive API policy | Existing setters in Encrypt, Decrypt, Derive Key, Sign, Signature Verify, MAC, MAC Verify, Encryption Key Information, and MAC Signature Key Information | Existing generic `Structure` consumers compile unchanged; typed-model conversion tests |
| KMIPKIT-0050-FR-005 | KMIPKIT-REQ-SPEC-4.16-002 / §4.16; KMIPKIT-REQ-SPEC-4.16-003 / §4.16; §11.6 Table 436 | Existing `cryptographic_parameters.rs` validation reused by typed model | CTR/CCM/GCM required-value tests; operation omission and unknown mode preservation |
| KMIPKIT-0050-FR-006 | KMIP v2.1 §8.2 Tables 395–396; §9.4; direct maintainer approval in `approval-record.md` | `ClientBatch` optional Authentication field and builder | Unit and fake-transport tests for absent and explicitly supplied values |
| KMIPKIT-0050-FR-007 | KMIP v2.1 §§8.1–8.3 Tables 394–396; project decision in `approval-record.md` | Request Header writer in `crates/kmipkit-client/src/execute.rs` | Captured single- and multi-item request checks exact single header Authentication and shared message scope |
| KMIPKIT-0050-FR-008 | `KMIPKIT-REQ-SPEC-9.4-001-001`, `KMIPKIT-REQ-SPEC-9.4-001-002`, and `KMIPKIT-REQ-SPEC-9.4-002` / §9.4 | Typed Authentication validation during construction; Request Header construction in the client | Empty/malformed generic Authentication is rejected before attachment; repeated Credential order is captured; client preflight and transport failures retain the required delivery state |
| KMIPKIT-0050-FR-009 | Project secret policy `AGENTS.md` §8; approved credential ownership contracts in `specs/008-credentials-attestation` | `Credential`, `Authentication`, transient request tree/buffer ownership paths | Canary redaction, clone/drop/zeroization tests; no body data in errors |
| KMIPKIT-0050-FR-010 | The 56 client-to-server `KMIPKIT-ELEM-OP-C2S-*` elements listed in `gap-matrix.md`; asynchronous §6.1.42–§6.1.45; approved direct API plan | `Client` direct method inventory in `crates/kmipkit-client/src/execute.rs` | Compile-time method matrix for all 56 operations and all `_with_options` methods |
| KMIPKIT-0050-FR-011 | Existing operation specifications; KMIP v2.1 async §§6.1.42–6.1.45 | Direct wrappers delegate to common single-item typed execution path | Fake-transport parity for response, delivery state, options, Pending/correlation, and one exchange |
| KMIPKIT-0050-FR-012 | Product boundary `AGENTS.md` §2; project API policy §9 | No transport/config changes; preserve existing API | Compile old generic call sites and negative UI tests for disallowed transport access |
| KMIPKIT-0050-FR-013 | KMIP v2.1 source hierarchy and project traceability policy `AGENTS.md` §5 | This file and `implementation-evidence.md` | Traceability checker verifies every FR has exact source and executable test references |

## Table 59 stable catalog elements

The following stable element IDs are in exact OASIS KMIP v2.1 §4.16 Table 59
order. Each maps to source section `4.16` in the reviewed catalog.

| Order | Stable element ID | Member |
|---:|---|---|
| 1 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-BLOCK-CIPHER-MODE` | Block Cipher Mode |
| 2 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-PADDING-METHOD` | Padding Method |
| 3 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-HASHING-ALGORITHM` | Hashing Algorithm |
| 4 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-KEY-ROLE-TYPE` | Key Role Type |
| 5 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-DIGITAL-SIGNATURE-ALGORITHM` | Digital Signature Algorithm |
| 6 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-CRYPTOGRAPHIC-ALGORITHM` | Cryptographic Algorithm |
| 7 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-RANDOM-IV` | Random IV |
| 8 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-IV-LENGTH` | IV Length |
| 9 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TAG-LENGTH` | Tag Length |
| 10 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-FIXED-FIELD-LENGTH` | Fixed Field Length |
| 11 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INVOCATION-FIELD-LENGTH` | Invocation Field Length |
| 12 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-COUNTER-LENGTH` | Counter Length |
| 13 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INITIAL-COUNTER-VALUE` | Initial Counter Value |
| 14 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-SALT-LENGTH` | Salt Length |
| 15 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR` | Mask Generator |
| 16 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR-HASHING-ALGORITHM` | Mask Generator Hashing Algorithm |
| 17 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-P-SOURCE` | P Source |
| 18 | `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TRAILER-FIELD` | Trailer Field |

## Direct method coverage inventory

The reviewed current gaps are the 14 methods listed in `gap-matrix.md`. Four
async operations already expose direct methods: Poll, Cancel, Process, and Query
Async Requests. Ten existing methods need an additive `_with_options` form.
Implementation evidence will record the complete 56-row name-to-request-to-
response mapping and exact final method symbols.
