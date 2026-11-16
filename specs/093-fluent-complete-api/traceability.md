# KMIPKIT-0093 Traceability

| Requirement | Pinned source and existing contract | Verification |
|---|---|---|
| FR-001–003 catalog | KMIP 2.1 §11.36 Table 470, §11.56 Table 487; KMIP 3.0 §11.37 Table 584, §11.58 Table 605; KMIPKIT-0047, -0069, -0071 | Exhaustive assigned-value and version/direction matrix; generator snapshot and negative unassigned-value tests |
| FR-004 operation wrappers | KMIP 2.1 §6.1 operation tables; KMIPKIT-0007, -0051, -0052, -0091 | Per-operation build, validation, batch conversion, and real TLS fixture tests. The Create Key Pair builder is wired through the typed Rust request path and exposed in C, Java, and Python; all four consumers exercise Create Key Pair, Sign, and Destroy through the controlled TLS fixture. Other operation wrappers remain tracked by their feature tasks. |
| FR-005 typed result conversion | KMIP 2.1 §8.6 Table 417 and §9.19; KMIPKIT-0007, -0014, -0043 | `ClientBatchOutcome` operation/result/response mappings; Success, Operation Pending, Operation Failed, unknown status, reason, and correlation tests; transport/protocol errors retain `RequestDeliveryState` |
| FR-006 expert attributes | KMIP 2.1 §§4–5, §7.1, §11.56; KMIPKIT-0048, ADR-0012, ADR-0014 | Wrong type, unknown tag, vendor structure, order, version, reserved tag, redaction, and no-send negative tests |
| FR-007 caller choices | KMIP 2.1 relevant §6.1 operation tables; ADR-0022; KMIPKIT-0091 | Exact preset value assertions; missing crypto decision and no-retry tests |
| FR-008 typed response fields | KMIP 2.1 §§8.4–8.6, §9.19, Table 417; operation-specific KMIP 2.1 §6.1 response tables; ADR-0018 response ownership; KMIPKIT-0052 | Typed result accessors preserve wire-presence semantics, status/reason/message, operation-specific fields, and Pending correlation; Debug and error redaction tests |
| FR-009 retained response encodings | KMIP 2.1 §§8.4–8.6; Profiles v2.1 §§5.4–5.5; ADR-0018 encoding and response ownership; KMIPKIT-0054–0057, -0095 | Shared checked TTLV hex/JSON/XML renderers preserve response order and unknown allowed values, enforce caller codec limits, and return payload-free errors; C/Java/Python owner and caller-copy tests |
| FR-010 inspection and diagnostics | KMIP 2.1 §§8–9; ADR-0018 encoding and response ownership; KMIPKIT-0007 | Explicit inspection only, redacted default formatting, and proof that encoding failures do not replace the typed operation result |
| FR-011 four languages | KMIPKIT-0043, -0053; Constitution III | C/JNI/CFFI ownership, ABI audit, Java/Python typing, compiled consumers and parity matrix. Create Key Pair builders and typed response handling are exercised in compiled C, Java/JNI, Python/CFFI, and Rust consumers; the complete API parity matrix and global closeout gates remain open. |
| FR-012 server direction | KMIP 2.1 §6.2, KMIPKIT-0066, ADR-0021 | Listener handler fixtures; no client send for server-only operation |
| FR-013 journeys | KMIP 2.1 §§6.1.1, 6.1.19, 6.1.21, 6.1.28 and Profiles §§5.3–5.5 | Rust, C, Java, and Python execute all five progressive journeys against the controlled TLS fixture: symmetric-key lifecycle, explicit AES-GCM operations, ECDSA key-pair/signing, Vendor Attribute Create/Get/Destroy, and an ordered Create/Get batch where omitted Get Unique Identifier resolves the preceding ID Placeholder. |

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
