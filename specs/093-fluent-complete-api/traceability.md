# KMIPKIT-0093 Traceability

| Requirement | Pinned source and existing contract | Verification |
|---|---|---|
| FR-001–003 catalog | KMIP 2.1 §11.36 Table 470, §11.56 Table 487; KMIP 3.0 §11.37 Table 584, §11.58 Table 605; KMIPKIT-0047, -0069, -0071 | Exhaustive assigned-value and version/direction matrix; generator snapshot and negative unassigned-value tests |
| FR-004–005 operation wrappers | KMIP 2.1 §6.1 operation tables, §§8–9; KMIPKIT-0007, -0051, -0052, -0091 | Per-operation build, validation, batch conversion, real TLS fixture and result status tests |
| FR-006 expert attributes | KMIP 2.1 §§4–5, §7.1, §11.56; KMIPKIT-0048, ADR-0012, ADR-0014 | Wrong type, unknown tag, vendor structure, order, version, reserved tag, redaction, and no-send negative tests |
| FR-007 caller choices | KMIP 2.1 relevant §6.1 operation tables; ADR-0022; KMIPKIT-0091 | Exact preset value assertions; missing crypto decision and no-retry tests |
| FR-008–010 responses and encodings | KMIP 2.1 §§8–9; Profiles §§5.4–5.5; ADR-0018 encoding and response ownership; KMIPKIT-0052, -0054–0057 | Missing field, Success/Pending/Failure, owner lifetime, secret redaction, TTLV/JSON/XML round trips and error isolation |
| FR-011 four languages | KMIPKIT-0043, -0053; Constitution III | C/JNI/CFFI ownership, ABI audit, Java/Python typing, compiled consumers and parity matrix |
| FR-012 server direction | KMIP 2.1 §6.2, KMIPKIT-0066, ADR-0021 | Listener handler fixtures; no client send for server-only operation |
| FR-013 journeys | KMIP 2.1 operation tables and Profiles §§5.3–5.5 | Five per-language applications against controlled TLS fixture |

The next catalog slice expands this table to one row per assigned operation
and one row per normative client requirement. A static count alone does not
establish implementation or conformance.
