# KMIPKIT-0093 Traceability

| Requirement | Pinned source and existing contract | Verification |
|---|---|---|
| FR-001–003 catalog | KMIP 2.1 §11.36 Table 470, §11.56 Table 487; KMIP 3.0 §11.37 Table 584, §11.58 Table 605; KMIPKIT-0047, -0069, -0071 | Exhaustive assigned-value and version/direction matrix; generator snapshot and negative unassigned-value tests |
| FR-004 operation wrappers | KMIP 2.1 §6.1 operation tables; KMIPKIT-0007, -0051, -0052, -0091 | Per-operation build, validation, batch conversion, and real TLS fixture tests |
| FR-005 typed result conversion | KMIP 2.1 §8.6 Table 417 and §9.19; KMIPKIT-0007, -0014, -0043 | `ClientBatchOutcome` operation/result/response mappings; Success, Operation Pending, Operation Failed, unknown status, reason, and correlation tests; transport/protocol errors retain `RequestDeliveryState` |
| FR-006 expert attributes | KMIP 2.1 §§4–5, §7.1, §11.56; KMIPKIT-0048, ADR-0012, ADR-0014 | Wrong type, unknown tag, vendor structure, order, version, reserved tag, redaction, and no-send negative tests |
| FR-007 caller choices | KMIP 2.1 relevant §6.1 operation tables; ADR-0022; KMIPKIT-0091 | Exact preset value assertions; missing crypto decision and no-retry tests |
| FR-008 typed response fields | KMIP 2.1 §§8.4–8.6, §9.19, Table 417; operation-specific KMIP 2.1 §6.1 response tables; ADR-0018 response ownership; KMIPKIT-0052 | Typed result accessors preserve wire-presence semantics, status/reason/message, operation-specific fields, and Pending correlation; Debug and error redaction tests |
| FR-009 retained response encodings | KMIP 2.1 §§8.4–8.6; Profiles v2.1 §§5.4–5.5; ADR-0018 encoding and response ownership; KMIPKIT-0054–0057, -0095 | Shared checked TTLV hex/JSON/XML renderers preserve response order and unknown allowed values, enforce caller codec limits, and return payload-free errors; C/Java/Python owner and caller-copy tests |
| FR-010 inspection and diagnostics | KMIP 2.1 §§8–9; ADR-0018 encoding and response ownership; KMIPKIT-0007 | Explicit inspection only, redacted default formatting, and proof that encoding failures do not replace the typed operation result |
| FR-011 four languages | KMIPKIT-0043, -0053; Constitution III | C/JNI/CFFI ownership, ABI audit, Java/Python typing, and parity matrix; the Rust consumer imports `kmipkit::CryptographicUsageMask` from the public umbrella crate in `tests/consumers/0093-journeys/rust/src/main.rs` |
| FR-012 server direction | KMIP 2.1 §6.2, KMIPKIT-0066, ADR-0021 | Listener handler fixtures; no client send for server-only operation |
| FR-013 journeys | KMIP 2.1 operation tables and Profiles §§5.3–5.5 | F001 Task 1: the four consumers in `tests/consumers/0093-journeys/` perform typed Create → Get → Destroy over the controlled TLS 1.3/mTLS responder in `tests/consumers/0093-journeys/local_journey_responder.py`; it decodes and checks requests and returns encoded TTLV responses |

The next catalog slice expands this table to one row per assigned operation
and one row per normative client requirement. A static count alone does not
establish implementation or conformance.
