# Specification: Standalone Python KMIP 2.1 Validator

## 1. Overview
The standalone Python KMIP 2.1 validator is an independent Python application that validates KMS implementations (Cosmian KMS, Utimaco ESKM, Thales CipherTrust Manager) and KMIPKit Python bindings against the canonical test catalog.

## 2. Requirements
1. **Isolated Workspace**: Independent Python project located at `interop/python-kmip-validator/` using standard `pyproject.toml`.
2. **Catalog and Capability Ingestion**: Load canonical test catalog (`kmipkit-1.0-test-catalog.json`) and KMS profile JSON schemas without external dependencies.
3. **Capability Matching**: Support filtering test cases according to supported operations, managed objects, and attributes.
4. **Collision Avoidance**: Provide `AttemptContext` to generate isolated, collision-free object names across tests and runs.
5. **Driver Abstraction**: Provide `KmipDriver` interface and `MockKmipDriver` for deterministic dry-runs and integration tests.
6. **Multi-Format Reporting**: Output ANSI terminal tables with symbols (`✅`, `❌`, `➖`), GitHub Flavored Markdown (`report.md`), interactive HTML (`report.html`), and machine-readable JSON (`results.json`).
7. **Cross-Language Parity**: Verify identical pass/unsupported counts against Cosmian KMS profile (40 passed, 83 unsupported).
8. **Lightweight CI**: Dedicated GitHub Actions workflow triggered only on Python validator or shared interop file changes.
