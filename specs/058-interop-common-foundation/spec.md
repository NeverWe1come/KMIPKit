# Feature Specification: KMIP 2.1 Interoperability Common Foundation and CI Routing

**Feature Branch**: `feature/KMIPKIT-0058-interop-common-foundation`
**Status**: Approved for implementation by maintainer plan authorization
**Owning identifier**: KMIPKIT-0058

Provide the shared schemas, test catalog generator, KMS capability profiles,
and light-CI routing for external KMIP 2.1 interoperability test suites.

## Purpose

Define the common foundation shared by standalone multi-language test suites
(`interop/rust-kmip-validator`, `interop/c-kmip-validator`,
`interop/java-kmip-validator`, and `interop/python-kmip-validator`), including:
1. JSON schemas for connection, capabilities, test catalog, and results.
2. An exhaustive test catalog derived from `specification/catalog/kmip-2.1.json`.
3. Server capability profiles for Cosmian KMS, Utimaco ESKM, Thales CipherTrust, and generic KMIP 2.1.
4. CI routing in `scripts/ci_impact.py` that excludes `interop/` changes from full KMIPKit library CI overhead.

## Requirements

| ID | Requirement | Normative source / project decision | Verification |
|---|---|---|---|
| KMIPKIT-0058-FR-001 | The shared schema set SHALL define JSON schemas for server connections, server capabilities, test catalog, and validation results. | Project interoperability architecture decision | JSON schema validation tests in `interop/shared/tests/test_schemas.py` |
| KMIPKIT-0058-FR-002 | The test catalog generator SHALL cover 100% of the 56 client-to-server operations, 9 managed object types, 63 attributes, 64 enumerations, and 3 bit masks from the normative catalog. | OASIS KMIP 2.1 Specification §§2–6; `specification/catalog/kmip-2.1.json` | Catalog completeness check in `interop/shared/catalog/generate_test_catalog.py --check` |
| KMIPKIT-0058-FR-003 | Server capability profiles SHALL be provided for Cosmian KMS, Utimaco ESKM, Thales CipherTrust, and Generic KMIP 2.1 conforming to the capabilities schema. | Vendor interoperability specification | Profile conformance tests in `interop/shared/tests/test_profiles.py` |
| KMIPKIT-0058-FR-004 | Changes confined to `interop/` SHALL be classified as lightweight `interop` scope in `scripts/ci_impact.py` without triggering library compilation, fuzzing, or coverage gates. | Project CI efficiency decision | Unit tests in `scripts/tests/test_ci_impact.py` |
| KMIPKIT-0058-FR-005 | A dedicated GitHub Actions workflow `.github/workflows/interop-common.yml` SHALL validate schemas, catalog generation, and capability profiles when `interop/shared/**` changes. | Project CI automation decision | Workflow schema validation and local test suite execution |
