# KMIPKit Interoperability & External Validation Suites

This directory contains standalone, multi-language validation suites designed to test
KMIP 2.1 servers (e.g., Cosmian KMS, Utimaco ESKM, Thales CipherTrust Manager, etc.)
and verify end-to-end conformance with KMIPKit across all supported languages:

- `interop/shared/`: Shared JSON schemas, normative test catalog, server capability profiles, and CLI tools.
- `interop/rust-kmip-validator/`: Standalone Rust test suite (PR #2).
- `interop/c-kmip-validator/`: Standalone C test suite (PR #3).
- `interop/java-kmip-validator/`: Standalone Java test suite (PR #4).
- `interop/python-kmip-validator/`: Standalone Python test suite (PR #5).

## Directory Structure

```text
interop/
├── README.md
├── shared/
│   ├── schemas/
│   │   ├── server-connection.schema.json
│   │   ├── server-capabilities.schema.json
│   │   ├── test-catalog.schema.json
│   │   └── validation-results.schema.json
│   ├── catalog/
│   │   ├── generate_test_catalog.py
│   │   └── kmipkit-1.0-test-catalog.json
│   ├── profiles/
│   │   ├── cosmian-kms.json
│   │   ├── utimaco-eskm.json
│   │   ├── thales-ciphertrust.json
│   │   ├── generic-kmip-2.1.json
│   │   └── connection.example.json
│   ├── tools/
│   │   └── configure_capabilities.py
│   └── tests/
│       └── test_schemas_and_profiles.py
```

## Quick Start: Testing Profile Capabilities

To inspect matching test cases for a server capability profile:

```bash
python3 interop/shared/tools/configure_capabilities.py \
  --profile interop/shared/profiles/cosmian-kms.json \
  --summary
```

To regenerate or verify the exhaustive test catalog against OASIS KMIP 2.1:

```bash
python3 interop/shared/catalog/generate_test_catalog.py --check
```
