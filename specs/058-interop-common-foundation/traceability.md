| Requirement ID | Normative Reference / Project Decision | Implementation Path | Verification Reference |
|---|---|---|---|
| KMIPKIT-0058-FR-001 | Project interoperability architecture decision | `interop/shared/schemas/` | `interop/shared/tests/test_schemas_and_profiles.py::InteropSharedTests.test_schemas_are_valid_json` |
| KMIPKIT-0058-FR-002 | OASIS KMIP 2.1 Specification §§2–6; `specification/catalog/kmip-2.1.json` | `interop/shared/catalog/generate_test_catalog.py` | `interop/shared/tests/test_schemas_and_profiles.py::InteropSharedTests.test_catalog_covers_all_56_c2s_operations` |
| KMIPKIT-0058-FR-003 | Vendor interoperability specification | `interop/shared/profiles/` | `interop/shared/tests/test_schemas_and_profiles.py::InteropSharedTests.test_profiles_conform_to_capabilities_schema` |
| KMIPKIT-0058-FR-004 | Project CI efficiency decision | `scripts/ci_impact.py` | `scripts/tests/test_ci_impact.py::CiImpactTests.test_each_isolated_component_has_only_its_checks_and_coverage` |
| KMIPKIT-0058-FR-005 | Project CI automation decision | `.github/workflows/interop-common.yml` | `interop/shared/tests/test_schemas_and_profiles.py` |
