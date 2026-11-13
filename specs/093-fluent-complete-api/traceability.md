# KMIPKIT-0093 Traceability

| Requirement | Pinned source and existing contract | Verification |
|---|---|---|
| FR-001 operation metadata | KMIP 2.0 §§6.1–6.2 and §11.36; KMIP 2.1 §§6.1–6.2 and §11.36 Table 470; KMIP 3.0 Working Draft (13 July 2026) §6.1 and §11.37 Table 584; KMIPKIT-0047, -0069, -0071 | `specification/catalog/operation-matrix-evidence.json` supplies reviewed directional source, typed model and route references. The broader attribute/field catalog remains owned by its respective slices. |
| FR-002 reproducible matrix generation | Existing catalog and `specification/api/public-api.json`; existing four-language symbol generators remain authoritative | `tools/normative_catalog/generate_fluent_operation_inventory.py --check`; `test_generated_inventory_is_deterministic_and_current`; public manifest generation `--check`. No public symbols are added or renamed by the matrix. |
| FR-003 versioned operation matrix | Exact edition/section/path per row in `specification/catalog/fluent-operation-inventory.json`; pinned KMIP 2.0/2.1/3.0 sources | `tools/normative_catalog/tests/test_fluent_operation_inventory.py`: exhaustive 66-code/69-directional-row reconciliation, directional earliest-version clauses, typed owner/route/test references, explicit unverified-route behavior, reserved-value and fabricated-evidence rejection. `testEvidence.focused` records exact existing executable tests per row; static references do not claim passing execution, route-version conformance or server capability. |
| FR-004 operation wrappers | KMIP 2.1 §6.1 operation tables; KMIPKIT-0007, -0051, -0052, -0091 | Per-operation build, validation, batch conversion, and real TLS fixture tests |
| FR-005 typed result conversion | KMIP 2.1 §8.6 Table 417 and §9.19; KMIPKIT-0007, -0014, -0043 | `ClientBatchOutcome` operation/result/response mappings; Success, Operation Pending, Operation Failed, unknown status, reason, and correlation tests; transport/protocol errors retain `RequestDeliveryState` |
| FR-006 expert attributes | KMIP 2.1 §§4–5, §7.1, §11.56; KMIPKIT-0048, ADR-0012, ADR-0014 | Wrong type, unknown tag, vendor structure, order, version, reserved tag, redaction, and no-send negative tests |
| FR-007 caller choices | KMIP 2.1 relevant §6.1 operation tables; ADR-0022; KMIPKIT-0091 | Exact preset value assertions; missing crypto decision and no-retry tests |
| FR-008 typed response fields | KMIP 2.1 §§8.4–8.6, §9.19, Table 417; operation-specific KMIP 2.1 §6.1 response tables; ADR-0018 response ownership; KMIPKIT-0052 | Typed result accessors preserve wire-presence semantics, status/reason/message, operation-specific fields, and Pending correlation; Debug and error redaction tests |
| FR-009 retained response encodings | KMIP 2.1 §§8.4–8.6; Profiles v2.1 §§5.4–5.5; ADR-0018 encoding and response ownership; KMIPKIT-0054–0057, -0095 | Shared checked TTLV hex/JSON/XML renderers preserve response order and unknown allowed values, enforce caller codec limits, and return payload-free errors; C/Java/Python owner and caller-copy tests |
| FR-010 inspection and diagnostics | KMIP 2.1 §§8–9; ADR-0018 encoding and response ownership; KMIPKIT-0007 | Explicit inspection only, redacted default formatting, and proof that encoding failures do not replace the typed operation result |
| FR-011 four languages | KMIPKIT-0043, -0053; Constitution III | C/JNI/CFFI ownership, ABI audit, Java/Python typing, and parity matrix; the Rust consumer imports `kmipkit::CryptographicUsageMask` from the public umbrella crate in `tests/consumers/0093-journeys/rust/src/main.rs` |
| FR-012 server direction | KMIP 2.1 §6.2, KMIPKIT-0066, ADR-0021 | Listener handler fixtures; no client send for server-only operation |
| FR-013 journeys | KMIP 2.1 §§4.35, 6.1, 6.1.2, 11.45, 11.58 Table 489; Profiles §§5.3–5.5 | F001.1: the four consumers in `tests/consumers/0093-journeys/` perform typed Create → Get → Destroy. F001.2: they create an AES-256 key and execute explicit AES-GCM Encrypt/Decrypt with fixture-checked IV, AAD, tag, ciphertext and plaintext. F001.3: they create ECDSA P-256 key pairs, sign and verify fixture bytes using the returned private/public identifiers and explicit SHA-256 parameters. F001.4: they submit the catalogued Object Group Text String on Create and then Destroy the returned identifier; each consumer rejects an Integer Object Group locally before transport. F001.5: each consumer sends an ordered two-item Create → Get batch; Get uses ID Placeholder Enumeration 1, both requests carry distinct Unique Batch Item IDs, and the encoded responses echo the corresponding IDs and server-assigned identifier. `tests/consumers/0093-journeys/local_journey_responder.py` validates the decoded request fields and emits encoded TTLV responses; `test_local_journey_responder.py` checks the batch order, count, placeholder, IDs, and response field order. `verify.sh` runs all five journeys for all four languages over TLS 1.3/mTLS and requires all 52 connections to complete. |

Task 3 review evidence explicitly checks Recommended Curve P-256 inside typed
Cryptographic Domain Parameters in Common Attributes (KMIP 2.1 sections 4.14,
6.1.2 and 11.45). `tests/consumers/0093-journeys/test_local_journey_responder.py`
rejects absent, empty, wrong-type and secp256k1 curve selections. The Java Sign
and Signature Verify consumers close their native builders with
try-with-resources, including failures during execution or response inspection.

The generated operation matrix now records one row per assigned operation and
direction. Full field-level and normative client requirement coverage remains
owned by the corresponding implementation slices. A static count or test
reference alone does not establish execution success or conformance.

## Generic Rust Create builder

`RequestBuilder::create` and `CreateBuilder` implement the Rust construction
slice of FR-004, FR-005, FR-006, FR-007, and FR-011 over `CreateRequest` and
`ClientRequest::Create` (OASIS KMIP v2.1 §6.1.8, Tables 186–187).
`fluent_request_tests::generic_create_builder_*` verifies the explicit Object
Type, prebuilt ordered attributes and optional Protection Storage Masks,
empty default attributes without cryptographic choices, unknown-value
preservation, existing cardinality validation, sanitized `NotSent` errors
with zero exchanges, batch conversion, explicit exchange options and
Asynchronous Indicator, the default KMIP 2.1 header, and Success/Pending
results with exactly one exchange. Local tests use the existing scripted
transport and production typed executor; this slice alone does not establish
four-language parity, external interoperability, or profile conformance.

`execute_boundary_tests::generic_create_masks_exception_accepts_only_its_exact_typed_setter`
checks that the existing AST boundary permits only the operation-specific
`CreateBuilder::protection_storage_masks(Structure) -> Self` signature;
wrong inputs, outputs, scope, builder names, and execution methods remain
rejected.
