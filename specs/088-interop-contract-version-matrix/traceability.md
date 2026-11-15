# Requirement traceability

| Requirement | Normative/project source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0088-FR-001 | OASIS KMIP 2.0/2.1 §§6.1–6.2, §11, §12; pinned source HTML | `generate_test_catalog.py` source inventories and `execution.protocol_versions` | Source inventory set equality and per-case applicability tests |
| KMIPKIT-0088-FR-002 | OASIS KMIP 2.0/2.1 version-specific operation, attribute, object, enum, and bit-mask sections | `normative_refs` generation | Every applicable case/version has exact OASIS reference test |
| KMIPKIT-0088-FR-003 | OASIS KMIP 2.1 §3 (pre-active Encrypt errors); shared catalog case descriptions | `requires.expected_result_reasons` | Schema and reason allowlist tests for pre-active Encrypt and active Destroy |
| KMIPKIT-0088-FR-004 | KMIPKIT-0083-FR-005 shared result identity | `server-connection.schema.json`, `connection.example.json` | Stable fingerprint pattern and missing/invalid value tests |
| KMIPKIT-0088-FR-005 | KMIPKIT-0083-FR-005 | `aggregate_matrix.py` grouped run matrices | Multi-run same-language preservation and per-version grouping tests |
| KMIPKIT-0088-FR-006 | KMIPKIT-0083-FR-005; OASIS Result Status and Result Reason enumerations | Aggregator evidence gate and enum-name-to-wire-code normalization | Incomparable/unverified gate, enum alias, reason, parity, and correctness-difference tests |
| KMIPKIT-0088-FR-007 | KMIPKIT-0083-FR-005/006 | `validation-results.schema.json`; JSON, Markdown, and HTML generators | Multi-axis, correctness, compatibility, not-comparable/unverified, not-run and redaction tests |
| KMIPKIT-0088-FR-008 | `AGENTS.md` §§6 and 10 | `.github/workflows/interop-common.yml` | Catalog/vector regeneration and shared unittest workflow |
