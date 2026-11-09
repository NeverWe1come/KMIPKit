# KMIPKIT-0050 Requirement Traceability

| Requirement | Normative/project source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0050-FR-001 | KMIP v2.1 §4.16 Table 59; §11.56; catalog Cryptographic Parameters and Table 59 member elements | `crates/kmipkit-protocol/src/cryptographic_parameters.rs`; exports from `crates/kmipkit-protocol/src/lib.rs` and the facade | `crates/kmipkit-protocol/tests/unit/cryptographic_parameters_tests.rs` covers all members and raw enumeration preservation |
| KMIPKIT-0050-FR-002 | KMIP v2.1 §10.1.2; project API compatibility §9 of `AGENTS.md` | `CryptographicParametersBuilder`, lossless TTLV bridge, ordered member schema in `crates/kmipkit-protocol/src/cryptographic_parameters.rs` | Builder permutation/order and unknown/vendor-member round trips in `cryptographic_parameters_tests.rs` |
| KMIPKIT-0050-FR-003 | KMIP v2.1 §4.16 Table 59 and §10.1.2 | Typed parser and payload-free validation in `cryptographic_parameters.rs` | Wrong types, duplicate singleton members, member ordering, and malformed TTLV in `cryptographic_parameters_tests.rs` |
| KMIPKIT-0050-FR-004 | Existing operation specs; project additive API policy | Additive typed setters in the applicable protocol request and key-information models | `crates/kmipkit-protocol/tests/cryptographic_parameters_typed_setters.rs`; existing generic `Structure` call sites remain compiled in the protocol suite |
| KMIPKIT-0050-FR-005 | KMIPKIT-REQ-SPEC-4.16-002 / §4.16; KMIPKIT-REQ-SPEC-4.16-003 / §4.16; §11.6 Table 436 | Shared existing IV Length and Tag Length validation is used by typed model and operation paths | CTR/CCM/GCM conditions, omission behavior, and unknown mode preservation in `cryptographic_parameters_tests.rs` |
| KMIPKIT-0050-FR-006 | KMIP v2.1 §8.2 Tables 395–396; §9.4; direct maintainer approval in `approval-record.md` | Optional one-shot Authentication on `ClientBatch` in `crates/kmipkit-client/src/execute.rs` | `crates/kmipkit-client/tests/unit/request_authentication_tests.rs` checks absence, presence, one-item and multi-item messages |
| KMIPKIT-0050-FR-007 | KMIP v2.1 §§8.1–8.3 Tables 394–396; project decision in `approval-record.md` | Request Header builder in `crates/kmipkit-client/src/execute.rs` | Captured TTLV checks in `request_authentication_tests.rs` verify one header field, Credential order, and batch-wide scope |
| KMIPKIT-0050-FR-008 | `KMIPKIT-REQ-SPEC-9.4-001-001`, `KMIPKIT-REQ-SPEC-9.4-001-002`, and `KMIPKIT-REQ-SPEC-9.4-002` / §9.4 | Typed Authentication parsing and preflight validation in protocol/client request construction | `request_authentication_tests.rs` rejects malformed/empty values before exchange and checks delivery states |
| KMIPKIT-0050-FR-009 | Project secret policy `AGENTS.md` §8; approved credential ownership contracts in `specs/008-credentials-attestation` | Existing secret wrappers plus owned transient TTLV/wire buffers; no client-global credential storage | Credential redaction and lifecycle coverage in `request_authentication_tests.rs`, `execute_lifecycle_tests.rs`, and protocol credential-secret tests |
| KMIPKIT-0050-FR-010 | The 56 client-to-server `KMIPKIT-ELEM-OP-C2S-*` elements listed in `gap-matrix.md`; asynchronous §6.1.42–§6.1.45; approved direct API plan | Complete `Client` operation methods in `crates/kmipkit-client/src/execute.rs` | `direct_method_surface_tests.rs` type-checks all 56 methods, `_with_options` variants, and asynchronous option forms |
| KMIPKIT-0050-FR-011 | Existing operation specifications; KMIP v2.1 async §§6.1.42–6.1.45 | Direct wrappers use the common typed one-item `ClientBatch` execution path | `direct_method_behavior_tests.rs` compares all 14 new request encodings with their batch equivalents and checks delivery and timeout-option propagation; async result behavior remains covered by the operation-specific lifecycle tests |
| KMIPKIT-0050-FR-012 | Product boundary `AGENTS.md` §2; project API policy §9 | Existing generic APIs remain source-compatible; no transport/configuration surface is added | `direct_method_surface_tests.rs`, existing compatibility tests, and public export auditor |
| KMIPKIT-0050-FR-013 | KMIP v2.1 source hierarchy and project traceability policy `AGENTS.md` §5 | This file, `gap-matrix.md`, and `implementation-evidence.md` | Manifest generator and Rust export audit; generated parity fixtures include the final declaration set |
| KMIPKIT-0050-FR-014 | Pinned OASIS KMIP 3.0 Working Draft §6.1.15 Table 297; §6.1.22 Table 318; §11.58 Table 605 | `crates/kmipkit-protocol/src/kem.rs` (`EncapsulateResponse`, `DecapsulateResponse`, `DATA_TAG`) | `kem_tests::test_encapsulate_request_and_response` and `kem_tests::test_decapsulate_request_and_response` assert exact response tag order and absence of the Key Material tag |
| KMIPKIT-0050-FR-015 | Project secret-redaction policy in `AGENTS.md` §8; payload scope from KMIP 3.0 §6.1.15 Table 297 and §6.1.22 Table 318 | KEM response structs contain no shared-secret field and derive `Debug` only over permitted response values | The two KEM response tests assert Debug output has no `shared_secret` field or shared-secret bytes |
| KMIPKIT-0050-FR-016 | Pinned KMIP 3.0 Working Draft §6.1.9 Tables 277–278; §6.1.10 Tables 280–281; §6.1.13 Tables 290–291; §6.1.14 Tables 293–294; §6.1.15 Tables 296–297; §6.1.22 Tables 317–318; §6.1.41 Tables 374–375 | `crates/kmipkit-protocol/src/kmip30_operations.rs`; `crates/kmipkit-client/src/execute.rs` | `kmip30_operation_tests.rs`; `kmip30_operation_surface_tests.rs` checks each direct and option method |
| KMIPKIT-0050-FR-017 | Pinned KMIP 3.0 §11.37 Table 584; KMIPKIT-0071 VersionGate requirement | `crates/kmipkit-protocol/src/version_gate.rs`; `crates/kmipkit-client/src/execute.rs` | Exact seven-code mapping, `0x43` negative test, and fake transport confirms V2.0/V2.1 rejection is `NotSent` |

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
