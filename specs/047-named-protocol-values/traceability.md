# KMIPKIT-0047 Traceability

| Requirement | Source | Implementation | Verification | Status |
|---|---|---|---|---|
| FR-001 | KMIP 2.1 §§11–12; assigned `enumeration_value` catalog entries (644 concrete values; the catalog has 723 total rows including reserved/extension placeholders) | `tools/normative_catalog/generate_named_values.py`; generated Rust/C/Java/Python values | T001–T003, T009 | Implemented; local checks recorded |
| FR-002 | KMIP 2.1 §§11–12; assigned `bitmask_value` catalog entries (33 concrete bits; the catalog has 44 total rows including reserved/extension placeholders) | `tools/normative_catalog/generate_named_values.py`; generated Rust/C/Java/Python values | T001–T003, T009 | Implemented; local checks recorded |
| FR-003 | KMIP 2.1 §§11–12; unknown/future value preservation policy | Generated lookup helpers and open raw representations | T002, T003 | Implemented; local checks recorded |
| FR-004 | KMIP 2.1 §§11–12; unknown/future value preservation policy | Generated known/unknown bit helpers | T002, T003 | Implemented; local checks recorded |
| FR-005 | Constitution §3; KMIPKIT-0043 parity contract | Rust, C, Java, and Python package surfaces and consumers | T002–T004 | Implemented; local checks recorded |
| FR-006 | Constitution §10; checked-in catalog generation contract | Deterministic catalog-backed generator and generated artifacts | T001, T004 | Implemented; generator `--check` recorded |
| FR-007 | Constitution §§4, 9; KMIPKIT-0043 API compatibility | Additive constants and lookup helpers; raw constructors and existing paths retained | T002, T006 | Implemented; local checks recorded |
