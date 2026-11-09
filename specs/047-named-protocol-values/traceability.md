# KMIPKIT-0047 Traceability

| Requirement | Source | Implementation | Verification | Status |
|---|---|---|---|---|
| FR-001 | KMIP 2.1 §§11–12; assigned `enumeration_value` catalog entries (644 concrete values; the catalog has 723 total rows including reserved/extension placeholders) | `tools/normative_catalog/generate_named_values.py`; generated Rust/C/Java/Python values | T001–T003, T009 | Implemented; local checks recorded |
| FR-002 | KMIP 2.1 §§11–12; assigned `bitmask_value` catalog entries (33 concrete bits; the catalog has 44 total rows including reserved/extension placeholders) | `tools/normative_catalog/generate_named_values.py`; generated Rust/C/Java/Python values | T001–T003, T009 | Implemented; local checks recorded |
| FR-003 | KMIP 2.1 §§11–12; unknown/future value preservation policy | Generated lookup helpers and open raw representations | T002, T003 | Implemented; local checks recorded |
| FR-004 | KMIP 2.1 §§11–12; unknown/future value preservation policy | Generated known/unknown bit helpers | T002, T003 | Implemented; local checks recorded |
| FR-005 | Constitution §3; KMIPKIT-0043 parity contract | Rust, C, Java, and Python package surfaces and consumers | T002–T004 | Implemented; local checks recorded |
| FR-006 | Constitution §10; reviewed KMIP 2.1 and checksum-verified edition catalogs | Deterministic catalog-backed generator and generated artifacts | T001, T004, T012 | Implemented; named-value and manifest `--check` commands pass |
| FR-007 | Constitution §§4, 9; KMIPKIT-0043 API compatibility | Additive constants and lookup helpers; raw constructors and existing paths retained | T002, T006 | Implemented; local checks recorded |
| FR-008 | Pinned KMIP 2.1 §11.36 Table 470 and KMIP 3.0 §§11.34–11.35 Tables 581–582, §11.37 Table 583; checksum-verified supplemental catalog | `specification/catalog/kmip-versioned-values.json`; `load_versioned_values` | T010–T012 | Implemented; checksum and exact-assignment tests pass |
| FR-009 | Same exact versioned OASIS Enumeration tables | Rust/C/Java/Python generated version namespaces and lookup helpers | T010–T012 | Implemented; Rust, C, Java, and Python consumer checks pass |
| FR-010 | KMIP 2.1 §11.36 Table 470 and KMIP 3.0 §11.37 Table 583; `8XXXXXXX` extension allocation in the Operation Enumeration | `VersionGate::operation_span`, `VersionGate::validate_operation`, and the exact extension-range predicate | T010–T012, T017–T019 | Implemented; tests reject unassigned standard values and preserve assigned vendor-extension values |
| FR-011 | KMIP 2.1 §11.56 and KMIP 3.0 §11.58 Table 605 exact tag assignments | `VersionGate::tag_span`; explicit KMIP 3.0 tag assignment list | T014–T016; `version_foundation_tests::test_version_gate_uses_exact_kmip_30_tag_assignments` | Implemented; 2.1 legacy tags remain accepted and exact 3.0 tags are gated |
