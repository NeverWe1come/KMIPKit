# Requirement Traceability

| Requirement | Normative / project source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0083-FR-001 | KMIP 2.0/2.1 §§2–9; `specification/catalog/kmip-2.1.json` | `interop/shared/catalog/`, `interop/shared/schemas/test-catalog.schema.json` | `interop/shared/tests/test_executable_contract.py` |
| KMIPKIT-0083-FR-002 | KMIP 2.0/2.1 §§2–9; `specification/catalog/kmip-2.1.json` | `interop/shared/catalog/generate_test_catalog.py` | `interop/shared/tests/test_executable_contract.py` |
| KMIPKIT-0083-FR-003 | Interoperability evidence policy in `docs/development/kmip-interoperability.md` | Catalog `capability_policy` | Capability selection and aggregation tests (pending implementation) |
| KMIPKIT-0083-FR-004 | `AGENTS.md` §8; KMIPKIT-0083-FR-004 | Catalog `execution_policy`; connection schema | Runner policy tests (pending implementation) |
| KMIPKIT-0083-FR-005 | Interoperability evidence policy in `docs/development/kmip-interoperability.md` | `interop/shared/schemas/validation-results.schema.json` | Result validation and aggregator tests (pending implementation) |
| KMIPKIT-0083-FR-006 | `AGENTS.md` §8 | Shared result and reporting contract | Secret-redaction tests (pending implementation) |
| KMIPKIT-0083-FR-007 | Public API manifest, `specification/api/public-api.json` | Catalog limitation metadata (pending implementation) | Catalog contract test (pending implementation) |
| KMIPKIT-0083-FR-008 | KMIPKIT-0058–0062; release artifact workflow | CI provenance and consumer installs (language PRs) | Offline package consumer workflows (pending implementation) |
