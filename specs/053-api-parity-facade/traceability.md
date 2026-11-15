# KMIPKIT-0053 Traceability

Exact source tables are inherited from specs 048–052 and reference the pinned OASIS KMIP 2.1 Standard. Project API choices do not create new OASIS requirements.

| ID | OASIS source | Requirement | Implementation | Verification |
|---|---|---|---|---|
| KMIPKIT-0053-R001 | Attribute tables in specs/048-typed-attributes/traceability.md; §10.1.2 | Preserve direct attribute tag, TTLV value alternative, order, cardinality, and local constraints. | C ABI, Java JNI, Python CFFI over Rust definitions. | 62-definition matrix, invalid tag/type and generic fallback tests. |
| KMIPKIT-0053-R002 | Object/key/wrapping tables in specs/049-managed-objects/traceability.md; §10.1.2 | Preserve object tags, field order/types and unknown children. | Opaque C handles and Java/Python owners over Rust. | Nine-object/key roundtrips and order checks. |
| KMIPKIT-0053-R003 | §4.16 Table 59 and affected §6.1 payload tables in specs/050-parameters-auth-client/traceability.md | Preserve Cryptographic Parameters types/order and future raw enum values. | C ABI/JNI/CFFI over shared Rust model. | All 18 fields, future values and extensions. |
| KMIPKIT-0053-R004 | §8.2 Table 395; §§8.1, 8.3 Tables 394, 396 | Authentication appears once in the Request Header for the message and its batch items. | Foreign batch setters call Rust request builder. | Encoded absent/present one-/multi-item tests. |
| KMIPKIT-0053-R005 | §6.1 operation payloads; §§8.1–8.3; §10.1.2; specs/051-pre-send-validation/traceability.md | Reject locally decidable normative violations before I/O; never guess generic schemas; omit values in reports. | All adapters use Rust validation. | Issue parity, NotSent, zero writes, valid-wire stability, redaction. |
| KMIPKIT-0053-R006 | §6.1 tables for 56 supported operations and async rules in specs/050-parameters-auth-client/gap-matrix.md | Preserve operation identity, request/options, response association and async correlation. | C ABI/JNI/CFFI wrappers. | 56-operation matrix and no-retry tests. |
| KMIPKIT-0053-R007 | §8.5 Table 398; §8.6 Table 399; §§9.13, 10.1.2; §11.56; specs/052-response-navigation/traceability.md | Preserve response presence, order, repetitions, unknown/vendor values at arbitrary depth. | KMIPKIT-0052 handles and wrappers. | Absent/empty/Pending and nested 0x54xxxx tests in four languages. |
| KMIPKIT-0053-R008 | §11.36 Table 470; KMIPKIT-0053-FR-007 | Every typed request builder exposes the exact KMIP Operation Enumeration it will send. | `ClientBatchItem::operation`, C ABI, Java accessor, and Python property. | C and Java compare every synchronous builder with the generated OASIS Operation values; Python verifies the live ABI result. |
| KMIPKIT-0053-R009 | §11.25 Table 461; §§3.11–3.12 Tables 25–26 | Dispatch Transparent EC Private/Public Key material only for Key Format Type 0x14/0x15; other raw values remain generic. | `key_material.rs::validate_key_material_item` and the C, Java, and Python typed readers. | Correct raw codes accept well-formed models and reject malformed EC Structures; reserved 0x0E/0x0F remain generic. |
| KMIPKIT-0053-R010 | §6.1.5 Table 177 (Cancel); §6.1.38 Table 276 and §8.6 Table 399 (Poll); §6.1.39 Table 279 (Process); §6.1.41 Table 286 (Query Asynchronous Requests; caption/body discrepancy retained per KMIPKIT-0052) | Each asynchronous response reader must match its KMIP Operation identity and expose the operation-specific fields while preserving generic raw TTLV access and owner lifetime. | Rust `ClientOperationOutcome` readers; C ABI v1 response readers; Java `AsyncOperationOutcome` readers; Python `AsyncOperationOutcome` readers, inventoried as `typed_async_response_readers`. | Compiled C consumer verifies all four operation identities, status fields, optional fields, and payload shape; Java TLS integration verifies all four typed readers and cross-operation absence; Python checks typed fields and mismatch absence; manifest parity regression requires exact reader and field mappings. |

## Project Decisions
- P001: Rust remains the sole protocol/model/validation implementation.
- P002: Idiomatic language naming is allowed only with exact mappings and runnable tests.
- P003: Generic TTLV remains the lossless route for unknown/future/vendor data.
- P004: Authentication is caller-owned, one-message scoped, never client-global.
- P005: Foreign owners have explicit close/release; only KMIPKit-owned copies are promised zeroization.
- P006: Generated outputs are written only by the pinned generator.
- P007: Rust uses typed validation schema enums while C, Java, and Python use explicit schema identifiers; all names accepted by the Rust model are kept in exact parser parity, including the explicitly unsupported `SetEndpointRole` descriptor.

## Verification Record
- Windows x86_64: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features --locked` passed.
- Windows x86_64 Rust line coverage: `cargo llvm-cov --workspace --all-features --locked --json --output-path <temp>/kmipkit-0053-final-coverage.json` passed after `python scripts/coverage_gate.py preflight --workspace .`. The normalized report contains all 140 eligible production files; workspace 93.50%, TTLV 99.06%, protocol 96.13%, client 97.43%, transport 95.16%, FFI 86.70%, and test-support 95.84%.
- `python -B tools/api_manifest/reconcile_0053_api_parity.py --write` is idempotent and reports `+0 functions`; `python -B tools/api_manifest/generate.py --check` verifies all 12 generated outputs. The full API route reconciliation and Encrypt/Decrypt parity tests pass.
- The Reconciler, binding export audits, full Rust workspace, Java suite, Python suite, and compiled Rust/C/Java/Python package consumers passed locally as listed in `implementation-evidence.md`.
- The all-platform and binding coverage aggregation remains outstanding. The maintainer instructed us to ignore hosted GitHub CI while billing is unavailable. No Linux/macOS, two-server interoperability, or qualified human security review result is claimed.
