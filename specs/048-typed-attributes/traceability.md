# KMIPKIT-0048 Traceability

The per-attribute stable API trace identifiers and their exact OASIS section,
table, tag, encoding, and multiplicity are in [gap-matrix.md](gap-matrix.md).
The 62 `KMIPKIT-0048-ATTR-<tag>` rows map to generated attribute markers in
`crates/kmipkit-protocol/src/typed_attributes_generated.rs`; the complete
catalog-to-generated mapping is verified by
`test_catalog_defines_all_and_only_direct_typed_attributes` and
`generated_typed_attribute_registry_covers_every_direct_definition`.
`KMIPKIT-0048-EXC-CERTIFICATE-ATTRIBUTES` records the source-justified
exclusion in §4.6/Table 39.

| Requirement | Normative source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0048-FR-001, FR-002 | KMIP Specification v2.1 §§4.1–4.63; per-attribute encoding tables in `gap-matrix.md`; §11.56 for tag allocation | generated marker definitions and value-category mapping | exhaustive catalog mapping test and per-category constructor/read tests |
| KMIPKIT-0048-FR-003 | KMIP Specification v2.1 §§5.1, 5.2, 5.5; Tables 157–158, 161; §4 multiplicity tables | `AttributeSet` typed insertion and ordered typed queries | repeated-value order, lossless duplicate retention, operation-level single-instance rejection, and generic-preservation tests |
| KMIPKIT-0048-FR-004 | KMIP Specification v2.1 §§5.6–5.7, Tables 162–163 | `CurrentAttribute` and `NewAttribute` typed constructors/views | valid typed wrapper and mismatch negative tests |
| KMIPKIT-0048-FR-005 | KMIP Specification v2.1 §5; KMIPKit security contract, `AGENTS.md` §8 | typed attribute error conversion | errors contain only fixed categories/tags and no values |
| KMIPKIT-0048-FR-006 | KMIP Specification v2.1 §§5.1–5.4, §10.1.2 | typed conversion over existing `Item`, `Value`, and `Structure` model | scalar/Structure round-trip and unknown-child preservation tests |
| KMIPKIT-0048-FR-007 | KMIP Specification v2.1 §§4, 5.1–5.7; accepted KMIPKIT-0016 attribute policy | existing generic APIs and `AttributeSet` policy remain the implementation authority | existing multiplicity, vendor-attribute, and mutation-preflight regressions |
| KMIPKIT-0048-FR-008 | KMIPKit generated-artifact contract, `AGENTS.md` §10 | normative catalog generator and public API manifest inputs/outputs | generator `--check`, manifest audit, and exact public declaration tests |
| KMIPKIT-0048-FR-009 | Documentation requirement, not a new protocol rule | English and Spanish attribute operation guides | executable Rust examples / documentation tests |
| KMIPKIT-0048-FR-010 | Approved Phase 5 plan, KMIPKIT-0053 dependency | binding parity is not implemented in this branch | 0053 traceability entry and parity criteria |
| KMIPKIT-0048-FR-011 | KMIP Specification v2.1 §4.31 Table 90 and §11.28 Table 465; KMIP Specification 3.0 §§4.35.1–4.35.20 and §11.58 Tag Enumeration | `crates/kmipkit-protocol/src/link_normalizer.rs`; `specification/catalog/kmip-versioned-tags.json`; generated `crates/kmipkit-ttlv/src/generated/tag_allocations.rs` | `link_normalizer_tests.rs` exact 14-entry bidirectional mapping, Enumeration/Integer identifier round trips, Text String rejection, and unmapped-tag/child rejection; `test_generate_ttlv_tags.py` checks all 20 assigned tags and the source checksum |
| KMIPKIT-0048-FR-012 | KMIP Specification 3.0 §11.58, Table 605; supported-version allocation contract | `specification/catalog/kmip-versioned-tags.json`; `tools/normative_catalog/generate_ttlv_tags.py`; generated `crates/kmipkit-ttlv/src/generated/tag_allocations.rs` | `test_generate_ttlv_tags.py::test_renders_every_assigned_kmip30_tag_from_table_605` checks all 60 exact assignments and generated allocation states |

## Per-attribute verification rule

Every non-excluded row in `gap-matrix.md` has one stable ID derived from its
immutable tag. The exhaustive mapping test compares that row's catalog
`element_id`, tag, normalized encoding, and multiplicity with the generated
marker registry. Category tests exercise each distinct TTLV representation;
wrapper tests exercise repeated and singleton retrieval behavior. No unknown
tag or value is rewritten by typed access.
