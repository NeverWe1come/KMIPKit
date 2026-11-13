# Traceability: KMIPKIT-0126

| Requirement | Source Reference | Design Element | Verification |
|---|---|---|---|
| KMIPKIT-0126-FR-001 | OASIS KMIP 2.1 §11.56 | `kmip-versioned-tags.json`, `generate_attribute_policy.py` | Unit tests & generator check |
| KMIPKIT-0126-FR-002 | OASIS KMIP 2.1 §3, KMIP 3.0 §11.56 | `generate_attribute_types.py`, 6 generated files | Parity tests (Rust, C, Java, Python) |
| KMIPKIT-0126-FR-003 | OASIS KMIP 3.0 §11.56 (Table 603) | `kmip-versioned-values.json`, `generate_wire_names.py` | Unit tests & generator check |
| KMIPKIT-0126-FR-004 | OASIS KMIP 3.0 §11, §12 | `kmip-versioned-values.json` | Catalog check & unit tests |
| KMIPKIT-0126-FR-005 | OASIS KMIP 2.0 §11.55, §11.33 | `version_gate.rs`, `kmip20_tests.rs`, spec 070 | Unit tests |
| KMIPKIT-0126-FR-006 | OASIS KMIP 3.0 §4, §6 | `batch.rs`, C/Java/Python bindings | Client batch unit & integration tests |
| KMIPKIT-0126-FR-007 | Spec 070, AGENTS.md §6 | `interop/java-kmip-validator/` | Maven test in java-kmip-validator |
